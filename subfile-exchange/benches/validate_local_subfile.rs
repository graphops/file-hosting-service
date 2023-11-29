use criterion::black_box;
use criterion::Criterion;

use criterion::{criterion_group, criterion_main};
use subfile_exchange::test_util::simple_subfile;

fn validate_local_subfile_benchmark(c: &mut Criterion) {
    let subfile = black_box(simple_subfile());
    c.bench_function("validate_local_subfile", |b| {
        b.iter(|| subfile.validate_local_subfile())
    });
}

criterion_group!(benches, validate_local_subfile_benchmark);
criterion_main!(benches);
