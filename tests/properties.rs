//! Property tests for the token invariants documented in `dev/DIRECTIVES.md`,
//! checked across randomized kinds and spans.

use std::cmp::Ordering;

use proptest::prelude::*;
use token_lang::{Span, Spanned, Symbol, Token, TokenKind};

/// A `Span` whose `start <= end` invariant always holds.
fn any_span() -> impl Strategy<Value = Span> {
    (any::<u32>(), any::<u32>()).prop_map(|(a, b)| {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        Span::new(lo, hi)
    })
}

/// A kind with every classification axis turned into data, so a generated value
/// can independently be trivia or not, eof or not, and carry a symbol or not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Classified {
    trivia: bool,
    eof: bool,
    sym: Option<u32>,
}

impl TokenKind for Classified {
    fn is_trivia(&self) -> bool {
        self.trivia
    }
    fn is_eof(&self) -> bool {
        self.eof
    }
    fn symbol(&self) -> Option<Symbol> {
        self.sym.and_then(Symbol::from_u32)
    }
}

fn any_classified() -> impl Strategy<Value = Classified> {
    (
        any::<bool>(),
        any::<bool>(),
        proptest::option::of(1..=u32::MAX),
    )
        .prop_map(|(trivia, eof, sym)| Classified { trivia, eof, sym })
}

proptest! {
    /// Construction preserves both halves: the kind and span read back exactly as
    /// they went in.
    #[test]
    fn new_preserves_kind_and_span(kind in any::<i64>(), span in any_span()) {
        let tok = Token::new(kind, span);
        prop_assert_eq!(*tok.kind(), kind);
        prop_assert_eq!(tok.span(), span);
    }

    /// A token and the plain `Spanned` pair carry the same two fields, so the
    /// conversion round-trips in both directions without loss.
    #[test]
    fn spanned_round_trip(kind in any::<i64>(), span in any_span()) {
        let tok = Token::new(kind, span);
        let spanned: Spanned<i64> = tok.into();
        prop_assert_eq!(spanned, Spanned::new(span, kind));
        prop_assert_eq!(Token::from(spanned), tok);
    }

    /// `map` applies the function to the kind and leaves the span untouched.
    #[test]
    fn map_transforms_kind_keeps_span(kind in any::<i32>(), span in any_span()) {
        let mapped = Token::new(kind, span).map(|k| i64::from(k) * 2);
        prop_assert_eq!(*mapped.kind(), i64::from(kind) * 2);
        prop_assert_eq!(mapped.span(), span);
    }

    /// Tokens order by span first, then kind — the contract that sorts a stream
    /// into source order. The token ordering must match the `(span, kind)` tuple
    /// ordering for every pair.
    #[test]
    fn orders_by_span_then_kind(
        ka in any::<i64>(), sa in any_span(),
        kb in any::<i64>(), sb in any_span(),
    ) {
        let a = Token::new(ka, sa);
        let b = Token::new(kb, sb);
        prop_assert_eq!(a.cmp(&b), (sa, ka).cmp(&(sb, kb)));
    }

    /// Sorting any stream of tokens yields non-decreasing spans (source order).
    #[test]
    fn sorting_yields_non_decreasing_spans(
        items in proptest::collection::vec((any::<i64>(), any_span()), 0..64),
    ) {
        let mut tokens: Vec<Token<i64>> =
            items.into_iter().map(|(k, s)| Token::new(k, s)).collect();
        tokens.sort();
        for pair in tokens.windows(2) {
            prop_assert!(matches!(
                pair[0].span().start().cmp(&pair[1].span().start()),
                Ordering::Less | Ordering::Equal
            ));
        }
    }

    /// The `Token` delegators return exactly what the underlying kind returns.
    #[test]
    fn delegators_match_kind(kind in any_classified(), span in any_span()) {
        let tok = Token::new(kind, span);
        prop_assert_eq!(tok.is_trivia(), kind.is_trivia());
        prop_assert_eq!(tok.is_eof(), kind.is_eof());
        prop_assert_eq!(tok.symbol(), kind.symbol());
    }

    /// `Display` is exactly `"{kind} @ {span}"`.
    #[test]
    fn display_is_kind_at_span(kind in any::<i64>(), span in any_span()) {
        let tok = Token::new(kind, span);
        prop_assert_eq!(tok.to_string(), format!("{} @ {}", kind, span));
    }
}
