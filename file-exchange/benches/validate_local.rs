use criterion::black_box;
use criterion::Criterion;

use criterion::async_executor::FuturesExecutor;
use criterion::{criterion_group, criterion_main};
use object_store::path::Path;

use file_exchange::test_util::simple_bundle;
use file_exchange::{
    config::{LocalDirectory, StorageMethod},
    manifest::{store::Store, LocalBundle},
};

fn validate_local_bundle_benchmark(c: &mut Criterion) {
    let store = black_box(
        Store::new(&StorageMethod::LocalFiles(LocalDirectory {
            main_dir: "../example-file".to_string(),
        }))
        .unwrap(),
    );
    let bundle = black_box(simple_bundle());
    let local_path = black_box(Path::from(""));
    let bundle = black_box(LocalBundle { bundle, local_path });

    c.bench_function("validate_local_bundle", |b| {
        b.to_async(FuturesExecutor)
            .iter(|| store.validate_local_bundle(&bundle))
    });
}

criterion_group!(benches, validate_local_bundle_benchmark);
criterion_main!(benches);
