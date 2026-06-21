# token-lang &mdash; Engineering Directives

> Engineering standards and the definition of done for this project. Read alongside `REPS.md` (root, authoritative) and `dev/ROADMAP.md` (current phase). If anything here conflicts with `REPS.md`, `REPS.md` wins.

---

## 0. Philosophy

This library is built and maintained to a production standard and treated as a flagship piece of work. Plan the full path, then build one verified step at a time. "Good enough" is treated as a defect. token-lang is the seam every later front-end agrees on: a lexer writes tokens through it and a parser reads them back, so the shape of a token and the cost of classifying one are paid on every token of every compilation.

---

## 1. What this is

token-lang defines the token type and nothing else. A `Token<K>` pairs a language-specific *kind* `K` with the `Span` of source it covers; the `TokenKind` trait gives generic, language-agnostic code the handful of queries a parser asks of any token — is this trivia, is this the end, what interned text does it carry. It owns the token type alone: no scanning, no grammar, no diagnostics. The language supplies `K`; token-lang supplies the pairing and the seam, so a lexer and a parser depend on each other only through this crate.

---

## 2. Engineering law (non-negotiable)

- **Performance** — peak is the baseline; a `Token<K>` is `Copy` whenever `K` is and adds nothing to `K` but an eight-byte span; classification and the field accessors are `#[inline]` and allocation-free; no "faster" claim without `criterion` numbers.
- **Correctness** — the invariants in section 4 are covered by property tests, cross-checked against direct field access and `span-lang`'s `Spanned`.
- **Security** — the crate holds no unsafe code and does no I/O or parsing, so it has no input boundary of its own; it must never widen one (no panicking constructors, no hidden allocation) for the layers that do.
- **Architecture** — SOLID, KISS, YAGNI; one responsibility; the token type stays generic over the language's kind rather than enumerating any language's keywords.
- **Cross-platform** — Linux/macOS/Windows first-class, verified by CI.
- **Error handling** — the surface is total: construction and classification cannot fail, so there is no fallible path to mishandle. Anything that could fail belongs in the lexer, not here.
- **Production-ready** — `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` from the first commit; no stray `println!`/`dbg!`; every public item has rustdoc with a runnable example.

---

## 3. Definition of done

1. Compiles clean on Linux/macOS/Windows, stable and MSRV 1.85.
2. `fmt`, `clippy -D warnings`, `test --all-features`, `cargo doc -D warnings` clean.
3. `cargo audit` + `cargo deny check` pass.
4. No `unwrap`/`expect`/`todo!`/`dbg!` in shipping code.
5. A Tier-1 API exists and headlines the docs.
6. Property tests cover every section-4 invariant.
7. Hot-path changes carry benchmarks; no regression over 5%.
8. Docs and `CHANGELOG.md` updated; the matching `docs/release/vX.Y.Z.md` written before the tag.

---

## 4. Project-specific invariants

- A `Token<K>` built from `(kind, span)` reads those exact two values back through `kind()` and `span()`; construction loses nothing.
- A token and `span-lang`'s `Spanned<K>` hold the same two fields, and the `From` conversions between them round-trip in both directions.
- Tokens order by span first, then kind, so sorting a stream sorts it into source order; the `Token` ordering equals the `(span, kind)` tuple ordering for every pair.
- `map` rewrites the kind and leaves the span untouched.
- The `Token` classification methods (`is_trivia`, `is_eof`, `symbol`) return exactly what the underlying kind returns — they are forwarders, never a second source of truth.
- The surface is total and panic-free: no constructor or accessor can fail, and the crate adds no allocation to a token beyond what `K` already costs.
