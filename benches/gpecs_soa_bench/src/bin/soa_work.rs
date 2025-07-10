use gpecs_soa_bench::{Large, work::Work};

fn work<T>(count: usize) -> T::Output
where
    T: Work,
{
    let vec = T::soa_slf_prepare_vec(count);
    let iter = T::soa_slf_prepare_iter(vec.slices());
    T::soa_slf_work(iter)
}

fn work_erased<T>(count: usize) -> T::Output
where
    T: Work,
{
    let vec = T::soa_ser_prepare_vec(count);
    let iter = T::soa_ser_prepare_iter(vec.slices());
    T::soa_ser_work(iter)
}

fn main() {
    let large = work::<Large>(100_000);
    println!("Result of work for `Large` is {large:?}");

    let large_erased = work_erased::<Large>(100_000);
    println!("Result of work for erased `Large` is {large_erased:?}");
}
