use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wwc_core::group::{
    stats::{TableStats, UnaryStat},
    Group,
};

pub fn criterion_benchmark(c: &mut Criterion) {
    let seed = Some(0);
    let group = Group::random(6, 4, 6, seed);
    c.bench_function("team_stats_for_TableStats", |b| {
        b.iter(|| TableStats::team_stats(black_box(&group)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
