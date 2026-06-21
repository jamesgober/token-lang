//! The [`Token`] type: a syntactic kind paired with the span it covers.

use core::fmt;

use intern_lang::Symbol;
use span_lang::{Span, Spanned};

use crate::TokenKind;

/// A single lexical token: a [`kind`](Token::kind) paired with the [`Span`] of
/// source it covers.
///
/// `Token<K>` is the shared seam between a lexer and a parser. The lexer produces
/// a stream of `Token<K>`; the parser consumes it. Neither needs to know the
/// other's internals — they agree only on this pair. The *kind* `K` is the
/// language's own classification (an `enum` of keywords, punctuation, literals,
/// and so on); token-lang stays language-agnostic by leaving `K` to the language
/// and owning only the pairing with a span.
///
/// A token is a *classified span*: `K` says **what** was lexed, the [`Span`] says
/// **where**. The type is `Copy` whenever `K` is, so a token whose kind is a plain
/// enum (a discriminant plus an eight-byte span) is cheap to pass by value.
///
/// `Token<K>` mirrors [`Spanned<K>`](span_lang::Spanned) but carries token
/// semantics: it orders by span first, so a slice of tokens sorts into source
/// order; its `Display` reads as `kind @ start..end`; and when `K: TokenKind` it
/// forwards the classification queries — [`is_trivia`](Token::is_trivia),
/// [`is_eof`](Token::is_eof), and [`symbol`](Token::symbol) — to the kind. Convert
/// between the two with the [`From`] impls when a layer wants the plain pair.
///
/// # Examples
///
/// ```
/// use token_lang::{Span, Token, TokenKind};
///
/// #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// enum Kind {
///     Ident,
///     Plus,
///     Eof,
/// }
/// impl TokenKind for Kind {
///     fn is_eof(&self) -> bool {
///         matches!(self, Kind::Eof)
///     }
/// }
///
/// // `a + b` lexed into tokens, then the end marker.
/// let a = Token::new(Kind::Ident, Span::new(0, 1));
/// let plus = Token::new(Kind::Plus, Span::new(2, 3));
/// let eof = Token::new(Kind::Eof, Span::new(5, 5));
///
/// assert_eq!(*a.kind(), Kind::Ident);
/// assert_eq!(plus.span(), Span::new(2, 3));
/// assert!(eof.is_eof());
///
/// // Tokens are `Copy` and sort into source order.
/// let mut stream = [eof, a, plus];
/// stream.sort();
/// assert_eq!(stream, [a, plus, eof]);
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token<K> {
    /// The half-open source range this token covers.
    ///
    /// Declared before [`kind`](Token::kind) so the derived ordering compares the
    /// span first, sorting a slice of tokens into source order.
    pub span: Span,
    /// The language-specific kind of this token.
    pub kind: K,
}

impl<K> Token<K> {
    /// Pairs a kind with the span it was lexed from.
    ///
    /// `const`, so a token can initialise a `const` or `static` table — useful for
    /// the fixed marker tokens (an end-of-input, a synthetic delimiter) a lexer
    /// hands out.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_lang::{Span, Token};
    ///
    /// let tok = Token::new("ident", Span::new(0, 5));
    /// assert_eq!(*tok.kind(), "ident");
    /// assert_eq!(tok.span(), Span::new(0, 5));
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(kind: K, span: Span) -> Self {
        Self { span, kind }
    }

    /// Borrows this token's kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_lang::{Span, Token};
    ///
    /// let tok = Token::new(42u8, Span::new(0, 1));
    /// assert_eq!(*tok.kind(), 42);
    /// ```
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> &K {
        &self.kind
    }

    /// Returns this token's span.
    ///
    /// `Span` is `Copy`, so this hands back the range by value rather than
    /// borrowing it.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_lang::{Span, Token};
    ///
    /// let tok = Token::new((), Span::new(3, 7));
    /// assert_eq!(tok.span(), Span::new(3, 7));
    /// ```
    #[inline]
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Consumes the token, returning just its kind and dropping the span.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_lang::{Span, Token};
    ///
    /// let tok = Token::new(String::from("if"), Span::new(0, 2));
    /// assert_eq!(tok.into_kind(), "if");
    /// ```
    #[inline]
    #[must_use]
    pub fn into_kind(self) -> K {
        self.kind
    }

    /// Transforms the kind with `f`, keeping the span unchanged.
    ///
    /// This is how a layer lifts a token from one kind to another without losing
    /// where it came from — for example, mapping a raw lexer kind onto a coarser
    /// parser kind, or wrapping the kind in a richer type.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_lang::{Span, Token};
    ///
    /// let raw = Token::new("123", Span::new(4, 7));
    /// let parsed = raw.map(|s| s.parse::<u32>().unwrap());
    /// assert_eq!(*parsed.kind(), 123);
    /// assert_eq!(parsed.span(), Span::new(4, 7));
    /// ```
    #[inline]
    #[must_use]
    pub fn map<U>(self, f: impl FnOnce(K) -> U) -> Token<U> {
        Token {
            span: self.span,
            kind: f(self.kind),
        }
    }

    /// Borrows the kind, yielding a `Token<&K>` with the same span.
    ///
    /// Mirrors [`Option::as_ref`]: it lets you inspect or [`map`](Token::map) the
    /// kind without consuming the original token.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_lang::{Span, Token};
    ///
    /// let owned = Token::new(String::from("name"), Span::new(0, 4));
    /// let len = owned.as_ref().map(|s| s.len());
    /// assert_eq!(*len.kind(), 4);
    /// // `owned` is still usable.
    /// assert_eq!(owned.kind(), "name");
    /// ```
    #[inline]
    #[must_use]
    pub fn as_ref(&self) -> Token<&K> {
        Token {
            span: self.span,
            kind: &self.kind,
        }
    }
}

impl<K: TokenKind> Token<K> {
    /// Whether this token's kind is trivia. Forwards to
    /// [`TokenKind::is_trivia`].
    ///
    /// # Examples
    ///
    /// ```
    /// use token_lang::{Span, Token, TokenKind};
    ///
    /// #[derive(Clone, Copy)]
    /// enum Kind {
    ///     Word,
    ///     Space,
    /// }
    /// impl TokenKind for Kind {
    ///     fn is_trivia(&self) -> bool {
    ///         matches!(self, Kind::Space)
    ///     }
    /// }
    ///
    /// assert!(Token::new(Kind::Space, Span::new(0, 1)).is_trivia());
    /// assert!(!Token::new(Kind::Word, Span::new(1, 5)).is_trivia());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_trivia(&self) -> bool {
        self.kind.is_trivia()
    }

    /// Whether this token's kind is the end-of-input marker. Forwards to
    /// [`TokenKind::is_eof`].
    ///
    /// # Examples
    ///
    /// ```
    /// use token_lang::{Span, Token, TokenKind};
    ///
    /// #[derive(Clone, Copy)]
    /// enum Kind {
    ///     Word,
    ///     Eof,
    /// }
    /// impl TokenKind for Kind {
    ///     fn is_eof(&self) -> bool {
    ///         matches!(self, Kind::Eof)
    ///     }
    /// }
    ///
    /// assert!(Token::new(Kind::Eof, Span::empty(9)).is_eof());
    /// assert!(!Token::new(Kind::Word, Span::new(0, 4)).is_eof());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_eof(&self) -> bool {
        self.kind.is_eof()
    }

    /// The interned lexeme this token carries, if any. Forwards to
    /// [`TokenKind::symbol`].
    ///
    /// # Examples
    ///
    /// ```
    /// use intern_lang::Interner;
    /// use token_lang::{Span, Symbol, Token, TokenKind};
    ///
    /// #[derive(Clone, Copy)]
    /// enum Kind {
    ///     Ident(Symbol),
    ///     Plus,
    /// }
    /// impl TokenKind for Kind {
    ///     fn symbol(&self) -> Option<Symbol> {
    ///         match self {
    ///             Kind::Ident(s) => Some(*s),
    ///             Kind::Plus => None,
    ///         }
    ///     }
    /// }
    ///
    /// let mut interner = Interner::new();
    /// let tok = Token::new(Kind::Ident(interner.intern("x")), Span::new(0, 1));
    /// assert_eq!(tok.symbol().and_then(|s| interner.resolve(s)), Some("x"));
    /// assert_eq!(Token::new(Kind::Plus, Span::new(1, 2)).symbol(), None);
    /// ```
    #[inline]
    #[must_use]
    pub fn symbol(&self) -> Option<Symbol> {
        self.kind.symbol()
    }
}

impl<K: fmt::Display> fmt::Display for Token<K> {
    /// Formats as `kind @ start..end`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.kind, self.span)
    }
}

impl<K> From<(K, Span)> for Token<K> {
    /// Builds a token from a `(kind, span)` pair, matching the
    /// [`new`](Token::new) argument order.
    #[inline]
    fn from((kind, span): (K, Span)) -> Self {
        Self::new(kind, span)
    }
}

impl<K> From<Token<K>> for Spanned<K> {
    /// A token is a spanned kind; this drops the token semantics for the plain
    /// [`Spanned`] pair, preserving both fields.
    #[inline]
    fn from(token: Token<K>) -> Self {
        Spanned::new(token.span, token.kind)
    }
}

impl<K> From<Spanned<K>> for Token<K> {
    /// The inverse of the token-to-[`Spanned`] conversion: reinterprets a spanned
    /// value as a token of that kind, preserving both fields.
    #[inline]
    fn from(spanned: Spanned<K>) -> Self {
        Self::new(spanned.value, spanned.span)
    }
}

#[cfg(test)]
mod tests {
    // Reconstructing a `Symbol` from a known id is `Option`-returning; unwrapping
    // it in a test where the id is a literal is the clearest form.
    #![allow(clippy::unwrap_used)]

    extern crate alloc;
    use alloc::string::{String, ToString};

    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Kind {
        Ident(Symbol),
        Plus,
        Space,
        Eof,
    }

    impl TokenKind for Kind {
        fn is_trivia(&self) -> bool {
            matches!(self, Kind::Space)
        }
        fn is_eof(&self) -> bool {
            matches!(self, Kind::Eof)
        }
        fn symbol(&self) -> Option<Symbol> {
            match self {
                Kind::Ident(s) => Some(*s),
                _ => None,
            }
        }
    }

    #[test]
    fn test_new_stores_kind_and_span() {
        let tok = Token::new(Kind::Plus, Span::new(2, 3));
        assert_eq!(*tok.kind(), Kind::Plus);
        assert_eq!(tok.span(), Span::new(2, 3));
    }

    #[test]
    fn test_map_keeps_span() {
        let tok = Token::new("123", Span::new(4, 7));
        let mapped = tok.map(|s| s.len());
        assert_eq!(*mapped.kind(), 3);
        assert_eq!(mapped.span(), Span::new(4, 7));
    }

    #[test]
    fn test_as_ref_does_not_consume() {
        let owned = Token::new(String::from("name"), Span::new(0, 4));
        let len = owned.as_ref().map(String::len);
        assert_eq!(*len.kind(), 4);
        assert_eq!(owned.kind(), "name");
    }

    #[test]
    fn test_orders_by_span_first() {
        // Spans differ; the smaller span sorts first regardless of kind value.
        let a = Token::new(Kind::Eof, Span::new(0, 1));
        let b = Token::new(Kind::Plus, Span::new(1, 2));
        assert!(a < b);
    }

    #[test]
    fn test_delegators_forward_to_kind() {
        let sym = Symbol::from_u32(3).unwrap();
        assert!(Token::new(Kind::Space, Span::empty(0)).is_trivia());
        assert!(Token::new(Kind::Eof, Span::empty(0)).is_eof());
        assert_eq!(
            Token::new(Kind::Ident(sym), Span::new(0, 1)).symbol(),
            Some(sym)
        );
        assert_eq!(Token::new(Kind::Plus, Span::new(0, 1)).symbol(), None);
    }

    #[test]
    fn test_display_reads_as_kind_at_span() {
        let tok = Token::new("if", Span::new(0, 2));
        assert_eq!(tok.to_string(), "if @ 0..2");
    }

    #[test]
    fn test_roundtrips_through_spanned() {
        let tok = Token::new(Kind::Plus, Span::new(2, 3));
        let spanned: Spanned<Kind> = tok.into();
        assert_eq!(spanned.span, Span::new(2, 3));
        assert_eq!(spanned.value, Kind::Plus);
        assert_eq!(Token::from(spanned), tok);
    }

    #[test]
    fn test_from_tuple_matches_new() {
        let from_tuple: Token<Kind> = (Kind::Plus, Span::new(0, 1)).into();
        assert_eq!(from_tuple, Token::new(Kind::Plus, Span::new(0, 1)));
    }

    #[test]
    fn test_token_is_copy_when_kind_is() {
        let tok = Token::new(Kind::Plus, Span::new(0, 1));
        let copy = tok;
        // Both usable: `tok` was copied, not moved.
        assert_eq!(tok, copy);
    }
}
