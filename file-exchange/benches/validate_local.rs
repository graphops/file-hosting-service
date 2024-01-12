use criterion::black_box;
use criterion::Criterion;

use criterion::{criterion_group, criterion_main};
use file_exchange::test_util::simple_bundle;

fn validate_local_bundle_benchmark(c: &mut Criterion) {
    let bundle = black_box(simple_bundle());
    c.bench_function("validate_local_bundle", |b| {
        b.iter(|| bundle.validate_local_bundle())
    });
}

criterion_group!(benches, validate_local_bundle_benchmark);
criterion_main!(benches);
