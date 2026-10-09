## Outcome

Describe the problem and resulting behavior. Link the issue with `Closes #...` only when this PR completes it.

## Scope

- Changed:
- Deferred or excluded:
- Related decisions or dependencies:

## Verification

List the checks actually run and their results. Include screenshots for UI changes and relevant failure/recovery evidence for pipeline changes.

```text
python3 scripts/check_repository.py
```

Add applicable build, test, contract, or device checks once those implementations exist. Explain checks that could not be run.

## Compatibility and production impact

Describe migrations, Script IR/API compatibility, affected cache entries and audio revisions, provider costs, rights/approval changes, and rollout or recovery steps. Write `N/A` when irrelevant.

## Review checklist

- [ ] The PR targets `develop` and has one reviewable purpose.
- [ ] Acceptance criteria are met, or remaining work is explicitly linked.
- [ ] Documentation and reusable templates reflect the resulting behavior.
- [ ] Logs, examples, and fixtures contain no secrets or unapproved source material.
- [ ] Applicable checks and remaining limitations are reported accurately.
