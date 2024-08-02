use core::marker::PhantomData;

pub struct SparseArena<T> {
    ph: PhantomData<T>,
}
