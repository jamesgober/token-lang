//! The [`TokenKind`] trait: the language-agnostic classification a parser reads.

use intern_lang::Symbol;

/// The classification a token kind exposes so that generic, language-agnostic
/// code can reason about a token stream without knowing the concrete kind.
///
/// A language defines its own kind type — an `enum` of keywords, punctuation,
/// literals, an end-of-input marker, and so on — and implements `TokenKind` for
/// it. token-lang stays language-agnostic by owning only this seam: a parser's
/// token cursor, a trivia filter, or a pretty-printer written against `TokenKind`
/// works for *any* language's kind, while [`Token<K>`](crate::Token) carries the
/// span.
///
/// Every method has a default, so a kind that has no trivia, no end marker, and
/// carries no interned text satisfies the trait with an empty `impl` block.
/// Override only the queries that apply to the language.
///
/// # Why a trait, not a fixed enum
///
/// The set of keywords and operators differs in every language, so token-lang
/// cannot enumerate them once and freeze them. What *is* universal is the handful
/// of questions a parser asks of any token regardless of language: should I skip
/// this as trivia, have I reached the end, and what interned text (if any) does it
/// carry. Those three questions are the whole trait.
///
/// # Examples
///
/// A small kind for a calculator language. Whitespace is trivia; identifiers carry
/// an interned [`Symbol`]; the rest are plain markers.
///
/// ```
/// use intern_lang::Interner;
/// use token_lang::{Symbol, TokenKind};
///
/// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// enum Kind {
///     Ident(Symbol),
///     Plus,
///     Whitespace,
///     Eof,
/// }
///
/// impl TokenKind for Kind {
///     fn is_trivia(&self) -> bool {
///         matches!(self, Kind::Whitespace)
///     }
///     fn is_eof(&self) -> bool {
///         matches!(self, Kind::Eof)
///     }
///     fn symbol(&self) -> Option<Symbol> {
///         match self {
///             Kind::Ident(sym) => Some(*sym),
///             _ => None,
///         }
///     }
/// }
///
/// let mut interner = Interner::new();
/// let total = Kind::Ident(interner.intern("total"));
///
/// assert!(Kind::Whitespace.is_trivia());
/// assert!(Kind::Eof.is_eof());
/// assert_eq!(total.symbol().and_then(|s| interner.resolve(s)), Some("total"));
/// assert_eq!(Kind::Plus.symbol(), None);
/// ```
///
/// A kind with no trivia or interned text needs no method bodies at all:
///
/// ```
/// use token_lang::TokenKind;
///
/// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// enum Op {
///     Add,
///     Sub,
/// }
///
/// impl TokenKind for Op {}
///
/// assert!(!Op::Add.is_trivia());
/// assert!(!Op::Add.is_eof());
/// assert_eq!(Op::Add.symbol(), None);
/// ```
pub trait TokenKind {
    /// Whether this kind is *trivia*: whitespace, comments, and anything else a
    /// parser skips because it is not part of the grammar.
    ///
    /// A lexer that preserves trivia (for a formatter or a lossless syntax tree)
    /// emits these kinds inline; a parser filters them out with this query.
    /// Defaults to `false`, so a language that discards trivia in the lexer need
    /// not override it.
    #[inline]
    fn is_trivia(&self) -> bool {
        false
    }

    /// Whether this kind is the end-of-input marker the lexer emits once the
    /// source is exhausted.
    ///
    /// A parser uses this to detect the end of the stream without tracking a
    /// separate length, and to stop before reading past it. Defaults to `false`.
    #[inline]
    fn is_eof(&self) -> bool {
        false
    }

    /// The interned lexeme this token carries, if any.
    ///
    /// Returns the [`Symbol`] for a kind whose text was interned — an identifier,
    /// a keyword, or an interned literal — and `None` for a kind with no textual
    /// payload, which is most punctuation, structural tokens, and the end marker.
    /// It lets generic code read a token's name (to build a path, look up a
    /// keyword, or render the source back) without matching on the concrete kind.
    ///
    /// Defaults to `None`, so a language that stores no interned text — or none on
    /// this kind — need not override it.
    #[inline]
    fn symbol(&self) -> Option<Symbol> {
        None
    }
}

#[cfg(test)]
mod tests {
    // Reconstructing a `Symbol` from a known id is `Option`-returning; unwrapping
    // it in a test where the id is a literal is the clearest form.
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Kind {
        Ident(Symbol),
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
        fn symbol(&self) -> Option<Symbol> {
            match self {
                Kind::Ident(sym) => Some(*sym),
                _ => None,
            }
        }
    }

    // A kind that overrides nothing exercises every default method.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Bare;
    impl TokenKind for Bare {}

    #[test]
    fn test_is_trivia_true_only_for_whitespace() {
        assert!(Kind::Whitespace.is_trivia());
        assert!(!Kind::Plus.is_trivia());
        assert!(!Kind::Eof.is_trivia());
    }

    #[test]
    fn test_is_eof_true_only_for_eof() {
        assert!(Kind::Eof.is_eof());
        assert!(!Kind::Plus.is_eof());
        assert!(!Kind::Whitespace.is_eof());
    }

    #[test]
    fn test_symbol_present_only_for_ident() {
        let sym = Symbol::from_u32(7).unwrap();
        assert_eq!(Kind::Ident(sym).symbol(), Some(sym));
        assert_eq!(Kind::Plus.symbol(), None);
    }

    #[test]
    fn test_defaults_are_all_false_and_none() {
        assert!(!Bare.is_trivia());
        assert!(!Bare.is_eof());
        assert_eq!(Bare.symbol(), None);
    }
}
