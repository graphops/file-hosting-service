use criterion::async_executor::FuturesExecutor;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use file_exchange::{
    manifest::{local_file_system::Store, FileManifest},
    test_util::CHUNK_SIZE,
};
fn new_file_manifest_benchmark_file_store(c: &mut Criterion) {
    // FileManifest::new(&self.config.read_dir, file_name, self.config.chunk_size)
    let read_dir = black_box("../example-file");
    let file_name = black_box("0017234600.dbin.zst");
    let file_size = black_box(CHUNK_SIZE);

    c.bench_function("new_file_manifest_benchmark_file_store", |b| {
        b.iter(|| FileManifest::new(read_dir, file_name, file_size).unwrap())
    });
}

fn new_file_manifest_benchmark_object_store(c: &mut Criterion) {
    let store = black_box(Store::new("../example-file").unwrap());
    let file_name = black_box("0017234600.dbin.zst");
    let file_size = black_box(Some(CHUNK_SIZE as usize));

    c.bench_function("new_file_manifest_benchmark_object_store", |b| {
        b.to_async(FuturesExecutor)
            .iter(|| store.file_manifest(file_name, file_size))
    });
}

criterion_group!(
    benches,
    new_file_manifest_benchmark_file_store,
    new_file_manifest_benchmark_object_store
);
criterion_main!(benches);
