use std::{any::type_name, hint::black_box};

use criterion::{criterion_group, BenchmarkId, Criterion};
use gpecs_soa::prelude::*;

use super::{with_capacity::WithCapacity, *};

pub(super) trait Push: WithCapacity {
    fn soa_slf_push(vec: &mut SoaVec<Self>, value: Self) {
        let value = black_box(value);
        vec.push(value);
    }

    fn soa_slf_clear(vec: &mut SoaVec<Self>) {
        vec.clear();
    }

    fn soa_std_push(vecs: &mut Self::Vecs, value: Self);

    fn soa_std_clear(vecs: &mut Self::Vecs);

    fn aos_std_push(vec: &mut Vec<Self>, value: Self) {
        let value = black_box(value);
        vec.push(value);
    }

    fn aos_std_clear(vec: &mut Vec<Self>) {
        vec.clear();
    }
}

impl Push for Zero {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let value = black_box(value);
        vecs.push(value);
    }

    fn soa_std_clear(vecs: &mut Self::Vecs) {
        vecs.clear();
    }
}

impl Push for Tiny {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (values,) = vecs;
        let (value,) = black_box(value);
        values.push(value);
    }

    fn soa_std_clear(vecs: &mut Self::Vecs) {
        let (values,) = vecs;
        values.clear();
    }
}

impl Push for Small {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (xs, ys, zs) = vecs;
        let (x, y, z) = black_box(value);
        xs.push(x);
        ys.push(y);
        zs.push(z);
    }

    fn soa_std_clear(vecs: &mut Self::Vecs) {
        let (xs, ys, zs) = vecs;
        xs.clear();
        ys.clear();
        zs.clear();
    }
}

impl Push for Medium {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (smalls1, smalls2, smalls3) = vecs;
        let (small1, small2, small3) = black_box(value);
        smalls1.push(small1);
        smalls2.push(small2);
        smalls3.push(small3);
    }

    fn soa_std_clear(vecs: &mut Self::Vecs) {
        let (smalls1, smalls2, smalls3) = vecs;
        smalls1.clear();
        smalls2.clear();
        smalls3.clear();
    }
}

impl Push for Big {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (smalls1, smalls2, arrays, strs1, strs2) = vecs;
        let (small1, small2, array, str1, str2) = black_box(value);
        smalls1.push(small1);
        smalls2.push(small2);
        arrays.push(array);
        strs1.push(str1);
        strs2.push(str2);
    }

    fn soa_std_clear(vecs: &mut Self::Vecs) {
        let (smalls1, smalls2, arrays, strs1, strs2) = vecs;
        smalls1.clear();
        smalls2.clear();
        arrays.clear();
        strs1.clear();
        strs2.clear();
    }
}

impl Push for Large {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (
            arrays1,
            arrays2,
            arrays3,
            arrays4,
            arrays5,
            arrays6,
            arrays7,
            arrays8,
            arrays9,
            arrays10,
        ) = vecs;
        let (array1, array2, array3, array4, array5, array6, array7, array8, array9, array10) =
            black_box(value);
        arrays1.push(array1);
        arrays2.push(array2);
        arrays3.push(array3);
        arrays4.push(array4);
        arrays5.push(array5);
        arrays6.push(array6);
        arrays7.push(array7);
        arrays8.push(array8);
        arrays9.push(array9);
        arrays10.push(array10);
    }

    fn soa_std_clear(vecs: &mut Self::Vecs) {
        let (
            arrays1,
            arrays2,
            arrays3,
            arrays4,
            arrays5,
            arrays6,
            arrays7,
            arrays8,
            arrays9,
            arrays10,
        ) = vecs;
        arrays1.clear();
        arrays2.clear();
        arrays3.clear();
        arrays4.clear();
        arrays5.clear();
        arrays6.clear();
        arrays7.clear();
        arrays8.clear();
        arrays9.clear();
        arrays10.clear();
    }
}

fn push_many<T>(c: &mut Criterion)
where
    T: Push + Default,
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
    push_many::<Zero>,
    push_many::<Tiny>,
    push_many::<Small>,
    push_many::<Medium>,
    push_many::<Big>,
    push_many::<Large>,
);
