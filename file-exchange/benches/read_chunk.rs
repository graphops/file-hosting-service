use criterion::{black_box, criterion_group, criterion_main, Criterion};
use file_exchange::manifest::file_reader::read_chunk;
use std::{fs::File, path::PathBuf};

fn read_chunk_benchmark(c: &mut Criterion) {
    let file_path = black_box(PathBuf::from("../example-file/0017234600.dbin.zst"));
    let file = black_box(File::open(&file_path).unwrap());

    let file_size = black_box(file.metadata().map(|d| d.len()).unwrap());

    // Define different test ranges
    let ranges = black_box(vec![
        (0, 999),                          // Small chunk from start
        (file_size / 3, file_size / 2),    // partial chunk from middle
        (file_size - 1000, file_size - 1), // Small chunk from end
    ]);

    c.bench_function("read_chunk", |b| {
        b.iter(|| {
            for &(start, end) in &ranges {
                let _ = read_chunk(&file_path, (start, end)).unwrap();
            }
        })
    });
}

criterion_group!(benches, read_chunk_benchmark);
criterion_main!(benches);
