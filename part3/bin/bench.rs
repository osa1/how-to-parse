#![allow(unused)]

use parsing_post as lib;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

const SIZES: [usize; 1] = [
    // 10,
    // 100,
    // 1_000,
    // 10_000,
    // 100_000,
    // 1_000_000,
    10_000_000,
    // 100_000_000,
    // 1_000_000_000,
];

fn tokenize_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tokenize [iter] Vec");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                lib::tokenize_iter(input)
                    .map(|ev| ev.unwrap().1)
                    .collect::<Vec<lib::Token>>()
            })
        });
    }
}

fn tokenize_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tokenize [push] Vec");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                let mut listener = lib::PushToTokens::new();
                lib::tokenize_push(input, &mut listener);
                let (_tokens, error) = listener.into_tokens();
                if error.is_some() {
                    panic!();
                }
            })
        });
    }
}

fn tokenize_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tokenize vec");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| lib::tokenize_list(input).unwrap())
        });
    }
}

fn ast_direct_recursive(c: &mut Criterion) {
    let mut group = c.benchmark_group("AST recursively");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| lib::parse_ast_recursive(input).unwrap());
        });
    }
}

fn ast_direct_non_recursive(c: &mut Criterion) {
    let mut group = c.benchmark_group("AST non-recursively");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| lib::parse_ast_non_recursive(input).unwrap());
        });
    }
}

fn parse_events_direct_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("Parse events [iter] Vec");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                lib::parse_events_iter(input)
                    .map(|ev| ev.unwrap())
                    .collect::<Vec<lib::ParseEvent>>()
            });
        });
    }
}

fn parse_events_direct_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("Parse events [push] Vec");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                let mut push_to_events = lib::PushToEvents::new();
                lib::parse_events_push(input, &mut push_to_events);
                let (_events, error) = push_to_events.into_events();
                if error.is_some() {
                    panic!();
                }
            });
        });
    }
}

fn tokenize_list_events_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tokenize [list] events [iter] Vec");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                lib::parse_events_iter_using_lexer_iter(
                    lib::tokenize_list(&input)
                        .unwrap()
                        .into_iter()
                        .map(Result::Ok),
                    input.len(),
                )
                .map(|ev| ev.unwrap())
                .collect::<Vec<lib::ParseEvent>>()
            });
        });
    }
}

fn tokenize_list_events_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tokenize [list] Events [push] Vec");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                let mut push_to_events = lib::PushToEvents::new();
                lib::parse_events_push_using_lexer_iter(
                    lib::tokenize_list(input)
                        .unwrap()
                        .into_iter()
                        .map(Result::Ok),
                    &mut push_to_events,
                    input.len(),
                );
                let (_events, error) = push_to_events.into_events();
                if error.is_some() {
                    panic!();
                }
            });
        });
    }
}

fn tokenize_iter_events_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tokenize [iter] events [iter] Vec");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                lib::parse_events_iter_using_lexer_iter(lib::tokenize_iter(&input), input.len())
                    .map(|ev| ev.unwrap())
                    .collect::<Vec<lib::ParseEvent>>()
            });
        });
    }
}

fn tokenize_iter_events_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tokenize [iter] Events [push] Vec (recursive)");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                let mut push_to_events = lib::PushToEvents::new();
                lib::parse_events_push_using_lexer_iter(
                    lib::tokenize_iter(input),
                    &mut push_to_events,
                    input.len(),
                );
                let (_events, error) = push_to_events.into_events();
                if error.is_some() {
                    panic!();
                }
            });
        });
    }
}

fn tokenize_iter_events_push_non_recursive(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tokenize [iter] Events [push] Vec (non recursive)");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                let mut push_to_events = lib::PushToEvents::new();
                lib::parse_events_push_using_lexer_iter(
                    lib::tokenize_iter(input),
                    &mut push_to_events,
                    input.len(),
                );
                let (_events, error) = push_to_events.into_events();
                if error.is_some() {
                    panic!();
                }
            });
        });
    }
}

fn tokenize_push_events_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tokenize [push] events [push] Vec");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                let mut push_to_events = lib::PushToEvents::new();
                lib::parse_events_push_using_lexer_push(input, &mut push_to_events);
                let (_events, error) = push_to_events.into_events();
                if error.is_some() {
                    panic!();
                }
            });
        });
    }
}

fn events_direct_iter_ast(c: &mut Criterion) {
    let mut group = c.benchmark_group("Events [iter] AST");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                let mut event_parser = lib::parse_events_iter(input);
                lib::event_to_tree(&mut event_parser, input).unwrap();
            });
        });
    }
}

fn events_direct_push_ast(c: &mut Criterion) {
    let mut group = c.benchmark_group("Events [push] AST");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                let mut listener = lib::AstBuilderListener::new(input);
                lib::parse_events_push(input, &mut listener);
                let (ast, error) = listener.into_ast();
                if error.is_some() {
                    panic!();
                }
                ast.unwrap();
            });
        });
    }
}

fn tokenize_list_events_iter_ast(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tokenize [list] events [iter] AST");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                let mut event_parser = lib::parse_events_iter_using_lexer_iter(
                    lib::tokenize_list(input)
                        .unwrap()
                        .into_iter()
                        .map(Result::Ok),
                    input.len(),
                );
                lib::event_to_tree(&mut event_parser, input).unwrap();
            });
        });
    }
}

fn tokenize_list_events_push_ast(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tokenize [list] events [push] AST");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                let mut listener = lib::AstBuilderListener::new(input);
                lib::parse_events_push_using_lexer_iter(
                    lib::tokenize_list(input)
                        .unwrap()
                        .into_iter()
                        .map(Result::Ok),
                    &mut listener,
                    input.len(),
                );
                let (ast, error) = listener.into_ast();
                if error.is_some() {
                    panic!();
                }
                ast.unwrap();
            });
        });
    }
}

fn tokenize_iter_events_iter_ast(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tokenize [iter] events [iter] AST");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                let mut event_parser =
                    lib::parse_events_iter_using_lexer_iter(lib::tokenize_iter(input), input.len());
                lib::event_to_tree(&mut event_parser, input).unwrap();
            });
        });
    }
}

fn tokenize_iter_events_push_ast(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tokenize [iter] events [push] AST");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                let mut listener = lib::AstBuilderListener::new(input);
                lib::parse_events_push_using_lexer_iter(
                    lib::tokenize_iter(input),
                    &mut listener,
                    input.len(),
                );
                let (ast, error) = listener.into_ast();
                if error.is_some() {
                    panic!();
                }
                ast.unwrap();
            });
        });
    }
}

fn tokenize_push_events_push_ast(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tokenize [push] events [push] AST");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                let mut listener = lib::AstBuilderListener::new(input);
                lib::parse_events_push_using_lexer_push(input, &mut listener);
                let (ast, error) = listener.into_ast();
                if error.is_some() {
                    panic!();
                }
                ast.unwrap();
            });
        });
    }
}

#[rustfmt::skip]
criterion_group!(
    benches,
    // tokenize_iter,
    // tokenize_push,
    // tokenize_list,

    // parse_events_direct_iter,
    // parse_events_direct_push,
    // tokenize_list_events_iter,
    // tokenize_list_events_push,
    // tokenize_iter_events_iter,
    // tokenize_iter_events_push,
    // tokenize_iter_events_push_non_recursive,
    // tokenize_push_events_push,

    ast_direct_recursive,
    ast_direct_non_recursive,
    // events_direct_iter_ast,
    // events_direct_push_ast,
    // tokenize_list_events_iter_ast,
    // tokenize_list_events_push_ast,
    tokenize_iter_events_iter_ast,
    // tokenize_iter_events_push_ast,
    tokenize_push_events_push_ast,
);
criterion_main!(benches);
