use std::{any::type_name, hint::black_box};

use criterion::{criterion_group, BenchmarkId, Criterion};
use gpecs_soa::prelude::*;

use super::{with_capacity::WithCapacity, *};

pub(super) trait PushMany: WithCapacity + Default {
    fn soa_push_many(count: usize, vec: &mut SoaVec<Self>) {
        for _ in 0..count {
            vec.push(black_box(Self::default()));
        }
        black_box(vec);
    }

    fn soa_vec_clear(vec: &mut SoaVec<Self>) {
        vec.clear();
        black_box(vec);
    }

    fn aos_push_many(count: usize, vec: &mut Vec<Self>) {
        for _ in 0..count {
            vec.push(black_box(Self::default()));
        }
        black_box(vec);
    }

    fn aos_vec_clear(vec: &mut Vec<Self>) {
        vec.clear();
        black_box(vec);
    }
}

impl PushMany for Zero {}

impl PushMany for Tiny {}

impl PushMany for Small {}

impl PushMany for Medium {}

impl PushMany for Big {}

impl PushMany for Large {}

fn push_many<T>(c: &mut Criterion)
where
    T: PushMany,
{
    const COUNT_RANGE: [usize; 6] = [0, 1, 10, 100, 1_000, 10_000];

    let mut group = c.benchmark_group(format!("Push many for `{}`", type_name::<T>()));
    for count in COUNT_RANGE {
        group.bench_with_input(
            BenchmarkId::new(SOA_FUNCTION_NAME, count),
            &count,
            |b, &count| {
                let mut vec = T::soa_with_capacity(0);
                b.iter(|| T::soa_push_many(count, &mut vec))
            },
        );
        group.bench_with_input(
            BenchmarkId::new(AOS_FUNCTION_NAME, count),
            &count,
            |b, &count| {
                let mut vec = T::aos_with_capacity(0);
                b.iter(|| T::aos_push_many(count, &mut vec))
            },
        );
    }
    group.finish();

    let group_name = format!("Push many (preallocated) for `{}`", type_name::<T>());
    let mut group = c.benchmark_group(group_name);
    for count in COUNT_RANGE {
        let mut vec = T::soa_with_capacity(count);
        group.bench_with_input(
            BenchmarkId::new(SOA_FUNCTION_NAME, count),
            &count,
            |b, &count| {
                b.iter(|| T::soa_push_many(count, &mut vec));
                T::soa_vec_clear(&mut vec);
            },
        );
        let mut vec = T::aos_with_capacity(count);
        group.bench_with_input(
            BenchmarkId::new(AOS_FUNCTION_NAME, count),
            &count,
            |b, &count| {
                b.iter(|| T::aos_push_many(count, &mut vec));
                T::aos_vec_clear(&mut vec);
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
