# Web implementation guide

This directory is reserved for Leptos Web. No Leptos package or dev server exists yet.

Studio and Theatre are distinct product surfaces and permission contexts; start with one Web package if that keeps the first slice smaller. Studio prioritizes a readable script editor, casting and accountable production states. Theatre exposes published discovery and cached audio playback.

Use the [UI system](../../docs/design/ui-system.md), [business rules](../../docs/product/business-rules.md) and [work plan](../../docs/work-plan/README.md). Preserve editor data through failed requests; keep mini-player state across navigation. Provider credentials and private object-store access never belong in browser code.
