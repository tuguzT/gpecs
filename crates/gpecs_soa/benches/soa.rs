use std::{any::type_name, hint::black_box};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use gpecs_soa::prelude::*;

type Zero = ();
type Tiny = (u32,);
type Small = (f64, f64, f64);
type Medium = (Small, Small, Small);
type Big = (Small, Small, [usize; 18], String, String);
type Large = ([u64; 64],);

fn soa_with_capacity<T>(capacity: usize)
where
    T: Soa,
{
    black_box(SoaVec::<T>::with_capacity(black_box(capacity)));
}

fn aos_with_capacity<T>(capacity: usize) {
    black_box(Vec::<T>::with_capacity(black_box(capacity)));
}

fn bench_with_capacity<T>(c: &mut Criterion)
where
    T: Soa,
{
    const KB: usize = 1024;
    const CAPACITY_RANGE: [usize; 10] =
        [0, 1, 10, 100, KB, KB * 2, KB * 4, KB * 8, KB * 16, KB * 32];

    let mut group = c.benchmark_group(format!("`with_capacity` for `{}`", type_name::<T>()));
    for capacity in CAPACITY_RANGE {
        group.bench_with_input(
            BenchmarkId::new("SoA (mine)", capacity),
            &capacity,
            |b, &capacity| b.iter(|| soa_with_capacity::<T>(capacity)),
        );
        group.bench_with_input(
            BenchmarkId::new("AoS (std)", capacity),
            &capacity,
            |b, &capacity| b.iter(|| aos_with_capacity::<T>(capacity)),
        );
    }
}

criterion_group!(
    benches,
    bench_with_capacity::<Zero>,
    bench_with_capacity::<Tiny>,
    bench_with_capacity::<Small>,
    bench_with_capacity::<Medium>,
    bench_with_capacity::<Big>,
    bench_with_capacity::<Large>,
);

criterion_main!(benches);
