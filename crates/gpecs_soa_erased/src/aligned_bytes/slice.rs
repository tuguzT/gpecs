use core::{alloc::Layout, mem::MaybeUninit};

use crate::error::{NotAlignedError, check_align};

pub struct AlignedUninitByteSlice<T>
where
    T: ?Sized,
{
    layout: Layout,
    bytes: T,
}

impl<T> AlignedUninitByteSlice<T> {
    #[inline]
    pub fn new(bytes: T, layout: Layout) -> Result<Self, NotAlignedError>
    where
        T: AsRef<[MaybeUninit<u8>]>,
    {
        let ptr = bytes.as_ref().as_ptr().cast();
        check_align(ptr, layout)?;

        let me = Self { layout, bytes };
        Ok(me)
    }

    #[inline]
    pub fn into_bytes(self) -> T {
        let Self { bytes, .. } = self;
        bytes
    }
}

impl<T> AlignedUninitByteSlice<T>
where
    T: ?Sized,
{
    #[inline]
    pub fn as_bytes(&self) -> &T {
        let Self { bytes, .. } = self;
        bytes
    }

    #[inline]
    pub fn as_mut_bytes(&mut self) -> &mut T {
        let Self { bytes, .. } = self;
        bytes
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub fn as_slice(&self) -> &[MaybeUninit<u8>]
    where
        T: AsRef<[MaybeUninit<u8>]>,
    {
        let Self { bytes, .. } = self;
        bytes.as_ref()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [MaybeUninit<u8>]
    where
        T: AsMut<[MaybeUninit<u8>]>,
    {
        let Self { bytes, .. } = self;
        bytes.as_mut()
    }
}

impl<T> AsRef<[MaybeUninit<u8>]> for AlignedUninitByteSlice<T>
where
    T: AsRef<[MaybeUninit<u8>]> + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &[MaybeUninit<u8>] {
        self.as_slice()
    }
}

impl<T> AsMut<[MaybeUninit<u8>]> for AlignedUninitByteSlice<T>
where
    T: AsMut<[MaybeUninit<u8>]> + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.as_mut_slice()
    }
}
