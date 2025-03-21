use alloc::boxed::Box;
use core::{
    borrow::Borrow,
    fmt::{self, Debug},
    marker::PhantomData,
    ptr,
};

use crate::traits::Soa;

use super::{
    assert::{assert_layouts, assert_same_len, validate_layout},
    field::{ErasedFieldSlicePtr, ErasedFieldSlicePtrIter},
    ErasedSoaPtrs,
};

pub struct ErasedSoaSlicePtrs<Fields> {
    len: usize,
    slices: Box<[ErasedFieldSlicePtr]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaSlicePtrs<Fields> {
    #[inline]
    pub fn new<I>(len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSlicePtr>,
    {
        Self {
            len,
            slices: slices
                .into_iter()
                .inspect(|slice| {
                    validate_layout::<Fields>(slice.descriptor().layout());
                    assert_same_len(len, slice.len());
                })
                .collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, slices: T::SlicePtrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let len = T::slice_ptrs_len(context, slices.clone());
        let ptrs = T::slice_ptrs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase(context, ptrs);
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .inspect(|desc| validate_layout::<Fields>(desc.borrow().layout()))
            .map(|desc| desc.borrow().clone());

        let slices = descriptors
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let len = desc.layout().size() * len;
                let slice = ptr::slice_from_raw_parts(ptr, len);
                ErasedFieldSlicePtr::new(desc, slice)
            })
            .collect();
        Self {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::SlicePtrs
    where
        T: Soa<Fields = Fields>,
    {
        let Self { slices, len, .. } = self;

        let descriptors: Box<[_]> = T::field_descriptors(context)
            .into_iter()
            .inspect(|desc| validate_layout::<Fields>(desc.borrow().layout()))
            .map(|desc| desc.borrow().clone())
            .collect();
        assert_eq!(slices.len(), descriptors.len());

        let ptrs = descriptors
            .iter()
            .zip(slices)
            .inspect(|(desc, slice)| assert_layouts(desc.layout(), slice.descriptor().layout()))
            .map(|(_, slice)| slice.as_ptr());
        let ptrs = T::ptrs_restore(context, ptrs);
        T::slices_from_raw_parts(context, ptrs, len)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn fields(&self) -> &[ErasedFieldSlicePtr] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }

    #[inline]
    pub fn fields_mut(&mut self) -> &mut [ErasedFieldSlicePtr] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }

    #[inline]
    pub fn into_fields(self) -> Box<[ErasedFieldSlicePtr]> {
        let Self { slices, .. } = self;
        slices
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaSlicePtrsIter<Fields> {
        let Self { slices, .. } = self;
        let slices = slices.iter().map(IntoIterator::into_iter);
        ErasedSoaSlicePtrsIter::new(slices)
    }
}

impl<Fields> Debug for ErasedSoaSlicePtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { len, slices, .. } = self;

        f.debug_struct("ErasedSoaSlicePtrs")
            .field("len", len)
            .field("slices", slices)
            .finish()
    }
}

impl<Fields> Clone for ErasedSoaSlicePtrs<Fields> {
    fn clone(&self) -> Self {
        let Self {
            len,
            slices,
            phantom,
        } = self;

        Self {
            len: len.clone(),
            slices: slices.clone(),
            phantom: phantom.clone(),
        }
    }
}

impl<Fields> IntoIterator for &ErasedSoaSlicePtrs<Fields> {
    type Item = ErasedSoaPtrs<Fields>;
    type IntoIter = ErasedSoaSlicePtrsIter<Fields>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Fields> IntoIterator for ErasedSoaSlicePtrs<Fields> {
    type Item = ErasedSoaPtrs<Fields>;
    type IntoIter = ErasedSoaSlicePtrsIter<Fields>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { slices, .. } = self;
        let slices = slices.into_vec().into_iter().map(IntoIterator::into_iter);
        ErasedSoaSlicePtrsIter::new(slices)
    }
}

pub struct ErasedSoaSlicePtrsIter<Fields> {
    slices: Box<[ErasedFieldSlicePtrIter]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaSlicePtrsIter<Fields> {
    #[inline]
    #[track_caller]
    pub(super) fn new<I>(slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSlicePtrIter>,
    {
        let mut slices = slices.into_iter().peekable();
        let len = slices
            .peek()
            .map(ExactSizeIterator::len)
            .expect("input slices should contain at least one field");

        Self {
            slices: slices
                .inspect(|iter| assert_same_len(len, iter.len()))
                .collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { slices, .. } = self;
        slices.iter().map(ExactSizeIterator::len).next().unwrap()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<Fields> Iterator for ErasedSoaSlicePtrsIter<Fields> {
    type Item = ErasedSoaPtrs<Fields>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if ErasedSoaSlicePtrsIter::is_empty(self) {
            return None;
        }

        let ptrs = self.slices.iter_mut().flat_map(Iterator::next);
        Some(ErasedSoaPtrs::new(ptrs))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<Fields> DoubleEndedIterator for ErasedSoaSlicePtrsIter<Fields> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if ErasedSoaSlicePtrsIter::is_empty(self) {
            return None;
        }

        let ptrs = self
            .slices
            .iter_mut()
            .flat_map(DoubleEndedIterator::next_back);
        Some(ErasedSoaPtrs::new(ptrs))
    }
}

impl<Fields> ExactSizeIterator for ErasedSoaSlicePtrsIter<Fields> {
    #[inline]
    fn len(&self) -> usize {
        ErasedSoaSlicePtrsIter::len(self)
    }
}
