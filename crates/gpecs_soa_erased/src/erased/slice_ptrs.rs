use alloc::boxed::Box;
use core::ptr;

use crate::{
    assert::{check_same_layout, check_same_len},
    error::LenMismatchError,
    field::{ErasedFieldSlicePtr, ErasedFieldSlicePtrIter},
    soa::traits::Soa,
};

use super::{error::IntoValueError, ErasedSoaPtrs};

#[derive(Debug, Clone)]
pub struct ErasedSoaSlicePtrs {
    len: usize,
    slices: Box<[ErasedFieldSlicePtr]>,
}

impl ErasedSoaSlicePtrs {
    #[inline]
    pub fn new<I>(len: usize, slices: I) -> Result<Self, LenMismatchError>
    where
        I: IntoIterator<Item = ErasedFieldSlicePtr>,
    {
        let slices = slices
            .into_iter()
            .map(|slice| {
                check_same_len(slice.len(), len)?;
                Ok(slice)
            })
            .collect::<Result<Box<[_]>, _>>()?;
        let me = unsafe { Self::actual_new(len, slices) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked<I>(len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSlicePtr>,
    {
        if cfg!(debug_assertions) {
            return Self::new(len, slices).expect("incorrect inputs");
        }
        unsafe { Self::actual_new(len, slices) }
    }

    #[inline]
    unsafe fn actual_new<I>(len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSlicePtr>,
    {
        let slices = slices.into_iter().collect();
        Self { len, slices }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, slices: T::SlicePtrs) -> Self
    where
        T: Soa,
    {
        let len = T::slice_ptrs_len(context, &slices);
        let ptrs = T::slice_ptrs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase(context, ptrs);
        let slices = T::field_descriptors(context)
            .into_iter()
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let desc = desc.as_ref().clone();
                let buffer = ptr::slice_from_raw_parts(ptr, desc.layout().size() * len);
                unsafe { ErasedFieldSlicePtr::new_unchecked(desc, buffer, len) }
            });
        unsafe { Self::new_unchecked(len, slices) }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> Result<T::SlicePtrs, IntoValueError<Self>>
    where
        T: Soa,
    {
        let Self { slices, .. } = &self;
        let result = T::field_descriptors(context)
            .into_iter()
            .zip(slices)
            .try_fold(0, |len, (desc, slice)| {
                check_same_layout(slice.descriptor().layout(), desc.as_ref().layout())?;
                Ok(len + 1)
            })
            .and_then(|len| {
                check_same_len(len, slices.len())?;
                Ok(())
            });
        if let Err(error) = result {
            return Err(IntoValueError::new(self, error));
        }

        let Self { slices, len, .. } = self;
        let ptrs = slices.into_vec().into_iter().map(|slice| slice.as_ptr());

        let ptrs = T::ptrs_restore(context, ptrs);
        let slices = T::slices_from_raw_parts(context, ptrs, len);
        Ok(slices)
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
    pub fn field_slices(&self) -> &[ErasedFieldSlicePtr] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }

    #[inline]
    pub fn into_field_slices(self) -> Box<[ErasedFieldSlicePtr]> {
        let Self { slices, .. } = self;
        slices
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaSlicePtrsIter {
        let Self { slices, .. } = self;
        let slices = slices.iter().map(IntoIterator::into_iter);
        ErasedSoaSlicePtrsIter::new(slices)
    }
}

impl IntoIterator for &ErasedSoaSlicePtrs {
    type Item = ErasedSoaPtrs;
    type IntoIter = ErasedSoaSlicePtrsIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for ErasedSoaSlicePtrs {
    type Item = ErasedSoaPtrs;
    type IntoIter = ErasedSoaSlicePtrsIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { slices, .. } = self;
        let slices = slices.into_vec().into_iter().map(IntoIterator::into_iter);
        ErasedSoaSlicePtrsIter::new(slices)
    }
}

pub struct ErasedSoaSlicePtrsIter {
    slices: Box<[ErasedFieldSlicePtrIter]>,
}

impl ErasedSoaSlicePtrsIter {
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

        let slices = slices
            .inspect(|iter| {
                check_same_len(iter.len(), len).expect("input slices should have the same length")
            })
            .collect();
        Self { slices }
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

impl Iterator for ErasedSoaSlicePtrsIter {
    type Item = ErasedSoaPtrs;

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

impl DoubleEndedIterator for ErasedSoaSlicePtrsIter {
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

impl ExactSizeIterator for ErasedSoaSlicePtrsIter {
    #[inline]
    fn len(&self) -> usize {
        ErasedSoaSlicePtrsIter::len(self)
    }
}
