use criterion::{black_box, criterion_group, criterion_main, Criterion};

use subfile_exchange::test_util::simple_subfile;

fn read_and_validate_file_benchmark(c: &mut Criterion) {
    let subfile = black_box(simple_subfile());

    c.bench_function("read_and_validate_file", |b| {
        let meta = black_box(subfile.chunk_files.first().unwrap());
        b.iter(|| subfile.read_and_validate_file(meta))
    });
}

criterion_group!(benches, read_and_validate_file_benchmark);
criterion_main!(benches);
