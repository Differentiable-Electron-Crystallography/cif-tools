//! Benchmarks for CIF dictionary loading performance

use cif_parser::CifDocument;
use cif_validator::dictionary::load_dictionary;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

fn dict_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("dics");
    path.push("cif_core.dic");
    path
}

fn bench_cif_parse(c: &mut Criterion) {
    let dict_path = dict_path();
    if !dict_path.exists() {
        eprintln!(
            "Skipping benchmark: dictionary not found at {:?}",
            dict_path
        );
        return;
    }

    let content = std::fs::read_to_string(&dict_path).expect("Failed to read file");

    c.bench_function("cif_parse", |b| {
        b.iter(|| {
            let doc = CifDocument::parse(black_box(&content)).expect("Failed to parse");
            black_box(doc)
        })
    });
}

fn bench_dictionary_load(c: &mut Criterion) {
    let dict_path = dict_path();
    if !dict_path.exists() {
        return;
    }

    let content = std::fs::read_to_string(&dict_path).expect("Failed to read file");
    let doc = CifDocument::parse(&content).expect("Failed to parse");

    c.bench_function("dictionary_load", |b| {
        b.iter(|| {
            let dict = load_dictionary(black_box(&doc)).expect("Failed to load dictionary");
            black_box(dict)
        })
    });
}

fn bench_full_dictionary_load(c: &mut Criterion) {
    let dict_path = dict_path();
    if !dict_path.exists() {
        return;
    }

    let path_str = dict_path.to_str().expect("Invalid path");

    c.bench_function("full_dictionary_load", |b| {
        b.iter(|| {
            let dict = cif_validator::load_dictionary_file(black_box(path_str))
                .expect("Failed to load dictionary");
            black_box(dict)
        })
    });
}

criterion_group!(
    benches,
    bench_cif_parse,
    bench_dictionary_load,
    bench_full_dictionary_load
);
criterion_main!(benches);
