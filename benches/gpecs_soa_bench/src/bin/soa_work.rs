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
    const COUNT: usize = 1_000_000;

    let work = work::<Large>(COUNT);
    println!("result of work for `Large` is {work:?}");

    let work_erased = work_erased::<Large>(COUNT);
    println!("result of work for erased `Large` is {work_erased:?}");

    assert_eq!(work, work_erased, "results should match");
}
