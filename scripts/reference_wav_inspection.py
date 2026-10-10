#!/usr/bin/env python3
"""Independent differential oracle for the inspect-wav CLI on stdlib-written PCM files.

Python's `wave` module writes every file and the oracle decodes samples with plain integer
arithmetic; hashlib supplies checksums. When ffprobe is on PATH it cross-checks container
metadata as a third reader. Files are synthetic, seeded and written to a temporary directory
that is always removed. This checks structure, checksums and sample peak only: it is not a
loudness, true-peak, clipping or listening check, and it never writes repository files.
"""

import argparse
import hashlib
import json
import random
import shutil
import subprocess
import sys
import tempfile
import wave
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BINARY = ROOT / "target/debug/inspect-wav"
# (channels, sample rate, bytes per sample, frames)
CASES = (
    (1, 8000, 1, 257),
    (2, 44100, 2, 1000),
    (1, 48000, 3, 333),
    (2, 96000, 4, 64),
    (6, 48000, 2, 10),
)


def frames_for(rng, channels, width, count):
    """Random codes that include each extreme once, so sign handling is always exercised."""
    if width == 1:
        low, high = 0, 255
    else:
        low, high = -(1 << (8 * width - 1)), (1 << (8 * width - 1)) - 1
    codes = [rng.randint(low, high) for _ in range(channels * count)]
    codes[rng.randrange(len(codes))] = low
    codes[rng.randrange(len(codes))] = high
    if width == 1:
        return codes, bytes(codes)
    encoded = b"".join(code.to_bytes(width, "little", signed=True) for code in codes)
    return codes, encoded


def expected_peaks(codes, channels, width):
    peaks = []
    for channel in range(channels):
        magnitudes = [
            abs(code - 128) if width == 1 else abs(code)
            for code in codes[channel::channels]
        ]
        magnitude = max(magnitudes)
        peaks.append((magnitude, magnitudes.index(magnitude)))
    return peaks


def ffprobe_stream(path):
    result = subprocess.run(
        ["ffprobe", "-v", "error", "-show_entries",
         "stream=sample_rate,channels,bits_per_sample,duration_ts", "-of", "json", str(path)],
        capture_output=True, check=True, text=True,
    )
    return json.loads(result.stdout)["streams"][0]


def check_case(binary, directory, rng, case):
    channels, rate, width, count = case
    codes, payload = frames_for(rng, channels, width, count)
    path = directory / f"pcm-{channels}ch-{rate}-{8 * width}bit.wav"
    with wave.open(str(path), "wb") as writer:
        writer.setnchannels(channels)
        writer.setsampwidth(width)
        writer.setframerate(rate)
        writer.writeframes(payload)
    with wave.open(str(path), "rb") as reader:
        assert reader.readframes(count) == payload, "wave did not round-trip its frames"
    result = subprocess.run([str(binary), str(path)], capture_output=True, check=False)
    if result.returncode != 0:
        raise AssertionError(f"{path.name}: exit {result.returncode}: {result.stderr!r}")
    report = json.loads(result.stdout)
    peak = report["measurements"]["sample_peak"]
    observed = {
        "warnings": report["warnings"],
        "file_sha256": report["file"]["sha256"],
        "data_sha256": report["data"]["sha256"],
        "fmt": [report["fmt"][key] for key in
                ("format_tag", "channels", "sample_rate_hz", "bits_per_sample", "block_align")],
        "frames": report["measurements"]["frames"],
        "full_scale": peak["full_scale"],
        "peaks": [(item["magnitude"], item["first_frame"]) for item in peak["channels"]],
    }
    # `wave` omits the pad byte of odd-sized data and leaves it out of the RIFF size.
    unpadded = [{"code": "unpadded_final_chunk", "id": "data", "offset": 36}]
    oracle = {
        "warnings": unpadded if len(payload) % 2 else [],
        "file_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "data_sha256": hashlib.sha256(payload).hexdigest(),
        "fmt": [1, channels, rate, 8 * width, channels * width],
        "frames": count,
        "full_scale": 1 << (8 * width - 1),
        "peaks": expected_peaks(codes, channels, width),
    }
    if observed != oracle:
        raise AssertionError(f"{path.name}: {observed} != {oracle}")
    if shutil.which("ffprobe"):
        stream = ffprobe_stream(path)
        probed = (int(stream["sample_rate"]), stream["channels"],
                  stream["bits_per_sample"], stream["duration_ts"])
        if probed != (rate, channels, 8 * width, count):
            raise AssertionError(f"{path.name}: ffprobe {probed}")
    return path.name


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", nargs="?", type=Path, default=DEFAULT_BINARY)
    parser.add_argument("--seed", type=int, default=20261010)
    args = parser.parse_args()
    if not args.binary.is_file():
        print(f"{args.binary} not found; run `cargo build --locked --bin inspect-wav`",
              file=sys.stderr)
        return 2
    rng = random.Random(args.seed)
    with tempfile.TemporaryDirectory(prefix="cantos-wav-oracle-") as directory:
        try:
            names = [check_case(args.binary, Path(directory), rng, case) for case in CASES]
        except (AssertionError, subprocess.CalledProcessError) as error:
            print(f"WAV inspection oracle mismatch: {error}", file=sys.stderr)
            return 1
    third_reader = "with ffprobe" if shutil.which("ffprobe") else "without ffprobe"
    print(f"inspect-wav agrees with Python wave/hashlib {third_reader}: {', '.join(names)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
