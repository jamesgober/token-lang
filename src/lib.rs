//! # token_lang
//!
//! The token type that sits between a lexer and a parser.
//!
//! token-lang defines one thing: a [`Token<K>`] — a language-specific *kind*
//! paired with the [`Span`] of source it covers — and the [`TokenKind`] trait that
//! lets generic, language-agnostic code reason about a token stream. A lexer
//! produces `Token<K>`; a parser consumes it; neither depends on the other,
//! because both agree only on this seam. It owns the token type and nothing else:
//! no scanning, no grammar, no diagnostics.
//!
//! ## The model
//!
//! A token is a *classified span*. The kind `K` says **what** was lexed — it is
//! the language's own `enum` of keywords, punctuation, literals, and an
//! end-of-input marker — and the [`Span`] says **where**. token-lang stays
//! reusable across languages by leaving `K` to the language and owning only the
//! pairing and the small set of questions every parser asks of any token:
//!
//! - [`is_trivia`](TokenKind::is_trivia) — skip whitespace and comments;
//! - [`is_eof`](TokenKind::is_eof) — detect the end of input;
//! - [`symbol`](TokenKind::symbol) — read the interned lexeme an identifier,
//!   keyword, or literal carries, as a cheap [`Symbol`] handle.
//!
//! Every trait method has a default, so a kind that has no trivia and carries no
//! interned text implements [`TokenKind`] with an empty `impl` block.
//!
//! ## Quickstart
//!
//! ```
//! use token_lang::{Span, Token, TokenKind};
//!
//! // A language defines its own kinds; token-lang carries them.
//! #[derive(Clone, Copy, Debug, PartialEq, Eq)]
//! enum Kind {
//!     Ident,
//!     Plus,
//!     Whitespace,
//!     Eof,
//! }
//!
//! impl TokenKind for Kind {
//!     fn is_trivia(&self) -> bool {
//!         matches!(self, Kind::Whitespace)
//!     }
//!     fn is_eof(&self) -> bool {
//!         matches!(self, Kind::Eof)
//!     }
//! }
//!
//! // `a + b`, lexed (including the whitespace) and terminated.
//! let tokens = [
//!     Token::new(Kind::Ident, Span::new(0, 1)),
//!     Token::new(Kind::Whitespace, Span::new(1, 2)),
//!     Token::new(Kind::Plus, Span::new(2, 3)),
//!     Token::new(Kind::Whitespace, Span::new(3, 4)),
//!     Token::new(Kind::Ident, Span::new(4, 5)),
//!     Token::new(Kind::Eof, Span::empty(5)),
//! ];
//!
//! // A parser keeps the significant tokens — without knowing the language.
//! let significant: usize = tokens
//!     .iter()
//!     .filter(|t| !t.is_trivia() && !t.is_eof())
//!     .count();
//! assert_eq!(significant, 3); // Ident, Plus, Ident
//! assert!(tokens.last().unwrap().is_eof());
//! ```
//!
//! ## `no_std`
//!
//! The crate is `no_std`-compatible and allocation-free: the token type and the
//! trait need neither the standard library nor `alloc`. The default `std` feature
//! is additive and only forwards to the standard-library builds of the coordinate
//! and interning crates. Disable default features to build for a `no_std` target.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unused_must_use)]
#![deny(unused_results)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::unreachable)]

mod kind;
mod token;

pub use kind::TokenKind;
pub use token::Token;

/// A four-byte `Copy` handle to an interned string, re-exported from
/// [`intern_lang`] so consumers can name the type [`TokenKind::symbol`] returns
/// without depending on `intern-lang` directly.
pub use intern_lang::Symbol;
/// The half-open byte range a token covers, re-exported from [`span_lang`] so
/// consumers can name the type [`Token`] is built from without depending on
/// `span-lang` directly.
pub use span_lang::Span;
/// A value paired with its [`Span`], re-exported from [`span_lang`]. [`Token`]
/// converts to and from `Spanned` through its [`From`] impls.
pub use span_lang::Spanned;
