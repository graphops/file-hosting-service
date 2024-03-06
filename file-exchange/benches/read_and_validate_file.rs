// use criterion::{black_box, criterion_group, criterion_main, Criterion};

// use file_exchange::test_util::simple_bundle;

// fn read_and_validate_file_benchmark(c: &mut Criterion) {
//     let bundle = black_box(simple_bundle());

//     c.bench_function("read_and_validate_file", |b| {
//         let meta = black_box(bundle.file_manifests.first().unwrap());
//         b.iter(|| bundle.read_and_validate_file(meta))
//     });
// }

// criterion_group!(benches, read_and_validate_file_benchmark);
// criterion_main!(benches);
