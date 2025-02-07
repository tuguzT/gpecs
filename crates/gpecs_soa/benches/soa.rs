use std::{any::type_name, array, convert::identity, hint::black_box, slice};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use gpecs_soa::{prelude::*, slice as soa_slice};

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

trait Work: Soa {
    fn work_item() -> Self;

    type SoaIter<'a>: Iterator + Clone;
    fn soa_prepare_iter(data: &SoaSlice<Self>) -> Self::SoaIter<'_>;
    fn soa_work(iter: Self::SoaIter<'_>);

    type AosIter<'a>: Iterator + Clone;
    fn aos_prepare_iter(data: &[Self]) -> Self::AosIter<'_>;
    fn aos_work(iter: Self::AosIter<'_>);
}

impl Work for Tiny {
    #[inline]
    fn work_item() -> Self {
        (1,)
    }

    type SoaIter<'a> = soa_slice::Iter<'a, Self>;

    fn soa_prepare_iter(data: &SoaSlice<Self>) -> Self::SoaIter<'_> {
        data.iter()
    }

    fn soa_work(iter: Self::SoaIter<'_>) {
        let mut result = 0;
        for (i,) in iter {
            result += *i;
        }
        black_box(result);
    }

    type AosIter<'a> = slice::Iter<'a, Self>;

    fn aos_prepare_iter(data: &[Self]) -> Self::AosIter<'_> {
        data.iter()
    }

    fn aos_work(iter: Self::AosIter<'_>) {
        let mut result = 0;
        for (i,) in iter {
            result += *i;
        }
        black_box(result);
    }
}

impl Work for Small {
    fn work_item() -> Self {
        (1.0, 0.2, -2.3)
    }

    type SoaIter<'a> = soa_slice::Iter<'a, Self>;

    fn soa_prepare_iter(data: &SoaSlice<Self>) -> Self::SoaIter<'_> {
        data.iter()
    }

    fn soa_work(iter: Self::SoaIter<'_>) {
        let mut result = 0.0;
        for (x, y, _) in iter {
            result += *x + *y;
        }
        black_box(result);
    }

    type AosIter<'a> = slice::Iter<'a, Self>;

    fn aos_prepare_iter(data: &[Self]) -> Self::AosIter<'_> {
        data.iter()
    }

    fn aos_work(iter: Self::AosIter<'_>) {
        let mut result = 0.0;
        for (x, y, _) in iter {
            result += *x + *y;
        }
        black_box(result);
    }
}

impl Work for Big {
    fn work_item() -> Self {
        let small = Small::work_item();
        (
            small,
            small,
            array::from_fn(identity),
            "".to_owned(),
            "Hello, World".to_owned(),
        )
    }

    type SoaIter<'a> = soa_slice::Iter<'a, Self>;

    fn soa_prepare_iter(data: &SoaSlice<Self>) -> Self::SoaIter<'_> {
        data.iter()
    }

    fn soa_work(iter: Self::SoaIter<'_>) {
        let mut result = 0;
        for (index, (_, _, array, _, hello)) in iter.enumerate() {
            result += index + array.iter().sum::<usize>() + hello.len();
        }
        black_box(result);
    }

    type AosIter<'a> = slice::Iter<'a, Self>;

    fn aos_prepare_iter(data: &[Self]) -> Self::AosIter<'_> {
        data.iter()
    }

    fn aos_work(iter: Self::AosIter<'_>) {
        let mut result = 0;
        for (index, (_, _, array, _, hello)) in iter.enumerate() {
            result += index + array.iter().sum::<usize>() + hello.len();
        }
        black_box(result);
    }
}

fn work<T>(c: &mut Criterion)
where
    T: Work,
{
    const COUNT_RANGE: [usize; 5] = [10, 100, 1_000, 10_000, 100_000];

    let mut group = c.benchmark_group(format!("Work for `{}`", type_name::<T>()));
    for count in COUNT_RANGE {
        let mut vec = SoaVec::<T>::with_capacity(count);
        for _ in 0..count {
            let value = black_box(T::work_item());
            vec.push(value);
        }
        let iter = T::soa_prepare_iter(&vec);
        group.bench_with_input(
            BenchmarkId::new(SOA_FUNCTION_NAME, count),
            &count,
            |b, _| b.iter(|| T::soa_work(iter.clone())),
        );

        let mut vec = Vec::<T>::with_capacity(count);
        for _ in 0..count {
            let value = black_box(T::work_item());
            vec.push(value);
        }
        let iter = T::aos_prepare_iter(&vec);
        group.bench_with_input(
            BenchmarkId::new(AOS_FUNCTION_NAME, count),
            &count,
            |b, _| b.iter(|| T::aos_work(iter.clone())),
        );
    }
}

criterion_group!(
    benches_work,
    work::<Tiny>,
    work::<Small>,
    // work::<Medium>,
    work::<Big>,
    // work::<Large>,
);

criterion_main!(
    benches_with_capacity,
    benches_push_many,
    benches_push_many_preallocated,
    benches_work,
);
