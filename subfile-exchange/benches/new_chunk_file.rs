use criterion::{black_box, criterion_group, criterion_main, Criterion};
use subfile_exchange::{test_util::CHUNK_SIZE, subfile::ChunkFile};

fn new_chunk_file_benchmark(c: &mut Criterion) {
    // ChunkFile::new(&self.config.read_dir, file_name, self.config.chunk_size)
    let read_dir = black_box("../example-file");
    let file_name = black_box("0017234600.dbin.zst");
    let file_size = black_box(CHUNK_SIZE);

    c.bench_function("new_chunk_file", |b| {
        b.iter(|| ChunkFile::new(read_dir, file_name, file_size).unwrap())
    });
}

criterion_group!(benches, new_chunk_file_benchmark);
criterion_main!(benches);
