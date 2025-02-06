use std::{any::type_name, hint::black_box};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use gpecs_soa::prelude::*;

type Zero = ();
type Tiny = (u32,);
type Small = (f64, f64, f64);
type Medium = (Small, Small, Small);
type Big = (Small, Small, [usize; 18], String, String);
type Large = ([u64; 32], [u64; 32], [u64; 32], [u64; 32], [u64; 32]);

const SOA_FUNCTION_NAME: &str = "SoA (mine)";
const AOS_FUNCTION_NAME: &str = "AoS (std)";

fn with_capacity<T>(c: &mut Criterion)
where
    T: Soa,
{
    const KB: usize = 1024;
    const CAPACITY_RANGE: [usize; 8] = [0, 1, 10, 100, KB, KB * 2, KB * 4, KB * 8];

    fn soa_with_capacity<T>(capacity: usize)
    where
        T: Soa,
    {
        black_box(SoaVec::<T>::with_capacity(black_box(capacity)));
    }

    fn aos_with_capacity<T>(capacity: usize) {
        black_box(Vec::<T>::with_capacity(black_box(capacity)));
    }

    let mut group = c.benchmark_group(format!("With capacity for `{}`", type_name::<T>()));
    for capacity in CAPACITY_RANGE {
        group.bench_with_input(
            BenchmarkId::new(SOA_FUNCTION_NAME, capacity),
            &capacity,
            |b, &capacity| b.iter(|| soa_with_capacity::<T>(capacity)),
        );
        group.bench_with_input(
            BenchmarkId::new(AOS_FUNCTION_NAME, capacity),
            &capacity,
            |b, &capacity| b.iter(|| aos_with_capacity::<T>(capacity)),
        );
    }
}

criterion_group!(
    benches_with_capacity,
    with_capacity::<Zero>,
    with_capacity::<Tiny>,
    with_capacity::<Small>,
    with_capacity::<Medium>,
    with_capacity::<Big>,
    with_capacity::<Large>,
);

fn push_many<T>(c: &mut Criterion)
where
    T: Soa + Default,
{
    const COUNT_RANGE: [usize; 6] = [0, 1, 10, 100, 1_000, 10_000];

    fn soa_push_many<T>(count: usize)
    where
        T: Soa + Default,
    {
        let mut vec = SoaVec::<T>::new();
        for _ in 0..count {
            vec.push(black_box(T::default()));
        }
        black_box(vec);
    }

    fn aos_push_many<T>(count: usize)
    where
        T: Default,
    {
        let mut vec = Vec::<T>::new();
        for _ in 0..count {
            vec.push(black_box(T::default()));
        }
        black_box(vec);
    }

    let mut group = c.benchmark_group(format!("Push many for `{}`", type_name::<T>()));
    for count in COUNT_RANGE {
        group.bench_with_input(
            BenchmarkId::new(SOA_FUNCTION_NAME, count),
            &count,
            |b, &capacity| b.iter(|| soa_push_many::<T>(capacity)),
        );
        group.bench_with_input(
            BenchmarkId::new(AOS_FUNCTION_NAME, count),
            &count,
            |b, &capacity| b.iter(|| aos_push_many::<T>(capacity)),
        );
    }
}

criterion_group!(
    benches_push_many,
    push_many::<Zero>,
    push_many::<Tiny>,
    push_many::<Small>,
    push_many::<Medium>,
    push_many::<Big>,
    push_many::<Large>,
);

criterion_main!(benches_with_capacity, benches_push_many);
