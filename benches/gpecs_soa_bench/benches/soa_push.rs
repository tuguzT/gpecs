use std::{any::type_name, hint::black_box};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use gpecs_soa_bench::{
    Big, Large, Medium, Small, Tiny, Zero, clear::Clear, names::*, push::Push,
    with_capacity::WithCapacity,
};
use gpecs_soa_erased::erased::ErasedSoa;

fn push<T>(c: &mut Criterion)
where
    T: WithCapacity + Push + Clear + Default,
{
    const COUNT_RANGE: [usize; 6] = [0, 1, 10, 100, 1_000, 10_000];
    let group_name = format!("Push many for `{}`", type_name::<T>());

    let mut group = c.benchmark_group(&group_name);
    for count in COUNT_RANGE {
        group
            .throughput(Throughput::Elements(count.try_into().unwrap()))
            .bench_with_input(
                BenchmarkId::new(SOA_SER_FUNCTION_NAME, count),
                &count,
                |b, &count| {
                    let context = Default::default();
                    let mut vec = T::soa_ser_with_capacity(0);
                    b.iter(|| {
                        for _ in 0..count {
                            let value = ErasedSoa::from::<T>(&context, Default::default());
                            T::soa_ser_push(&mut vec, value);
                        }
                        black_box(&mut vec);
                    });
                },
            );
    }
    group.finish();

    let mut group = c.benchmark_group(&group_name);
    for count in COUNT_RANGE {
        group
            .throughput(Throughput::Elements(count.try_into().unwrap()))
            .bench_with_input(
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

        group
            .throughput(Throughput::Elements(count.try_into().unwrap()))
            .bench_with_input(
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

        group
            .throughput(Throughput::Elements(count.try_into().unwrap()))
            .bench_with_input(
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
}

fn push_preallocated<T>(c: &mut Criterion)
where
    T: WithCapacity + Push + Clear + Default,
{
    const COUNT_RANGE: [usize; 6] = [0, 1, 10, 100, 1_000, 10_000];
    let group_name = format!("Push many (preallocated) for `{}`", type_name::<T>());

    let mut group = c.benchmark_group(&group_name);
    for count in COUNT_RANGE {
        let context = Default::default();
        let mut vec = T::soa_ser_with_capacity(count);
        group
            .throughput(Throughput::Elements(count.try_into().unwrap()))
            .bench_with_input(
                BenchmarkId::new(SOA_SER_FUNCTION_NAME, count),
                &count,
                |b, &count| {
                    b.iter(|| {
                        for _ in 0..count {
                            let value = ErasedSoa::from::<T>(&context, Default::default());
                            T::soa_ser_push(&mut vec, value);
                        }
                        black_box(&mut vec);
                    });
                    T::soa_ser_clear(&mut vec);
                },
            );
    }
    group.finish();

    let mut group = c.benchmark_group(&group_name);
    for count in COUNT_RANGE {
        let mut vec = T::soa_slf_with_capacity(count);
        group
            .throughput(Throughput::Elements(count.try_into().unwrap()))
            .bench_with_input(
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

        let mut vec = T::soa_std_with_capacity(count);
        group
            .throughput(Throughput::Elements(count.try_into().unwrap()))
            .bench_with_input(
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
        group
            .throughput(Throughput::Elements(count.try_into().unwrap()))
            .bench_with_input(
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
    group.finish();
}

criterion_group!(
    push_benches,
    push::<Zero>,
    push::<Tiny>,
    push::<Small>,
    push::<Medium>,
    push::<Big>,
    push::<Large>,
);

criterion_group!(
    push_preallocated_benches,
    push_preallocated::<Zero>,
    push_preallocated::<Tiny>,
    push_preallocated::<Small>,
    push_preallocated::<Medium>,
    push_preallocated::<Big>,
    push_preallocated::<Large>,
);

criterion_main!(push_benches, push_preallocated_benches);
