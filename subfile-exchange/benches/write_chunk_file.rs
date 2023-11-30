use criterion::{black_box, criterion_group, criterion_main, Criterion};

use subfile_exchange::file_hasher::write_chunk_file;

fn write_chunk_file_benchmark(c: &mut Criterion) {
    let read_dir = black_box("../example-file");
    let file_name = black_box("0017234600.dbin.zst");

    c.bench_function("write_chunk_file", |b| {
        b.iter(|| write_chunk_file(read_dir, file_name).unwrap())
    });
}

criterion_group!(benches, write_chunk_file_benchmark);
criterion_main!(benches);
