use core::{marker::PhantomData, mem::MaybeUninit};

pub struct ErasedSoaFields<A> {
    _byte: MaybeUninit<A>,
    _phantom: PhantomData<*const A>,
}
