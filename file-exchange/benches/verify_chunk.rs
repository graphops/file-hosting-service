use std::path::PathBuf;

use criterion::black_box;
use criterion::Criterion;

use criterion::{criterion_group, criterion_main};
use file_exchange::manifest::{file_hasher::verify_chunk, file_reader::read_chunk};
use file_exchange::test_util::simple_file_manifest;

fn verify_chunk_benchmark(c: &mut Criterion) {
    let file_path = black_box(PathBuf::from("../example-file/0017234600.dbin.zst"));
    let file_manifest = black_box(simple_file_manifest());
    // read a chunk
    let (start, end) = black_box((
        file_manifest.total_bytes / file_manifest.chunk_size * file_manifest.chunk_size,
        file_manifest.total_bytes - 1,
    ));
    let data = black_box(read_chunk(&file_path, (start, end)).unwrap());
    let last_hash = black_box(file_manifest.chunk_hashes.last().unwrap());

    c.bench_function("verify_chunk", |b| {
        b.iter(|| verify_chunk(&data, last_hash))
    });
}

criterion_group!(benches, verify_chunk_benchmark);
criterion_main!(benches);
