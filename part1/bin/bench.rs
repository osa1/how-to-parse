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

fn simple_parser_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Recursive descent");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| lib::parse_ast(input).unwrap());
        });
    }
}

fn event_to_tree_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Event generator to AST");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| lib::event_to_tree(&mut lib::parse_events(input), input).unwrap());
        });
    }
}

fn lexgen_event_to_tree_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Lexgen event to AST");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| lib::event_to_tree(&mut lib::parse_events_lexgen(input), input).unwrap());
        });
    }
}

fn push_to_ast(c: &mut Criterion) {
    let mut group = c.benchmark_group("Push event parser to AST");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| lib::parse_events_push(input, &mut lib::AstBuilderListener::new(input)));
        });
    }
}

fn parse_events(c: &mut Criterion) {
    let mut group = c.benchmark_group("Parse events");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                lib::parse_events(input)
                    .map(|ev| ev.unwrap())
                    .collect::<Vec<lib::ParseEvent>>()
            });
        });
    }
}

fn parse_events_lexgen(c: &mut Criterion) {
    let mut group = c.benchmark_group("Parse events lexgen");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                lib::parse_events_lexgen(input)
                    .map(|ev| ev.unwrap())
                    .collect::<Vec<lib::ParseEvent>>()
            });
        });
    }
}

fn parse_events_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("Parse events via push");
    for size in SIZES {
        let input = lib::gen_input(size);
        group.throughput(Throughput::BytesDecimal(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &input, |b, input| {
            b.iter(|| {
                let mut push_to_events = lib::PushToEvents::new();
                lib::parse_events_push(input, &mut push_to_events);
                let (_events, _error) = push_to_events.into_events();
            });
        });
    }
}

criterion_group!(
    benches,
    simple_parser_bench,
    event_to_tree_bench,
    lexgen_event_to_tree_bench,
    push_to_ast,
    parse_events,
    parse_events_lexgen,
    parse_events_push,
);
criterion_main!(benches);
