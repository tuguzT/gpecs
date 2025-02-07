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

    fn soa<T>(capacity: usize)
    where
        T: Soa,
    {
        black_box(SoaVec::<T>::with_capacity(black_box(capacity)));
    }

    fn aos<T>(capacity: usize) {
        black_box(Vec::<T>::with_capacity(black_box(capacity)));
    }

    let mut group = c.benchmark_group(format!("With capacity for `{}`", type_name::<T>()));
    for capacity in CAPACITY_RANGE {
        group.bench_with_input(
            BenchmarkId::new(SOA_FUNCTION_NAME, capacity),
            &capacity,
            |b, &capacity| b.iter(|| soa::<T>(capacity)),
        );
        group.bench_with_input(
            BenchmarkId::new(AOS_FUNCTION_NAME, capacity),
            &capacity,
            |b, &capacity| b.iter(|| aos::<T>(capacity)),
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

    fn soa<T>(count: usize)
    where
        T: Soa + Default,
    {
        let mut vec = SoaVec::<T>::new();
        for _ in 0..count {
            vec.push(black_box(T::default()));
        }
        black_box(vec);
    }

    fn aos<T>(count: usize)
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
            |b, &count| b.iter(|| soa::<T>(count)),
        );
        group.bench_with_input(
            BenchmarkId::new(AOS_FUNCTION_NAME, count),
            &count,
            |b, &count| b.iter(|| aos::<T>(count)),
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

fn push_many_preallocated<T>(c: &mut Criterion)
where
    T: Soa + Default,
{
    const COUNT_RANGE: [usize; 6] = [0, 1, 10, 100, 1_000, 10_000];

    fn soa<T>(count: usize, vec: &mut SoaVec<T>)
    where
        T: Soa + Default,
    {
        for _ in 0..count {
            vec.push(black_box(T::default()));
        }
        black_box(vec);
    }

    fn aos<T>(count: usize, vec: &mut Vec<T>)
    where
        T: Default,
    {
        for _ in 0..count {
            vec.push(black_box(T::default()));
        }
        black_box(vec);
    }

    let group_name = format!("Push many (preallocated) for `{}`", type_name::<T>());
    let mut group = c.benchmark_group(group_name);
    for count in COUNT_RANGE {
        let mut vec = SoaVec::<T>::with_capacity(count);
        group.bench_with_input(
            BenchmarkId::new(SOA_FUNCTION_NAME, count),
            &count,
            |b, &count| {
                b.iter(|| soa::<T>(count, &mut vec));
                vec.clear();
            },
        );
        let mut vec = Vec::<T>::with_capacity(count);
        group.bench_with_input(
            BenchmarkId::new(AOS_FUNCTION_NAME, count),
            &count,
            |b, &count| {
                b.iter(|| aos::<T>(count, &mut vec));
                vec.clear();
            },
        );
    }
}

criterion_group!(
    benches_push_many_preallocated,
    push_many_preallocated::<Zero>,
    push_many_preallocated::<Tiny>,
    push_many_preallocated::<Small>,
    push_many_preallocated::<Medium>,
    push_many_preallocated::<Big>,
    push_many_preallocated::<Large>,
);

criterion_main!(
    benches_with_capacity,
    benches_push_many,
    benches_push_many_preallocated,
);
