use core::{
    fmt::{self, Debug},
    ptr::NonNull,
};

use crate::{ErasedSoaNonNullPtrs, ptr::slice::NonNullSliceItemPtr};

pub struct ErasedSoaNonNullSlicePtrs<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    len: usize,
    ptrs: ErasedSoaNonNullPtrs<D, P>,
}

impl<D, P> ErasedSoaNonNullSlicePtrs<D, P>
where
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub unsafe fn new_unchecked(
        layouts: D,
        buffer: NonNull<[P::Item]>,
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        let ptrs = unsafe { ErasedSoaNonNullPtrs::from_parts(layouts, buffer, capacity, offset) };
        unsafe { Self::from_ptrs(ptrs, len) }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedSoaNonNullPtrs<D, P>, len: usize) -> Self {
        Self { len, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, NonNull<[P::Item]>, usize, usize, usize) {
        let Self { ptrs, len } = self;
        let (layouts, buffer, capacity, offset) = ptrs.into_parts();
        (layouts, buffer, capacity, offset, len)
    }

    #[inline]
    pub unsafe fn map_layouts<N, F>(self, f: F) -> ErasedSoaNonNullSlicePtrs<N, P>
    where
        F: FnOnce(D) -> N,
    {
        let Self { ptrs, len } = self;

        let ptrs = unsafe { ptrs.map_layouts(f) };
        unsafe { ErasedSoaNonNullSlicePtrs::from_ptrs(ptrs, len) }
    }
}

impl<D, P> ErasedSoaNonNullSlicePtrs<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> NonNull<[P::Item]> {
        let Self { ptrs, .. } = self;
        ptrs.as_buffer()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.offset()
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { ptrs, .. } = self;
        ptrs.layouts()
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<D, P> Debug for ErasedSoaNonNullSlicePtrs<D, P>
where
    D: Debug + ?Sized,
    P: NonNullSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { len, ptrs } = self;
        f.debug_struct("ErasedSoaSliceNonNullPtrs")
            .field("len", len)
            .field("ptrs", &ptrs)
            .finish()
    }
}

impl<D, P> Clone for ErasedSoaNonNullSlicePtrs<D, P>
where
    D: Clone,
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { len, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { len, ptrs }
    }
}

impl<D, P> Copy for ErasedSoaNonNullSlicePtrs<D, P>
where
    D: Copy,
    P: NonNullSliceItemPtr,
{
}
