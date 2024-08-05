use std::{io::Cursor, path::Path, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jpegxl_rs::ThreadsRunner;
use jxl_oxide::JxlThreadPool;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn decode(c: &mut Criterion) {
    let mut bench_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    bench_path.push("benches/data");

    let oxide_pool = JxlThreadPool::rayon(None);
    let libjxl_pool = ThreadsRunner::new(None, None).unwrap();

    for entry in std::fs::read_dir(&bench_path).unwrap() {
        let entry = entry.unwrap();
        let mut path = entry.path();
        if path.extension() != Some(std::ffi::OsStr::new("jxl")) {
            continue;
        }

        path.set_extension("");
        let name = path.file_name().unwrap().to_str().unwrap();
        bench_one(c, &bench_path, name, &oxide_pool, &libjxl_pool);
    }
}

fn libjxl_create_decoder<'pr, 'mm>(
    pool: &'pr ThreadsRunner,
) -> jpegxl_rs::decode::JxlDecoder<'pr, 'mm> {
    jpegxl_rs::decoder_builder()
        .parallel_runner(pool)
        .build()
        .expect("failed to create libjxl decoder")
}

fn bench_one(
    c: &mut Criterion,
    bench_path: &Path,
    name: &str,
    oxide_pool: &JxlThreadPool,
    libjxl_pool: &ThreadsRunner,
) {
    let mut g = c.benchmark_group(name);
    g.sample_size(40);
    g.warm_up_time(Duration::from_secs(5));
    g.measurement_time(Duration::from_secs(15));

    let path = bench_path.join(format!("{}.jxl", name));
    let data = std::fs::read(path).unwrap();

    let pixels = {
        let reader = Cursor::new(&data);
        let image = jxl_oxide::JxlImage::builder()
            .pool(JxlThreadPool::none())
            .read(reader)
            .unwrap();
        image.width() as u64 * image.height() as u64
    };
    g.throughput(criterion::Throughput::Elements(pixels));

    g.bench_with_input("jxl-oxide", &data, |b, data| {
        b.iter(|| {
            let reader = Cursor::new(data);
            let image = jxl_oxide::JxlImage::builder()
                .pool(oxide_pool.clone())
                .read(reader)
                .unwrap();
            image.render_frame(black_box(0)).unwrap()
        })
    });

    g.bench_with_input("libjxl", &data, |b, data| {
        b.iter(|| {
            let decoder = libjxl_create_decoder(libjxl_pool);
            decoder.decode_with::<u8>(black_box(data)).unwrap()
        })
    });

    g.finish();
}

criterion_group!(group, decode);
criterion_main!(group);
