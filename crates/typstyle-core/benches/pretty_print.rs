use std::fs;

use criterion::{criterion_group, criterion_main, Criterion};
use typst_syntax::Source;
use typstyle_core::{Config, Typstyle};

fn bench_pretty(c: &mut Criterion, id: &str, path: &str, config: Config) {
    let content = fs::read_to_string(path).unwrap();
    let source = Source::detached(content);
    let t = Typstyle::new(config);

    c.bench_function(id, |b| {
        b.iter(|| {
            t.format_source(source.clone())
                .render()
                .expect("expect errorless")
        })
    });
}

/// (path, name)
const TEST_ASSETS: &[(&str, &str)] = &[
    ("articles/undergraduate-math", "undergraduate-math"),
    ("articles/_cpe", "cpe"),
    ("packages/cetz-manual", "cetz-manual"),
    ("packages/codly", "codly"),
    ("packages/fletcher-diagram", "fletcher-diagram"),
    ("packages/fletcher-draw", "fletcher-draw"),
    ("packages/tablex", "tablex"),
    ("packages/touying/core", "touying-core"),
    ("packages/touying/utils", "touying-utils"),
    ("unit/code/perf-nest", "deep-nested-args"),
    (
        "ai/feat/comprehensive-showcase",
        "ai-comprehensive-showcase",
    ),
];

fn benchmark_pretty(c: &mut Criterion) {
    for (path, name) in TEST_ASSETS {
        bench_pretty(
            c,
            &format!("pretty-{name}"),
            &format!("../../tests/fixtures/{path}.typ"),
            if *name == "deep-nested-args" {
                Config::new().with_width(10) // special config
            } else {
                Config::new()
            },
        );
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = benchmark_pretty
}
criterion_main!(benches);
