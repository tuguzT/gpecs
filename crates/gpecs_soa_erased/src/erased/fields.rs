use core::{marker::PhantomData, mem::MaybeUninit};

pub struct ErasedSoaFields {
    _byte: MaybeUninit<u8>,
    _phantom: PhantomData<*const u8>,
}
