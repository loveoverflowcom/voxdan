# Cantos Studio Web

`cantos-studio` is a minimal Leptos 0.7.8 CSR/WASM consumer of the actual local Axum/PostgreSQL
revision API. It displays a complete JSON draft and a read-only stored export, plus bounded
manuscript import and source/extraction comparison, plus a bounded AI proposal review flow.
The full scene/speaker editor, production console and Cantos Theatre remain planned. The
adapter's live model gate is separately recorded in the
[adaptation evidence](../../docs/evidence/ai-script-adaptation.md).

## Build and run

Requires Rust 1.87, `wasm32-unknown-unknown`, Trunk 0.21.14 and matching wasm-bindgen 0.2.100.
From the repository root:

```sh
rustup target add wasm32-unknown-unknown --toolchain 1.87.0
cargo clippy -p cantos-studio --target wasm32-unknown-unknown --locked -- -D warnings
cd apps/web
NO_COLOR=true trunk build --locked
```

Trunk fetches its matching wasm-bindgen helper on its first build. An offline build requires
that helper on `PATH` as well as the Cargo cache; the Mac run used
`~/Library/Caches/dev.trunkrs.trunk/wasm-bindgen-0.2.100`. Generated `dist/` is ignored.
Serve the built files with the local Axum host, using the [isolated database runner](../server/README.md).
The actual walkthrough used `python3 scripts/serve_studio.py` at `http://127.0.0.1:8080`.

`Trunk.toml` also declares an optional 8081 development proxy to 8080. If using
`trunk serve --locked`, explicitly set the backend's `CANTOS_WEB_ORIGIN=http://127.0.0.1:8081`.
That proxy mode was not used as browser evidence.

## Interaction and ownership

Sign in on each page with an operator-issued development token. It becomes a same-origin
HttpOnly/SameSite=Strict session cookie; the form clears the token after success. Credentials
are not kept in local storage or build-time config. No fake backend or fallback dataset exists.
The visible mode label identifies the live development backend.

The pure `editor.rs` owns immutable save snapshots, request tickets and error transitions.
Reads never replace nonempty local text. A stale save requires a read/compare and explicit
"keep text, use this head as base" action. A timeout preserves the original actor, operation
key, base and snapshot; retries reconcile that same operation before another save. Session
denial after an ambiguous attempt does not discard it. A different actor cannot retry it.
Editing during a request survives its acknowledgement.

The component obtains the concrete HTTP adapter from Studio context; URLs and wire decoding
stay in `api.rs`. The browser does not decide backend permissions or Script IR validity.
`contracts/api/` is shared with Axum; no server crate enters the WASM dependency graph.

Vietnamese/English resources cover every async/error state. Semantic labels, polite status,
visible focus and keyboard-operable controls use native HTML. Guarded aria-disabled actions
remain focusable with a reason. No animation or audio is implemented. Page-return handling
reapplies controlled values if Safari restores WASM state but clears autocomplete-off fields.
This is not durable draft persistence across reload/closing a tab.

The import section selects a file and explicit format, records its source and permission
evidence, then saves original bytes and the parse outcome. The receipt ID reopens the saved
source; original download returns the byte-exact upload. Failures retain the immutable request
for a same-operation retry by the same actor. A changed selection is a new operation.
Prose labels and scene suggestions remain review findings, unknown speakers remain visible,
and publication permission stays unverified. The UI renders manuscript markup as inert text.
It never assigns voices or silently submits an imported source as an accepted script.

The adaptation section pins an imported source, target script UUID and base revision. It shows
the configured server-side localhost provider and requires explicit source/adaptation permission
authorization. Run IDs reopen private proposals; source blocks, conversion warnings, coverage
claims and unsupported pacing/prosody remain inspectable beside the editable JSON candidate.
Provider success remains a proposal. A creator explicitly acknowledges the findings and accepts
the validated candidate, then reopens the pinned immutable revision. The ordinary editor's local
draft is separate. Unknown cost displays as unavailable; provider configuration and an HTTP
fixture result never prove a real model run.

An uncertain mutation retains its exact actor/operation/snapshot for reconciliation. A different
actor, stale response or changed input cannot silently replace the visible proposal or accepted
receipt. Revision conflicts require a new reviewed run against the current base. Full scene and
character authoring belongs to issue #4.

## Tokens and evidence

`contracts/design/studio-tokens-0.1.0.json` is the versioned source of semantic light/dark
colors and logical dimensions. Generate CSS with `python3 scripts/studio_tokens.py`; verify
drift with `python3 scripts/studio_tokens.py --check`. System/light/dark selection changes
only this page. System fonts retain Vietnamese diacritics; no downloaded font is bundled.
CMP mapping is deferred until its real runtime exists.

[Browser observations](../../docs/evidence/script-revision-persistence.md#browser-observations)
separate real keyboard/read/save and inspected captures from pure reducer tests and compilation.
VoiceOver, OS Telex/VNI composition, exact instrumented viewport coverage and native devices
remain unverified; no complete accessibility or full Studio acceptance claim is made.
