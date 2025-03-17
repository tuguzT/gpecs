use core::mem::{ManuallyDrop, MaybeUninit};

pub union ErasedByte<Fields> {
    _byte: u8,
    _fields: ManuallyDrop<MaybeUninit<Fields>>,
}

unsafe impl<Fields> Send for ErasedByte<Fields> where Fields: Send {}
unsafe impl<Fields> Sync for ErasedByte<Fields> where Fields: Sync {}
