#!/usr/bin/env python3
"""Check bootstrap documents, local links, JSON fixtures and text hygiene.

This does not validate application behavior, YAML schemas or audio quality.
It uses only Python's standard library and Git.
"""

import json
import re
import subprocess
import sys
from pathlib import Path
from urllib.parse import unquote, urlsplit


ROOT = Path(__file__).resolve().parents[1]
REQUIRED = (
    "README.md",
    "AGENTS.md",
    "CONTRIBUTING.md",
    "docs/README.md",
    "docs/product/brief.md",
    "docs/product/business-rules.md",
    "docs/product/production-pipeline.md",
    "docs/architecture/overview.md",
    "docs/architecture/script-ir.md",
    "docs/architecture/mobile.md",
    "docs/design/ui-system.md",
    "docs/work-plan/README.md",
    "templates/README.md",
    "contracts/examples/episode-draft.json",
    ".github/pull_request_template.md",
    ".github/ISSUE_TEMPLATE/config.yml",
    ".github/workflows/repository-checks.yml",
)
TEXT_SUFFIXES = {".md", ".py", ".json", ".yml", ".yaml", ".example"}
TEXT_NAMES = {".gitignore", ".editorconfig"}
LINK = re.compile(r"!?\[[^\]\n]*\]\(([^)\n]+)\)")
FENCE = re.compile(r"^\s*(`{3,}|~{3,})")


def prose_only(content):
    """Skip fenced examples; paths shown as examples need not exist."""
    lines = []
    fence = None
    for line in content.splitlines():
        marker = FENCE.match(line)
        if marker:
            token = marker.group(1)
            if fence is None:
                fence = token
            elif token[0] == fence[0] and len(token) >= len(fence):
                fence = None
            continue
        if fence is None:
            lines.append(line)
    return "\n".join(lines)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def check():
    errors = []
    for name in REQUIRED:
        if not (ROOT / name).is_file():
            errors.append(f"missing required file: {name}")

    result = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=ROOT, capture_output=True, check=True,
    )
    names = sorted(set(result.stdout.decode("utf-8").rstrip("\0").split("\0")))
    checked = 0
    for name in filter(None, names):
        path = ROOT / name
        if path.suffix not in TEXT_SUFFIXES and path.name not in TEXT_NAMES:
            continue
        if not path.is_file():
            errors.append(f"missing tracked file: {name}")
            continue
        try:
            content = path.read_bytes().decode("utf-8")
        except UnicodeDecodeError:
            errors.append(f"{name}: expected UTF-8 text")
            continue
        checked += 1
        if not content.endswith("\n"):
            errors.append(f"{name}: missing final newline")
        if "\r" in content:
            errors.append(f"{name}: expected LF line endings")
        for number, line in enumerate(content.splitlines(), 1):
            # Markdown two-space hard breaks are intentional.
            trailing = line[len(line.rstrip()):]
            if trailing and not (path.suffix == ".md" and trailing == "  "):
                errors.append(f"{name}:{number}: trailing whitespace")

        if path.suffix == ".json":
            try:
                json.loads(content, object_pairs_hook=unique_object)
            except (ValueError, json.JSONDecodeError) as exc:
                errors.append(f"{name}: invalid JSON: {exc}")

        if path.suffix != ".md":
            continue
        for match in LINK.finditer(prose_only(content)):
            target = match.group(1).strip().split(' "', 1)[0].strip("<>")
            parsed = urlsplit(target)
            if parsed.scheme or parsed.netloc or not parsed.path:
                continue
            relative = unquote(parsed.path)
            destination = ROOT / relative.lstrip("/") if relative.startswith("/") else path.parent / relative
            if not destination.exists():
                errors.append(f"{name}: broken local link: {target}")

    if errors:
        print("Repository checks failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print(f"Repository checks passed: {checked} text files; local links and JSON syntax valid.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(check())
    except (OSError, subprocess.CalledProcessError) as exc:
        print(f"Unable to check repository: {exc}", file=sys.stderr)
        sys.exit(1)
