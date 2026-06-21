<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br>
    <b>token-lang</b>
    <br>
    <sub><sup>TOKEN DEFINITIONS</sup></sub>
</h1>

<div align="center">
    <a href="https://crates.io/crates/token-lang"><img alt="Crates.io" src="https://img.shields.io/crates/v/token-lang"></a>
    <a href="https://crates.io/crates/token-lang"><img alt="Downloads" src="https://img.shields.io/crates/d/token-lang?color=%230099ff"></a>
    <a href="https://docs.rs/token-lang"><img alt="docs.rs" src="https://img.shields.io/docsrs/token-lang"></a>
    <a href="https://github.com/jamesgober/token-lang/actions"><img alt="CI" src="https://github.com/jamesgober/token-lang/actions/workflows/ci.yml/badge.svg"></a>
    <a href="https://github.com/rust-lang/rfcs/blob/master/text/2495-min-rust-version.md"><img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.85%2B-blue"></a>
</div>

<br>

<div align="left">
    <p>
        token-lang is the BASE-tier crate that follows: Token kinds and the token type only, so lexer and parser stay independent of each other. Part of the -lang language-construction family; see _strategy/LANG_COLLECTION.md for the master plan.
    </p>
    <br>
    <hr>
    <p>
        <strong>MSRV is 1.85+</strong> (Rust 2024 edition).
    </p>
    <blockquote>
        <strong>Status: pre-1.0, in active development.</strong> The public API is being designed across the 0.x series and frozen at <code>1.0.0</code>. See <a href="./CHANGELOG.md"><code>CHANGELOG.md</code></a>.
    </blockquote>
</div>

<hr>
<br>

## Installation

```toml
[dependencies]
token-lang = "0.2"
```

<br>

## Usage

A language defines its own token *kind*; token-lang pairs it with a `Span` and lets
generic code classify the stream.

```rust
use token_lang::{Span, Token, TokenKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Ident,
    Plus,
    Whitespace,
    Eof,
}

impl TokenKind for Kind {
    fn is_trivia(&self) -> bool {
        matches!(self, Kind::Whitespace)
    }
    fn is_eof(&self) -> bool {
        matches!(self, Kind::Eof)
    }
}

let tokens = [
    Token::new(Kind::Ident, Span::new(0, 1)),
    Token::new(Kind::Whitespace, Span::new(1, 2)),
    Token::new(Kind::Plus, Span::new(2, 3)),
    Token::new(Kind::Eof, Span::empty(3)),
];

// A parser skips trivia and stops at the end — without knowing the language.
let significant = tokens.iter().filter(|t| !t.is_trivia() && !t.is_eof()).count();
assert_eq!(significant, 2);
```

See <a href="./docs/API.md"><code>docs/API.md</code></a> for the full surface.

<br>

## Status

<strong>Status: pre-1.0, in active development.</strong> The <code>v0.2.0</code>
release lands the core — the `Token` type and the `TokenKind` seam. The public API
is being finalized across the 0.x series and frozen at <code>1.0.0</code>; see the
<a href="./dev/ROADMAP.md"><code>ROADMAP</code></a> and <a href="./docs/API.md"><code>docs/API.md</code></a>.

<hr>
<br>

## Contributing

See <a href="./dev/DIRECTIVES.md"><code>dev/DIRECTIVES.md</code></a> for engineering standards and the definition of done. Before a PR: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features` must be clean.

<br>

<div id="license">
    <h2>License</h2>
    <p>Licensed under either of</p>
    <ul>
        <li><b>Apache License, Version 2.0</b> &mdash; <a href="./LICENSE-APACHE">LICENSE-APACHE</a></li>
        <li><b>MIT License</b> &mdash; <a href="./LICENSE-MIT">LICENSE-MIT</a></li>
    </ul>
    <p>at your option.</p>
</div>

<div align="center">
  <h2></h2>
  <sup>COPYRIGHT <small>&copy;</small> 2026 <strong>James Gober <me@jamesgober.com>.</strong></sup>
</div>
