use criterion::async_executor::FuturesExecutor;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use file_exchange::test_util::simple_bundle;
use file_exchange::{
    config::{LocalDirectory, StorageMethod},
    manifest::store::Store,
};
use object_store::path::Path;

fn read_and_validate_file_benchmark(c: &mut Criterion) {
    let store = black_box(
        Store::new(&StorageMethod::LocalFiles(LocalDirectory {
            main_dir: "../example-file".to_string(),
        }))
        .unwrap(),
    );
    let bundle = black_box(simple_bundle());
    let path = black_box(Path::from(""));

    c.bench_function("read_and_validate_file", |b| {
        let meta = black_box(bundle.file_manifests.first().unwrap());
        b.to_async(FuturesExecutor)
            .iter(|| store.read_and_validate_file(meta, &path))
    });
}

criterion_group!(benches, read_and_validate_file_benchmark);
criterion_main!(benches);
