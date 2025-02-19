use std::{any::type_name, hint::black_box};

use criterion::{criterion_group, BenchmarkId, Criterion};
use gpecs_soa::prelude::*;

use super::*;

fn push_many<T>(c: &mut Criterion)
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

    let mut group = c.benchmark_group(format!("Push many for `{}`", type_name::<T>()));
    for count in COUNT_RANGE {
        group.bench_with_input(
            BenchmarkId::new(SOA_FUNCTION_NAME, count),
            &count,
            |b, &count| b.iter(|| soa::<T>(count, &mut SoaVec::new())),
        );
        group.bench_with_input(
            BenchmarkId::new(AOS_FUNCTION_NAME, count),
            &count,
            |b, &count| b.iter(|| aos::<T>(count, &mut Vec::new())),
        );
    }
    group.finish();

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
    benches,
    push_many::<Zero>,
    push_many::<Tiny>,
    push_many::<Small>,
    push_many::<Medium>,
    push_many::<Big>,
    push_many::<Large>,
);
