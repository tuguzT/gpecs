use std::{any::type_name, hint::black_box};

use criterion::{criterion_group, BenchmarkId, Criterion};
use gpecs_soa_bench::{
    clear::Clear, push::Push, with_capacity::WithCapacity, Big, Large, Medium, Small, Tiny, Zero,
};
use gpecs_soa_erased::erased::ErasedSoa;

use super::names::*;

fn push<T>(c: &mut Criterion)
where
    T: WithCapacity + Push + Clear + Default,
{
    const COUNT_RANGE: [usize; 6] = [0, 1, 10, 100, 1_000, 10_000];

    let mut group = c.benchmark_group(format!("Push many for `{}`", type_name::<T>()));
    for count in COUNT_RANGE {
        group.bench_with_input(
            BenchmarkId::new(SOA_SLF_FUNCTION_NAME, count),
            &count,
            |b, &count| {
                let mut vec = T::soa_slf_with_capacity(0);
                b.iter(|| {
                    for _ in 0..count {
                        T::soa_slf_push(&mut vec, Default::default());
                    }
                    black_box(&mut vec);
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new(SOA_SER_FUNCTION_NAME, count),
            &count,
            |b, &count| {
                let context = Default::default();
                let mut vec = T::soa_ser_with_capacity(0);
                b.iter(|| {
                    for _ in 0..count {
                        let value = ErasedSoa::from::<T>(&context, Default::default()).unwrap();
                        T::soa_ser_push(&mut vec, value);
                    }
                    black_box(&mut vec);
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new(SOA_STD_FUNCTION_NAME, count),
            &count,
            |b, &count| {
                let mut vec = T::soa_std_with_capacity(0);
                b.iter(|| {
                    for _ in 0..count {
                        T::soa_std_push(&mut vec, Default::default());
                    }
                    black_box(&mut vec);
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new(AOS_STD_FUNCTION_NAME, count),
            &count,
            |b, &count| {
                let mut vec = T::aos_std_with_capacity(0);
                b.iter(|| {
                    for _ in 0..count {
                        T::aos_std_push(&mut vec, Default::default());
                    }
                    black_box(&mut vec);
                });
            },
        );
    }
    group.finish();

    let group_name = format!("Push many (preallocated) for `{}`", type_name::<T>());
    let mut group = c.benchmark_group(group_name);
    for count in COUNT_RANGE {
        let mut vec = T::soa_slf_with_capacity(count);
        group.bench_with_input(
            BenchmarkId::new(SOA_SLF_FUNCTION_NAME, count),
            &count,
            |b, &count| {
                b.iter(|| {
                    for _ in 0..count {
                        T::soa_slf_push(&mut vec, Default::default());
                    }
                    black_box(&mut vec);
                });
                T::soa_slf_clear(&mut vec);
            },
        );
        let context = Default::default();
        let mut vec = T::soa_ser_with_capacity(count);
        group.bench_with_input(
            BenchmarkId::new(SOA_SER_FUNCTION_NAME, count),
            &count,
            |b, &count| {
                b.iter(|| {
                    for _ in 0..count {
                        let value = ErasedSoa::from::<T>(&context, Default::default()).unwrap();
                        T::soa_ser_push(&mut vec, value);
                    }
                    black_box(&mut vec);
                });
                T::soa_ser_clear(&mut vec);
            },
        );
        let mut vec = T::soa_std_with_capacity(count);
        group.bench_with_input(
            BenchmarkId::new(SOA_STD_FUNCTION_NAME, count),
            &count,
            |b, &count| {
                b.iter(|| {
                    for _ in 0..count {
                        T::soa_std_push(&mut vec, Default::default());
                    }
                    black_box(&mut vec);
                });
                T::soa_std_clear(&mut vec);
            },
        );
        let mut vec = T::aos_std_with_capacity(count);
        group.bench_with_input(
            BenchmarkId::new(AOS_STD_FUNCTION_NAME, count),
            &count,
            |b, &count| {
                b.iter(|| {
                    for _ in 0..count {
                        T::aos_std_push(&mut vec, Default::default());
                    }
                    black_box(&mut vec);
                });
                T::aos_std_clear(&mut vec);
            },
        );
    }
}

criterion_group!(
    benches,
    push::<Zero>,
    push::<Tiny>,
    push::<Small>,
    push::<Medium>,
    push::<Big>,
    push::<Large>,
);
