//! Benchmarks for CIF parsing performance

use cif_parser::{CIFParser, CifDocument, Rule};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pest::Parser;
use std::path::PathBuf;

fn dict_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates/cif-parser -> crates/
    path.push("cif-validator");
    path.push("dics");
    path.push("cif_core.dic");
    path
}

fn count_pairs(pair: pest::iterators::Pair<Rule>) -> usize {
    let mut count = 1;
    for inner in pair.into_inner() {
        count += count_pairs(inner);
    }
    count
}

fn bench_pest_parse_lazy(c: &mut Criterion) {
    let dict_path = dict_path();
    if !dict_path.exists() {
        eprintln!(
            "Skipping benchmark: dictionary not found at {:?}",
            dict_path
        );
        return;
    }

    let content = std::fs::read_to_string(&dict_path).expect("Failed to read file");

    c.bench_function("pest_parse_lazy", |b| {
        b.iter(|| {
            let pairs = CIFParser::parse(Rule::file, black_box(&content)).expect("Failed to parse");
            black_box(pairs)
        })
    });
}

fn bench_pest_full_traversal(c: &mut Criterion) {
    let dict_path = dict_path();
    if !dict_path.exists() {
        return;
    }

    let content = std::fs::read_to_string(&dict_path).expect("Failed to read file");

    c.bench_function("pest_full_traversal", |b| {
        b.iter(|| {
            let pairs = CIFParser::parse(Rule::file, black_box(&content)).expect("Failed to parse");
            let mut total = 0;
            for pair in pairs {
                total += count_pairs(pair);
            }
            black_box(total)
        })
    });
}

fn bench_full_ast_parse(c: &mut Criterion) {
    let dict_path = dict_path();
    if !dict_path.exists() {
        return;
    }

    let content = std::fs::read_to_string(&dict_path).expect("Failed to read file");

    c.bench_function("full_ast_parse", |b| {
        b.iter(|| {
            let doc = CifDocument::parse(black_box(&content)).expect("Failed to parse");
            black_box(doc)
        })
    });
}

criterion_group!(
    benches,
    bench_pest_parse_lazy,
    bench_pest_full_traversal,
    bench_full_ast_parse
);
criterion_main!(benches);
