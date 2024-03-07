use criterion::async_executor::FuturesExecutor;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use file_exchange::manifest::local_file_system::Store;
use file_exchange::test_util::simple_bundle;
use rand::Rng;
use std::{fs::File, ops::Range, path::PathBuf};

fn random_file_range(file_size: usize) -> Range<usize> {
    let mut rng = rand::thread_rng();
    let start: usize = rng.gen_range(0..file_size);
    let end: usize = rng.gen_range(start..=file_size); // end is inclusive here to ensure it's at least 'start'
    Range { start, end }
}

fn read_chunk_benchmark(c: &mut Criterion) {
    let file_path = black_box(PathBuf::from("../example-file/0017234600.dbin.zst"));
    let store = black_box(Store::new("../example-file").unwrap());
    let _bundle = black_box(simple_bundle());
    let file_name = black_box("0017234600.dbin.zst");
    let file = black_box(File::open(file_path).unwrap());
    let file_size: usize = black_box(
        file.metadata()
            .map(|d| d.len())
            .unwrap()
            .try_into()
            .unwrap(),
    );

    c.bench_function("read_chunk", |b| {
        let range = black_box(random_file_range(file_size));
        b.to_async(FuturesExecutor)
            .iter(|| store.range_read(file_name, &range))
    });
}

criterion_group!(benches, read_chunk_benchmark);
criterion_main!(benches);
