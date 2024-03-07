use criterion::black_box;
use criterion::Criterion;

use criterion::{criterion_group, criterion_main};
use file_exchange::manifest::file_hasher::hash_chunk;
use file_exchange::test_util::{random_bytes, CHUNK_SIZE};

fn hash_chunk_benchmark(c: &mut Criterion) {
    let data = black_box(random_bytes(CHUNK_SIZE.try_into().unwrap()));

    c.bench_function("hash_chunk", |b| b.iter(|| hash_chunk(&data)));
}

criterion_group!(benches, hash_chunk_benchmark);
criterion_main!(benches);
