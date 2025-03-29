use core::mem::ManuallyDrop;

use crate::align::Align;

pub union ErasedByte<A>
where
    A: Align,
{
    _byte: u8,
    _align: ManuallyDrop<A>,
}

unsafe impl<A> Send for ErasedByte<A> where A: Align + Send {}
unsafe impl<A> Sync for ErasedByte<A> where A: Align + Sync {}
