use core::marker::PhantomData;

pub type SparseArena<T> = EpochSparseArena<usize, T>;

pub struct EpochSparseArena<K, V> {
    ph: PhantomData<(K, V)>,
}
