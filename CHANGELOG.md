<h1 align="center">
    <img width="90px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <br><b>CHANGELOG</b>
</h1>
<p>
  All notable changes to <code>token-lang</code> will be documented in this file. The format is based on <a href="https://keepachangelog.com/en/1.1.0/">Keep a Changelog</a>,
  and this project adheres to <a href="https://semver.org/spec/v2.0.0.html/">Semantic Versioning</a>.
</p>

---

## [Unreleased]

## [1.0.0] - 2026-06-28

API freeze. `1.0.0` declares the public surface stable under Semantic Versioning
until `2.0`. There are no code changes from `0.2.0` — this release marks the
contract: `Token<K>`, the `TokenKind` trait, and the re-exported coordinate and
interning types are now stable, and no breaking change will be made without a major
version bump.

### Changed

- `docs/API.md` and the crate docs now document the surface as the stable `1.0`
  contract, with the SemVer promise recorded.

## [0.2.0] - 2026-06-20

The core: the token type and the lexer↔parser seam. Wires `span-lang` and
`intern-lang`, the first phase with domain logic.

### Added

- `Token<K>` — a language-specific kind paired with the `Span` of source it covers.
  Construction (`new`), accessors (`kind`, `span`, `into_kind`), `map` for rewriting
  the kind while keeping the span, and `as_ref`. `Copy` when `K` is; orders by span
  first so a stream sorts into source order; `Display` as `kind @ start..end`.
- `TokenKind` — the classification trait a language implements for its kind:
  `is_trivia`, `is_eof`, and `symbol` (returning the interned lexeme as a `Symbol`),
  all default-implemented so an empty `impl` works. `Token` forwards all three when
  `K: TokenKind`.
- `From` conversions between `Token<K>`, `span-lang`'s `Spanned<K>`, and `(K, Span)`.
- Re-exports of `Span`, `Spanned` (`span-lang`) and `Symbol` (`intern-lang`).
- `serde` feature: derives `Serialize`/`Deserialize` for `Token`, forwarded to the
  dependency types.
- Property tests for the documented invariants, integration and serde tests, and
  Criterion benchmarks for the parser-side hot paths.
- `dev/DIRECTIVES.md` and `docs/API.md`.

### Changed

- Aligned `clippy.toml` MSRV to Rust 1.85 (matching `Cargo.toml` and the sibling
  crates).

### Fixed

- `Cargo.toml` `keywords` and `categories` were unquoted bare words — invalid TOML
  that prevented the crate from building; they are now quoted strings.

---

## [0.1.0] - 2026-06-18

Initial scaffold and repository bootstrap. No domain logic yet &mdash; this release establishes the structure, tooling, and quality gates the implementation will be built on.

### Added

- `Cargo.toml` with crate metadata, Rust 2024 edition, MSRV 1.85.
- Dual `Apache-2.0 OR MIT` license files.
- `README.md`, `CHANGELOG.md`, and a documentation skeleton.
- `REPS.md` compliance baseline.
- `.github/workflows/ci.yml` CI matrix; `deny.toml`, `clippy.toml`, `rustfmt.toml`.
- `dev/DIRECTIVES.md` and `dev/ROADMAP.md` (committed engineering standards + plan).

[Unreleased]: https://github.com/jamesgober/token-lang/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/jamesgober/token-lang/compare/v0.2.0...v1.0.0
[0.2.0]: https://github.com/jamesgober/token-lang/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jamesgober/token-lang/releases/tag/v0.1.0
