//! serde round-trip checks for [`Token`], behind the `serde` feature.

#![cfg(feature = "serde")]

use intern_lang::Interner;
use serde::{Deserialize, Serialize};
use token_lang::{Span, Symbol, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Kind {
    Ident(Symbol),
    Plus,
    Eof,
}

#[test]
fn test_token_round_trips_through_json() {
    let tok = Token::new(Kind::Plus, Span::new(2, 5));
    let json = serde_json::to_string(&tok).unwrap();
    let back: Token<Kind> = serde_json::from_str(&json).unwrap();
    assert_eq!(tok, back);
    // Both fields survive the trip.
    assert_eq!(back.span(), Span::new(2, 5));
    assert_eq!(*back.kind(), Kind::Plus);
}

#[test]
fn test_token_with_symbol_payload_round_trips() {
    let mut interner = Interner::new();
    let tok = Token::new(Kind::Ident(interner.intern("name")), Span::new(0, 4));

    let json = serde_json::to_string(&tok).unwrap();
    let back: Token<Kind> = serde_json::from_str(&json).unwrap();

    assert_eq!(tok, back);
    // The interned id survived, so it still resolves against the same interner.
    let Kind::Ident(sym) = back.into_kind() else {
        panic!("round-tripped an Ident");
    };
    assert_eq!(interner.resolve(sym), Some("name"));
}

#[test]
fn test_token_stream_round_trips_in_order() {
    let tokens = [
        Token::new(Kind::Plus, Span::new(0, 1)),
        Token::new(Kind::Plus, Span::new(1, 2)),
        Token::new(Kind::Eof, Span::empty(2)),
    ];
    let json = serde_json::to_string(&tokens).unwrap();
    let back: Vec<Token<Kind>> = serde_json::from_str(&json).unwrap();
    assert_eq!(back, tokens);
}
