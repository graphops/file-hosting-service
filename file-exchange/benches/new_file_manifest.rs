use criterion::async_executor::FuturesExecutor;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use file_exchange::{
    config::{LocalDirectory, StorageMethod},
    manifest::store::Store,
    test_util::CHUNK_SIZE,
};

fn new_file_manifest_benchmark_object_store(c: &mut Criterion) {
    let store = black_box(
        Store::new(&StorageMethod::LocalFiles(LocalDirectory {
            main_dir: "../example-file".to_string(),
        }))
        .unwrap(),
    );
    let file_name = black_box("0017234600.dbin.zst");
    let file_size = black_box(Some(CHUNK_SIZE as usize));

    c.bench_function("new_file_manifest_benchmark_object_store", |b| {
        b.to_async(FuturesExecutor)
            .iter(|| store.file_manifest(file_name, None, file_size))
    });
}

criterion_group!(benches, new_file_manifest_benchmark_object_store);
criterion_main!(benches);
