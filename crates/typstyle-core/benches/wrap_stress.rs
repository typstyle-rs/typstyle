//! Stress-benchmarks unspaced CJK sentence wrapping across input sizes.
//!
//! Run the scaling benchmark:
//!
//! ```text
//! cargo bench -p typstyle-core --bench wrap_stress
//! ```
//!
//! The `wrap-cjk-scale/sentence/<repetitions>` cases scale a single unspaced
//! CJK paragraph with inline markup across three sizes so quadratic rescanning
//! regressions show up in Criterion's scaling plots. This target is excluded
//! from automated CI benchmark runs and is intended for on-demand local checks.

use std::sync::LazyLock;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use typst_syntax::Source;
use typstyle_core::{Config, Typstyle, WrapMode};

/// Scales a continuous unspaced CJK prose run so quadratic rescanning shows up
/// across input sizes.
fn benchmark_wrap_cjk_scale(c: &mut Criterion) {
    const FRAGMENT: &str = "这是一段用于排版测试的*中文说明*。下一句描述_不同的格式设置_。";

    let formatter = Typstyle::new(Config::new().with_wrap_mode(WrapMode::Sentence));
    let mut group = c.benchmark_group("wrap-cjk-scale");
    for repetitions in [64, 256, 1024] {
        let bytes = (FRAGMENT.len() * repetitions) as u64;
        let source = LazyLock::new(move || Source::detached(FRAGMENT.repeat(repetitions)));
        group.throughput(Throughput::Bytes(bytes));
        group.bench_function(BenchmarkId::new("sentence", repetitions), |b| {
            let source = &*source;
            let format = || {
                formatter
                    .format_source(source.clone())
                    .render()
                    .expect("stress document should format")
            };
            let output = format();
            assert_eq!(output.lines().count(), 2 * repetitions);
            assert!(!Source::detached(output).root().diagnosis().errors);
            b.iter(format);
        });
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = benchmark_wrap_cjk_scale
}
criterion_main!(benches);
