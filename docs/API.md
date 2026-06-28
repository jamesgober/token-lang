# token-lang &mdash; API Reference

> Complete reference for every public item in `token-lang`, with examples.
> **Status: stable (1.0).** The surface below is the `1.0` contract; it follows
> [Semantic Versioning](#semver-promise) and will not change in a breaking way before
> `2.0`. See [`dev/ROADMAP.md`](../dev/ROADMAP.md).

## SemVer promise

As of `1.0.0` the public API documented here is **stable**. The crate follows
[Semantic Versioning](https://semver.org):

- No item in this reference will be **removed or changed in a breaking way** within
  the `1.x` series. Breaking changes wait for `2.0`.
- New functionality arrives in **minor** releases (`1.1`, `1.2`, …) and is additive —
  a new defaulted `TokenKind` method, a new `Token` helper, a new trait impl.
- Bug fixes, documentation, and internal changes are **patch** releases.
- The **MSRV** is Rust `1.85`. Raising it is a minor change, called out in the
  changelog; it is never a patch.
- The **`serde` representation** of `Token` is part of the contract: a `Token`
  serialised by one `1.x` reads back in any later `1.x`.

Anything not in this reference — exact `Debug` output, the internal module layout —
is not part of the contract and may change at any time.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Quick start](#quick-start)
- [The model](#the-model)
- [`Token`](#token)
  - [`Token::new`](#tokennew)
  - [`Token::kind`](#tokenkind)
  - [`Token::span`](#tokenspan)
  - [`Token::into_kind`](#tokeninto_kind)
  - [`Token::map`](#tokenmap)
  - [`Token::as_ref`](#tokenas_ref)
  - [`Token::is_trivia` / `is_eof` / `symbol`](#tokenis_trivia--is_eof--symbol)
  - [Conversions](#conversions)
  - [Ordering and traits](#ordering-and-traits)
- [`TokenKind`](#tokenkind-1)
  - [`TokenKind::is_trivia`](#tokenkindis_trivia)
  - [`TokenKind::is_eof`](#tokenkindis_eof)
  - [`TokenKind::symbol`](#tokenkindsymbol)
- [Re-exported types](#re-exported-types)
- [Serialization](#serialization)
- [Feature flags](#feature-flags)

---

## Overview

token-lang defines the token type that sits between a lexer and a parser. A
[`Token<K>`](#token) is a *classified span*: a language-specific *kind* `K` paired
with the [`Span`](#re-exported-types) of source it covers. The
[`TokenKind`](#tokenkind-1) trait gives generic, language-agnostic code the small
set of questions a parser asks of any token — skip trivia, detect the end, read an
interned lexeme.

It is the seam above [`span-lang`](https://docs.rs/span-lang) (the `where` of a
token) and [`intern-lang`](https://docs.rs/intern-lang) (the interned `what`): a
lexer produces `Token<K>`, a parser consumes it, and neither depends on the other.
token-lang owns the token type alone — scanning, grammar, and diagnostics live in
other crates. The language supplies `K`; this crate stays reusable across every
language by never enumerating any one language's keywords.

---

## Installation

```toml
[dependencies]
token-lang = "0.2"
```

Or from the terminal:

```bash
cargo add token-lang
```

The crate is `no_std`-friendly and allocation-free: the token type and the trait
need neither the standard library nor `alloc`. The default `std` feature only
forwards to the standard-library builds of `span-lang` and `intern-lang`; with
`default-features = false` the crate stays `no_std` and the whole surface works
exactly as documented here.

---

## Quick start

```rust
use token_lang::{Span, Token, TokenKind};

// A language defines its own kinds; token-lang carries them.
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

// `a + b`, lexed (whitespace preserved) and terminated.
let tokens = [
    Token::new(Kind::Ident, Span::new(0, 1)),
    Token::new(Kind::Whitespace, Span::new(1, 2)),
    Token::new(Kind::Plus, Span::new(2, 3)),
    Token::new(Kind::Whitespace, Span::new(3, 4)),
    Token::new(Kind::Ident, Span::new(4, 5)),
    Token::new(Kind::Eof, Span::empty(5)),
];

// A parser keeps the significant tokens — without knowing the language.
let significant = tokens
    .iter()
    .filter(|t| !t.is_trivia() && !t.is_eof())
    .count();
assert_eq!(significant, 3);
```

---

## The model

A token is `kind + span`. The kind `K` is the language's own classification — an
`enum` of keywords, punctuation, literals, and an end-of-input marker — and the
span says where in the source it was lexed. token-lang owns only the pairing and
the [`TokenKind`](#tokenkind-1) trait, so:

- the same `Token<K>` type serves every language, parameterized by that language's
  `K`;
- generic tooling (a parser's token cursor, a trivia filter, a pretty-printer)
  written against `TokenKind` works for any `K`;
- a token costs exactly `K` plus an eight-byte span, and is `Copy` whenever `K` is.

The kind carries any payload a token needs: an identifier or literal kind holds a
[`Symbol`](#re-exported-types) (a four-byte interned handle from `intern-lang`),
which [`symbol`](#tokenkindsymbol) hands back to generic code.

---

## `Token`

```rust
pub struct Token<K> {
    pub span: Span,
    pub kind: K,
}
```

A single lexical token. The fields are public — a token is a transparent pair — but
the accessors below read more clearly and are `const`. `span` is declared first so
the derived ordering compares it first, sorting a slice of tokens into source
order.

```rust
use token_lang::{Span, Token};

let tok = Token::new("ident", Span::new(0, 5));
assert_eq!(*tok.kind(), "ident");
assert_eq!(tok.span(), Span::new(0, 5));
```

### `Token::new`

```rust
pub const fn new(kind: K, span: Span) -> Token<K>
```

Pairs a kind with the span it was lexed from. `const`, so it can initialise a
`const` or `static` table of marker tokens.

```rust
use token_lang::{Span, Token};

const EOF: Token<u8> = Token::new(0, Span::empty(0));
assert_eq!(*EOF.kind(), 0);
```

### `Token::kind`

```rust
pub const fn kind(&self) -> &K
```

Borrows the token's kind.

### `Token::span`

```rust
pub const fn span(&self) -> Span
```

Returns the token's span. `Span` is `Copy`, so this hands the range back by value.

### `Token::into_kind`

```rust
pub fn into_kind(self) -> K
```

Consumes the token, returning just its kind and dropping the span.

```rust
use token_lang::{Span, Token};

let tok = Token::new(String::from("if"), Span::new(0, 2));
assert_eq!(tok.into_kind(), "if");
```

### `Token::map`

```rust
pub fn map<U>(self, f: impl FnOnce(K) -> U) -> Token<U>
```

Transforms the kind with `f`, keeping the span unchanged. This is how a pass lifts
a token from one kind to another — mapping a raw lexer kind onto a coarser parser
kind, or wrapping the kind in a richer type — without losing where it came from.

```rust
use token_lang::{Span, Token};

let raw = Token::new("123", Span::new(4, 7));
let parsed = raw.map(|s| s.parse::<u32>().unwrap());
assert_eq!(*parsed.kind(), 123);
assert_eq!(parsed.span(), Span::new(4, 7)); // span preserved
```

### `Token::as_ref`

```rust
pub fn as_ref(&self) -> Token<&K>
```

Borrows the kind, yielding a `Token<&K>` with the same span — like
[`Option::as_ref`], so you can inspect or `map` the kind without consuming the
token.

```rust
use token_lang::{Span, Token};

let owned = Token::new(String::from("name"), Span::new(0, 4));
let len = owned.as_ref().map(|s| s.len());
assert_eq!(*len.kind(), 4);
assert_eq!(owned.kind(), "name"); // still usable
```

### `Token::is_trivia` / `is_eof` / `symbol`

```rust
pub fn is_trivia(&self) -> bool   // where K: TokenKind
pub fn is_eof(&self) -> bool      // where K: TokenKind
pub fn symbol(&self) -> Option<Symbol> // where K: TokenKind
```

When `K: TokenKind`, a token forwards the three classification queries to its kind,
so a parser can ask them of the token directly. Each returns exactly what the
matching [`TokenKind`](#tokenkind-1) method returns.

```rust
use intern_lang::Interner;
use token_lang::{Span, Symbol, Token, TokenKind};

#[derive(Clone, Copy)]
enum Kind {
    Ident(Symbol),
    Space,
    Eof,
}
impl TokenKind for Kind {
    fn is_trivia(&self) -> bool { matches!(self, Kind::Space) }
    fn is_eof(&self) -> bool { matches!(self, Kind::Eof) }
    fn symbol(&self) -> Option<Symbol> {
        match self { Kind::Ident(s) => Some(*s), _ => None }
    }
}

let mut interner = Interner::new();
let id = Token::new(Kind::Ident(interner.intern("x")), Span::new(0, 1));
assert!(Token::new(Kind::Space, Span::new(1, 2)).is_trivia());
assert!(Token::new(Kind::Eof, Span::empty(2)).is_eof());
assert_eq!(id.symbol().and_then(|s| interner.resolve(s)), Some("x"));
```

### Conversions

A token and `span-lang`'s [`Spanned<K>`](#re-exported-types) carry the same two
fields, so they convert both ways without loss; a `(K, Span)` tuple converts in too,
matching the [`new`](#tokennew) argument order.

```rust
use token_lang::{Span, Spanned, Token};

let tok = Token::new(7u8, Span::new(2, 3));

// Token <-> Spanned round-trips.
let spanned: Spanned<u8> = tok.into();
assert_eq!(spanned, Spanned::new(Span::new(2, 3), 7));
assert_eq!(Token::from(spanned), tok);

// From a (kind, span) pair.
let from_tuple: Token<u8> = (7u8, Span::new(2, 3)).into();
assert_eq!(from_tuple, tok);
```

### Ordering and traits

`Token<K>` derives `Clone`, `Copy` (when `K: Copy`), `Debug`, `PartialEq`, `Eq`,
`PartialOrd`, `Ord`, and `Hash`, and implements `Display` (as `kind @ start..end`)
when `K: Display`. Ordering compares the span first, then the kind, so sorting a
`Vec<Token<K>>` orders it by source position.

```rust
use token_lang::{Span, Token};

let mut stream = [
    Token::new("c", Span::new(4, 5)),
    Token::new("a", Span::new(0, 1)),
    Token::new("b", Span::new(2, 3)),
];
stream.sort();
let order: Vec<_> = stream.iter().map(|t| *t.kind()).collect();
assert_eq!(order, ["a", "b", "c"]);
assert_eq!(stream[0].to_string(), "a @ 0..1");
```

---

## `TokenKind`

```rust
pub trait TokenKind {
    fn is_trivia(&self) -> bool { false }
    fn is_eof(&self) -> bool { false }
    fn symbol(&self) -> Option<Symbol> { None }
}
```

The classification a token kind exposes so generic code can reason about a stream
without knowing the concrete kind. A language implements it for its own kind type.
Every method has a default, so a kind with no trivia, no end marker, and no interned
text satisfies the trait with an empty `impl`:

```rust
use token_lang::TokenKind;

#[derive(Clone, Copy)]
enum Op { Add, Sub }
impl TokenKind for Op {}

assert!(!Op::Add.is_trivia());
assert!(!Op::Add.is_eof());
assert_eq!(Op::Add.symbol(), None);
```

### `TokenKind::is_trivia`

```rust
fn is_trivia(&self) -> bool
```

Whether this kind is trivia — whitespace, comments, anything a parser skips because
it is not part of the grammar. A lexer that preserves trivia (for a formatter or a
lossless tree) emits these inline; a parser filters them with this query. Defaults
to `false`.

### `TokenKind::is_eof`

```rust
fn is_eof(&self) -> bool
```

Whether this kind is the end-of-input marker the lexer emits once the source is
exhausted. A parser uses it to detect the end without tracking a separate length.
Defaults to `false`.

### `TokenKind::symbol`

```rust
fn symbol(&self) -> Option<Symbol>
```

The interned lexeme this token carries, if any — the [`Symbol`](#re-exported-types)
for an identifier, keyword, or interned literal, and `None` for a kind with no
textual payload (most punctuation, structural tokens, the end marker). It lets
generic code read a token's name without matching the concrete kind. Defaults to
`None`.

---

## Re-exported types

token-lang re-exports the dependency types its surface speaks, so a consumer need
not also name `span-lang` or `intern-lang`:

```rust
pub use span_lang::{Span, Spanned};
pub use intern_lang::Symbol;
```

- **`Span`** — a half-open `start..end` byte range, the `where` of a token.
  `Span::new(start, end)`, `Span::empty(at)`, `.start()`, `.end()`, `.len()`,
  `.contains(pos)`. See the [`span-lang` docs](https://docs.rs/span-lang).
- **`Spanned<T>`** — a value paired with its `Span`; [`Token`](#token) converts to
  and from it.
- **`Symbol`** — a four-byte `Copy` interned-string handle, the type
  [`symbol`](#tokenkindsymbol) returns. Resolve it against the
  [`Interner`](https://docs.rs/intern-lang) that issued it. See the
  [`intern-lang` docs](https://docs.rs/intern-lang).

---

## Serialization

With the `serde` feature, [`Token<K>`](#token) derives `Serialize` and
`Deserialize` (for any `K` that does), and the re-exported `Span` and `Symbol`
carry their own serde implementations behind the same feature. A token serialises
as its two fields and round-trips through any serde format.

```rust,ignore
use token_lang::{Span, Token};

let tok = Token::new(7u8, Span::new(2, 5));
let json = serde_json::to_string(&tok)?;
let back: Token<u8> = serde_json::from_str(&json)?;
assert_eq!(tok, back);
# Ok::<(), serde_json::Error>(())
```

A `Symbol` carried in a kind serialises as its raw id, which is only meaningful with
the interner that issued it — persist tokens alongside that interner.

---

## Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | yes     | Forwards to `span-lang/std` and `intern-lang/std`. The crate needs no `std` itself; this only affects the dependency builds. |
| `serde` | no      | Derives `Serialize`/`Deserialize` for `Token`, and forwards to `span-lang/serde` and `intern-lang/serde` for the re-exported types. |

Disabling `std` keeps the crate `no_std` (it needs no `alloc` either):

```toml
[dependencies]
token-lang = { version = "0.2", default-features = false }
```

---

<sub>Copyright &copy; 2026 <strong>James Gober</strong>.</sub>
