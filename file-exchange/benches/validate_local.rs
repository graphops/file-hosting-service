use criterion::black_box;
use criterion::Criterion;

use criterion::async_executor::FuturesExecutor;
use criterion::{criterion_group, criterion_main};
use object_store::path::Path;

use file_exchange::manifest::{local_file_system::Store, LocalBundle};
use file_exchange::test_util::simple_bundle;

fn validate_local_bundle_benchmark(c: &mut Criterion) {
    let store = black_box(Store::new("../example-file").unwrap());
    let bundle = black_box(simple_bundle());
    let _file_name = black_box("0017234600.dbin.zst");
    let local_path = black_box(Path::from(""));
    let bundle = black_box(LocalBundle { bundle, local_path });

    c.bench_function("validate_local_bundle", |b| {
        b.to_async(FuturesExecutor)
            .iter(|| store.validate_local_bundle(&bundle))
    });
}

criterion_group!(benches, validate_local_bundle_benchmark);
criterion_main!(benches);
