//! Criterion benchmarks for the token hot paths.
//!
//! token-lang itself does no scanning, so the paths that matter are the ones a
//! parser runs over a finished stream: skipping trivia and stopping at the end,
//! reading interned lexemes back out, and sorting tokens into source order. Each
//! benchmark builds a representative stream once, then measures the per-pass work.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use intern_lang::{Interner, Symbol};
use token_lang::{Span, Token, TokenKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
            Kind::Ident(s) => Some(*s),
            _ => None,
        }
    }
}

/// A stream of `n` `ident +` pairs with interleaved whitespace, terminated by an
/// end marker — the shape a lexer hands a parser.
fn stream(n: usize) -> Vec<Token<Kind>> {
    let mut interner = Interner::new();
    let mut tokens = Vec::with_capacity(n * 4 + 1);
    let mut at = 0u32;
    for i in 0..n {
        let sym = interner.intern(&format!("ident_{i}"));
        tokens.push(Token::new(Kind::Ident(sym), Span::new(at, at + 5)));
        tokens.push(Token::new(Kind::Whitespace, Span::new(at + 5, at + 6)));
        tokens.push(Token::new(Kind::Plus, Span::new(at + 6, at + 7)));
        tokens.push(Token::new(Kind::Whitespace, Span::new(at + 7, at + 8)));
        at += 8;
    }
    tokens.push(Token::new(Kind::Eof, Span::empty(at)));
    tokens
}

fn bench_filter_trivia(c: &mut Criterion) {
    let tokens = stream(1_024);
    c.bench_function("filter_trivia/1024", |b| {
        b.iter(|| {
            let count = tokens
                .iter()
                .filter(|t| !t.is_trivia() && !t.is_eof())
                .count();
            black_box(count)
        });
    });
}

fn bench_collect_symbols(c: &mut Criterion) {
    let tokens = stream(1_024);
    c.bench_function("collect_symbols/1024", |b| {
        b.iter(|| {
            let mut last = None;
            for t in &tokens {
                if let Some(sym) = t.symbol() {
                    last = Some(black_box(sym));
                }
            }
            black_box(last)
        });
    });
}

fn bench_sort_into_source_order(c: &mut Criterion) {
    let mut shuffled = stream(1_024);
    shuffled.reverse();
    c.bench_function("sort_source_order/1024", |b| {
        b.iter_batched(
            || shuffled.clone(),
            |mut tokens| {
                tokens.sort();
                black_box(tokens)
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_filter_trivia,
    bench_collect_symbols,
    bench_sort_into_source_order
);
criterion_main!(benches);
