use core::{
    alloc::Layout,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    slice,
};

use crate::aligned_bytes::AlignedBytesFromLayout;

use super::AlignedBytes;

const ZERO_UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AlignedInitBytes<B>
where
    B: ?Sized,
{
    bytes: B,
}

impl<B> AlignedInitBytes<B> {
    #[inline]
    pub unsafe fn new_unchecked(bytes: B) -> Self {
        Self { bytes }
    }

    #[inline]
    pub fn into_inner(self) -> B {
        let Self { bytes } = self;
        bytes
    }
}

impl<B> AlignedInitBytes<B>
where
    B: AlignedBytes,
{
    #[inline]
    pub fn new(mut bytes: B) -> Self {
        bytes.as_uninit_bytes_mut().fill(ZERO_UNINIT_BYTE);
        unsafe { Self::new_unchecked(bytes) }
    }
}

impl<B> AlignedInitBytes<B>
where
    B: AlignedBytesFromLayout,
{
    #[inline]
    pub fn from_layout(layout: Layout) -> Result<Self, B::Error> {
        let bytes = B::from_layout(layout)?;
        let me = Self::new(bytes);
        Ok(me)
    }

    #[inline]
    pub fn set_layout(&mut self, layout: Layout) -> Result<(), B::Error> {
        let Self { bytes } = self;

        let old_len = bytes.layout().size();
        bytes.set_layout(layout)?;

        if let Some(remainder) = bytes.as_uninit_bytes_mut().get_mut(old_len..) {
            remainder.fill(ZERO_UNINIT_BYTE);
        }
        Ok(())
    }
}

impl<B> AlignedInitBytes<B>
where
    B: AlignedBytes + ?Sized,
{
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        let Self { bytes } = self;
        let bytes = bytes.as_uninit_bytes();

        let data = bytes.as_ptr().cast();
        let len = bytes.len();
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        let Self { bytes } = self;
        let bytes = bytes.as_uninit_bytes_mut();

        let data = bytes.as_mut_ptr().cast();
        let len = bytes.len();
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { bytes } = self;
        bytes.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { bytes } = self;
        bytes.as_mut_ptr()
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { bytes } = self;
        bytes.layout()
    }
}

impl<B> Deref for AlignedInitBytes<B>
where
    B: AlignedBytes + ?Sized,
{
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<B> DerefMut for AlignedInitBytes<B>
where
    B: AlignedBytes + ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<B> AsRef<[u8]> for AlignedInitBytes<B>
where
    B: AlignedBytes + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl<B> AsMut<[u8]> for AlignedInitBytes<B>
where
    B: AlignedBytes + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self
    }
}
