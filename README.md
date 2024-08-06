# jxl-decode-bench

## Prerequisites
1. Install clang. It will be used to build libjxl from source.
1. Add test images to `benches/data/` directory. File extension should be `.jxl`.

## Running the benchmark
```
cargo bench
```

## Criterion.rs report
HTML report will be available at `target/criterion/report/index.html` after running benchmark.
