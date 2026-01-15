use core::{
    alloc::Layout,
    error::Error,
    fmt::{self, Display},
    mem::MaybeUninit,
};

use crate::{
    error::{InsufficientLenError, NotAlignedError, check_align, check_sufficient_len},
    storage::AlignedSlice,
};

// TODO: support & test all the addressable units

pub struct AlignedUninitByteSlice<T>
where
    T: ?Sized,
{
    layout: Layout,
    inner: T,
}

impl<T> AlignedUninitByteSlice<T> {
    #[inline]
    pub unsafe fn new_unchecked(inner: T, layout: Layout) -> Self {
        Self { layout, inner }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        let Self { inner, .. } = self;
        inner
    }
}

impl<T> AlignedUninitByteSlice<T>
where
    T: AsRef<[MaybeUninit<u8>]>,
{
    #[inline]
    pub fn new(inner: T, layout: Layout) -> Result<Self, AlignedUninitByteSliceError> {
        let slice = inner.as_ref();
        check_align(slice.as_ptr().cast(), layout)?;
        check_sufficient_len(slice.len() * size_of::<u8>(), layout.size())?;

        let me = unsafe { Self::new_unchecked(inner, layout) };
        Ok(me)
    }
}

impl<T> AlignedUninitByteSlice<T>
where
    T: ?Sized,
{
    #[inline]
    pub fn as_inner(&self) -> &T {
        let Self { inner, .. } = self;
        inner
    }

    #[inline]
    pub fn as_mut_inner(&mut self) -> &mut T {
        let Self { inner, .. } = self;
        inner
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
        let Self { inner, .. } = self;
        inner.as_ref()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [MaybeUninit<u8>]
    where
        T: AsMut<[MaybeUninit<u8>]>,
    {
        let Self { inner, .. } = self;
        inner.as_mut()
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

unsafe impl<T> AlignedSlice<u8> for AlignedUninitByteSlice<T>
where
    T: AsRef<[MaybeUninit<u8>]> + AsMut<[MaybeUninit<u8>]> + ?Sized,
{
    #[inline]
    fn as_ptr(&self) -> *const u8 {
        let slice = self.as_slice();
        slice.as_ptr().cast()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        let slice = self.as_mut_slice();
        slice.as_mut_ptr().cast()
    }

    #[inline]
    fn layout(&self) -> Layout {
        Self::layout(self)
    }
}

#[derive(Debug, Clone)]
pub enum AlignedUninitByteSliceError {
    NotAligned(NotAlignedError),
    InsufficientLen(InsufficientLenError),
}

impl From<NotAlignedError> for AlignedUninitByteSliceError {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<InsufficientLenError> for AlignedUninitByteSliceError {
    #[inline]
    fn from(error: InsufficientLenError) -> Self {
        Self::InsufficientLen(error)
    }
}

impl Display for AlignedUninitByteSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAligned(e) => e.fmt(f),
            Self::InsufficientLen(e) => e.fmt(f),
        }
    }
}

impl Error for AlignedUninitByteSliceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotAligned(e) => Some(e),
            Self::InsufficientLen(e) => Some(e),
        }
    }
}
