# token-lang - Roadmap

> Path from scaffold to a stable 1.0. Hard parts are front-loaded; each phase has hard exit criteria.
> Master plan: ../../_strategy/LANG_COLLECTION.md
>
> **Anti-deferral rule:** no listed hard task moves to a later phase unless this file records the move and the reason.

## v0.1.0 - Scaffold (DONE)
Compiles, CI green, structure correct, no domain logic.
- [x] Manifest, README, CHANGELOG, REPS, dual license, CI, deny, clippy, rustfmt.

## v0.2.0 - Core (DONE)
Token kinds and the token type only, so lexer and parser stay independent of each other.
Dependencies (wires span, intern) are wired here, when first used.
Exit criteria:
- [x] Every public item has rustdoc + a runnable example.
- [x] Core invariants property-tested (full DIRECTIVES + API authored at this stage).

Delivered: `Token<K>` (kind + span; wires `span-lang`) and the `TokenKind` trait
(`is_trivia` / `is_eof` / `symbol`; the last wires `intern-lang`'s `Symbol`), with
`Token` ↔ `Spanned` conversions, source-order ordering, optional `serde`, and a
`no_std` build. `dev/DIRECTIVES.md` and `docs/API.md` authored; property tests cover
every section-4 invariant. See `dev/NOTES.md` for the one frozen-surface decision
(the `symbol` accessor) awaiting sign-off before the 1.0 freeze.

## v1.0.0 - API freeze (DONE)
Public surface stable and frozen until 2.0.
- [x] docs/API.md marked stable; SemVer promise recorded.
- [x] Full test + benchmark suite green on all three platforms.

No code changes from v0.2.0 — the freeze marks the contract. The `symbol` accessor
decision resolved as option A (kept on `TokenKind`) and is now part of the stable
1.0 surface.
