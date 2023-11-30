use std::path::PathBuf;

use criterion::black_box;
use criterion::Criterion;

use criterion::{criterion_group, criterion_main};
use subfile_exchange::file_hasher::verify_chunk;
use subfile_exchange::file_reader::read_chunk;
use subfile_exchange::test_util::simple_chunk_file;

fn verify_chunk_benchmark(c: &mut Criterion) {
    let file_path = black_box(PathBuf::from("../example-file/0017234600.dbin.zst"));
    // let file = black_box(File::open(&file_path).unwrap());

    // let file_size = black_box(file.metadata().map(|d| d.len()).unwrap());

    // // Define different test ranges
    // let ranges = black_box(vec![
    //     (0, 999),                          // Small chunk from start
    //     (file_size / 3, file_size / 2),    // partial chunk from middle
    //     (file_size - 1000, file_size - 1), // Small chunk from end
    // ]);

    // c.bench_function("read_chunk", |b| {
    //     b.iter(|| {
    //         for &(start, end) in &ranges {
    //             let data = black_box(read_chunk(&file_path, (start, end)).unwrap());

    //         }
    //     })
    // });

    let chunk_file = black_box(simple_chunk_file());
    // read a chunk
    let (start, end) = black_box((
        chunk_file.total_bytes / chunk_file.chunk_size * chunk_file.chunk_size,
        chunk_file.total_bytes - 1,
    ));
    let data = black_box(read_chunk(&file_path, (start, end)).unwrap());
    let last_hash = black_box(chunk_file.chunk_hashes.last().unwrap());

    c.bench_function("verify_chunk", |b| {
        b.iter(|| verify_chunk(&data, last_hash))
    });
}

criterion_group!(benches, verify_chunk_benchmark);
criterion_main!(benches);

// pub const SUBFILE_MANIFEST = r#"files:
// - name: example-create-17686085.dbin
//   hash: QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv
// - name: 0017234500.dbin.zst
//   hash: Qmexz4ZariJteKHHXMxsSeSjvyLZf7SUWz77bsvLUQG1Vn
// - name: 0017234600.dbin.zst
//   hash: QmadNB1AQnap3czUime3gEETBNUj7HHzww6hVh5F6w7Boo
// - name: 0017686111-c1ed20dc4cffd7bd-ebfe6d2b6a25625a-17686021-default.dbin
//   hash: QmSEDiCKax7rjxS3kvGJ3dPdHkm2bztFZkR5KDqfpgyuQw
// - name: 0017686115-f8d105f60fa2e78d-7d23a3e458beaff1-17686021-default.dbin
//   hash: QmVx3JX5TNrSqMKyP5xQJ2CYmcqG4VaBdPnbji3PuvUFx6
// file_type: flatfiles
// spec_version: 0.0.0
// description: random flatfiles
// chain_id: '0'
// block_range:
//   start_block: null
//   end_block: null";

// pub fn init_logger() {
//     env::set_var("RUST_LOG", "warn,subfile_exchange=trace");
//     init_tracing(String::from("pretty")).unwrap();
// }
