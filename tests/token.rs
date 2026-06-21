//! Integration tests for the public token surface, exercised the way a lexer and
//! a parser use it: build a token stream, then classify and navigate it through
//! the crate's public API only.

use intern_lang::Interner;
use token_lang::{Span, Spanned, Symbol, Token, TokenKind};

/// A small but representative language kind: identifiers and string literals carry
/// interned text, whitespace and comments are trivia, the rest are markers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Ident(Symbol),
    Str(Symbol),
    Plus,
    Whitespace,
    LineComment,
    Eof,
}

impl TokenKind for Kind {
    fn is_trivia(&self) -> bool {
        matches!(self, Kind::Whitespace | Kind::LineComment)
    }
    fn is_eof(&self) -> bool {
        matches!(self, Kind::Eof)
    }
    fn symbol(&self) -> Option<Symbol> {
        match self {
            Kind::Ident(s) | Kind::Str(s) => Some(*s),
            _ => None,
        }
    }
}

/// Lex `foo + "bar"` with a trailing comment into a token stream, mirroring what a
/// real lexer would emit (trivia preserved, terminated by EOF).
fn lex(interner: &mut Interner) -> Vec<Token<Kind>> {
    let ident = interner.intern("foo");
    let literal = interner.intern("bar");
    vec![
        Token::new(Kind::Ident(ident), Span::new(0, 3)),
        Token::new(Kind::Whitespace, Span::new(3, 4)),
        Token::new(Kind::Plus, Span::new(4, 5)),
        Token::new(Kind::Whitespace, Span::new(5, 6)),
        Token::new(Kind::Str(literal), Span::new(6, 11)),
        Token::new(Kind::LineComment, Span::new(11, 17)),
        Token::new(Kind::Eof, Span::empty(17)),
    ]
}

#[test]
fn test_parser_skips_trivia_and_stops_at_eof() {
    let mut interner = Interner::new();
    let tokens = lex(&mut interner);

    let significant: Vec<&Token<Kind>> = tokens
        .iter()
        .filter(|t| !t.is_trivia() && !t.is_eof())
        .collect();

    // foo, +, "bar" — whitespace, the comment, and EOF are filtered out.
    assert_eq!(significant.len(), 3);
    assert_eq!(*significant[0].kind(), Kind::Ident(interner.intern("foo")));
    assert_eq!(*significant[1].kind(), Kind::Plus);
    assert!(tokens.last().unwrap().is_eof());
}

#[test]
fn test_symbols_resolve_back_to_source_text() {
    let mut interner = Interner::new();
    let tokens = lex(&mut interner);

    let names: Vec<&str> = tokens
        .iter()
        .filter_map(|t| t.symbol())
        .map(|sym| interner.resolve(sym).unwrap())
        .collect();

    assert_eq!(names, ["foo", "bar"]);
}

#[test]
fn test_tokens_sort_into_source_order() {
    let mut interner = Interner::new();
    let mut tokens = lex(&mut interner);
    tokens.reverse();
    tokens.sort();

    // After sorting, spans are non-decreasing across the whole stream.
    for pair in tokens.windows(2) {
        assert!(pair[0].span().start() <= pair[1].span().start());
    }
    assert!(tokens.last().unwrap().is_eof());
}

#[test]
fn test_span_covers_expected_range() {
    let mut interner = Interner::new();
    let tokens = lex(&mut interner);

    // The string literal "bar" occupies bytes 6..11.
    let lit = tokens
        .iter()
        .find(|t| t.symbol().is_some() && t.span().start().to_u32() == 6);
    let lit = lit.expect("string literal present");
    assert_eq!(lit.span(), Span::new(6, 11));
    assert_eq!(lit.span().len(), 5);
}

#[test]
fn test_map_rewrites_kind_keeping_span() {
    // A pass that erases interned payloads but keeps positions, as a desugaring
    // step might: map each kind to a tag, preserving every span.
    let mut interner = Interner::new();
    let tokens = lex(&mut interner);

    let tagged: Vec<Token<&'static str>> = tokens
        .into_iter()
        .map(|t| {
            let span = t.span();
            let tag = match t.into_kind() {
                Kind::Ident(_) => "ident",
                Kind::Str(_) => "str",
                Kind::Plus => "plus",
                Kind::Whitespace => "ws",
                Kind::LineComment => "comment",
                Kind::Eof => "eof",
            };
            Token::new(tag, span)
        })
        .collect();

    assert_eq!(*tagged[0].kind(), "ident");
    assert_eq!(tagged[0].span(), Span::new(0, 3));
    assert_eq!(*tagged.last().unwrap().kind(), "eof");
}

#[test]
fn test_spanned_conversions_round_trip() {
    let tok = Token::new(Kind::Plus, Span::new(4, 5));
    let spanned: Spanned<Kind> = tok.into();
    assert_eq!(spanned, Spanned::new(Span::new(4, 5), Kind::Plus));
    assert_eq!(Token::from(spanned), tok);
}

#[test]
fn test_display_renders_kind_at_span() {
    let tok = Token::new("return", Span::new(10, 16));
    assert_eq!(tok.to_string(), "return @ 10..16");
}
