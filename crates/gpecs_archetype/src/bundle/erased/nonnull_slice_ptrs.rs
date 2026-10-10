use core::{
    fmt::{self, Debug},
    ptr::NonNull,
};

use gpecs_soa_erased::{ErasedSoaNonNullSlicePtrs, ptr::slice::NonNullSliceItemPtr};

use crate::bundle::erased::ErasedBundleNonNullPtrs;

pub struct ErasedBundleNonNullSlicePtrs<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    inner: ErasedSoaNonNullSlicePtrs<D, P>,
}

impl<D, P> ErasedBundleNonNullSlicePtrs<D, P>
where
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_inner(inner: ErasedSoaNonNullSlicePtrs<D, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundleNonNullPtrs<D, P>, len: usize) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { ErasedSoaNonNullSlicePtrs::from_ptrs(inner, len) };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn into_inner(self) -> ErasedSoaNonNullSlicePtrs<D, P> {
        let Self { inner } = self;
        inner
    }
}

impl<D, P> ErasedBundleNonNullSlicePtrs<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> NonNull<[P::Item]> {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { inner } = self;
        inner.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { inner } = self;
        inner.offset()
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { inner } = self;
        inner.layouts()
    }
}

impl<D, P> Debug for ErasedBundleNonNullSlicePtrs<D, P>
where
    D: Debug + ?Sized,
    P: NonNullSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("ErasedBundleNonNullSlicePtrs")
            .field("inner", &inner)
            .finish()
    }
}

impl<D, P> Clone for ErasedBundleNonNullSlicePtrs<D, P>
where
    D: Clone,
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        unsafe { Self::from_inner(inner) }
    }
}

impl<D, P> Copy for ErasedBundleNonNullSlicePtrs<D, P>
where
    D: Copy,
    P: NonNullSliceItemPtr,
{
}
