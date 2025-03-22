use core::mem::{ManuallyDrop, MaybeUninit};

pub union ErasedByte<F>
where
    F: Fields,
{
    _byte: u8,
    _fields: ManuallyDrop<MaybeUninit<F>>,
}

unsafe impl<F> Send for ErasedByte<F> where F: Fields + Send {}
unsafe impl<F> Sync for ErasedByte<F> where F: Fields + Sync {}

pub struct Unaligned(());

pub struct Aligned<Fields>(Fields);

pub trait Fields: private::Sealed {
    const ALIGNED: bool;
}

impl Fields for Unaligned {
    const ALIGNED: bool = false;
}

impl<F> Fields for Aligned<F> {
    const ALIGNED: bool = true;
}

mod private {
    pub trait Sealed {}

    impl Sealed for super::Unaligned {}
    impl<F> Sealed for super::Aligned<F> {}
}
