use std::path::PathBuf;

use criterion::black_box;
use criterion::Criterion;

use criterion::{criterion_group, criterion_main};
use subfile_exchange::subfile::{file_hasher::verify_chunk, file_reader::read_chunk};
use subfile_exchange::test_util::simple_chunk_file;

fn verify_chunk_benchmark(c: &mut Criterion) {
    let file_path = black_box(PathBuf::from("../example-file/0017234600.dbin.zst"));
    let chunk_file = black_box(simple_chunk_file());
    // read a chunk
    let (start, end) = black_box((
        chunk_file.total_bytes / chunk_file.chunk_size * chunk_file.chunk_size,
        chunk_file.total_bytes - 1,
    ));
    let data = black_box(read_chunk(&file_path, (start, end)).unwrap());
    let last_hash = black_box(chunk_file.chunk_hashes.last().unwrap());

    c.bench_function("verify_chunk", |b| {
        b.iter(|| verify_chunk(&data, last_hash))
    });
}

criterion_group!(benches, verify_chunk_benchmark);
criterion_main!(benches);
