use std::{any::type_name, hint::black_box};

use criterion::{criterion_group, criterion_main, Criterion};
use gpecs_soa::prelude::*;

type Zero = ();
type Tiny = (u32,);
type Small = (f64, f64, f64);
type Medium = (Small, Small, Small);
type Big = (Small, Small, [usize; 18], String, String);
type Large = ([u64; 64],);

fn soa_with_capacity<T, const CAP: usize>()
where
    T: Soa,
{
    black_box(SoaVec::<T>::with_capacity(black_box(CAP)));
}

fn aos_with_capacity<T, const CAP: usize>() {
    black_box(Vec::<T>::with_capacity(black_box(CAP)));
}

fn bench_with_capacity<T, const CAP: usize>(c: &mut Criterion)
where
    T: Soa,
{
    let mut group = c.benchmark_group(format!("With capacity for `{}`", type_name::<T>()));
    group.bench_function("SoA (mine)", |b| b.iter(soa_with_capacity::<T, CAP>));
    group.bench_function("AoS (std)", |b| b.iter(aos_with_capacity::<T, CAP>));
}

criterion_group!(
    benches,
    bench_with_capacity::<Zero, 100>,
    bench_with_capacity::<Tiny, 100>,
    bench_with_capacity::<Small, 100>,
    bench_with_capacity::<Medium, 100>,
    bench_with_capacity::<Big, 100>,
    bench_with_capacity::<Large, 100>,
);

criterion_main!(benches);
