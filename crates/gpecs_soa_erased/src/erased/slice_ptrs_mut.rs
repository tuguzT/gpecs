use alloc::boxed::Box;
use core::ptr;

use crate::{
    assert::{check_same_layout, check_same_len},
    error::LenMismatchError,
    field::{ErasedFieldSliceMutPtr, ErasedFieldSliceMutPtrIter},
    soa::traits::Soa,
};

use super::{error::IntoValueError, ErasedSoaMutPtrs, ErasedSoaPtrs, ErasedSoaSlicePtrsIter};

#[derive(Debug, Clone)]
pub struct ErasedSoaSliceMutPtrs {
    len: usize,
    slices: Box<[ErasedFieldSliceMutPtr]>,
}

impl ErasedSoaSliceMutPtrs {
    #[inline]
    pub fn new<I>(len: usize, slices: I) -> Result<Self, LenMismatchError>
    where
        I: IntoIterator<Item = ErasedFieldSliceMutPtr>,
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
        I: IntoIterator<Item = ErasedFieldSliceMutPtr>,
    {
        if cfg!(debug_assertions) {
            return Self::new(len, slices).expect("incorrect inputs");
        }
        unsafe { Self::actual_new(len, slices) }
    }

    #[inline]
    unsafe fn actual_new<I>(len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSliceMutPtr>,
    {
        let slices = slices.into_iter().collect();
        Self { len, slices }
    }

    #[inline]
    pub fn from<'context, T>(
        context: &'context T::Context,
        slices: T::SliceMutPtrs<'context>,
    ) -> Self
    where
        T: Soa,
    {
        let len = T::slice_mut_ptrs_len(context, &slices);
        let ptrs = T::slice_mut_ptrs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let slices = T::field_descriptors(context)
            .into_iter()
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let desc = desc.as_ref().clone();
                let buffer = ptr::slice_from_raw_parts_mut(ptr, desc.layout().size() * len);
                unsafe { ErasedFieldSliceMutPtr::new_unchecked(desc, buffer, len) }
            });
        unsafe { Self::new_unchecked(len, slices) }
    }

    #[inline]
    pub unsafe fn into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::SliceMutPtrs<'_>, IntoValueError<Self>>
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

        let ptrs = T::ptrs_restore_mut(context, ptrs);
        let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
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
    pub fn field_slices(&self) -> &[ErasedFieldSliceMutPtr] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }

    #[inline]
    pub fn into_field_slices(self) -> Box<[ErasedFieldSliceMutPtr]> {
        let Self { slices, .. } = self;
        slices
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaSlicePtrsIter {
        let Self { slices, .. } = self;
        let slices = slices.iter().map(IntoIterator::into_iter);
        ErasedSoaSlicePtrsIter::new(slices)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedSoaSliceMutPtrsIter {
        let Self { slices, .. } = self;
        let slices = slices.iter_mut().map(IntoIterator::into_iter);
        ErasedSoaSliceMutPtrsIter::new(slices)
    }
}

impl IntoIterator for &ErasedSoaSliceMutPtrs {
    type Item = ErasedSoaPtrs;
    type IntoIter = ErasedSoaSlicePtrsIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for &mut ErasedSoaSliceMutPtrs {
    type Item = ErasedSoaMutPtrs;
    type IntoIter = ErasedSoaSliceMutPtrsIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl IntoIterator for ErasedSoaSliceMutPtrs {
    type Item = ErasedSoaMutPtrs;
    type IntoIter = ErasedSoaSliceMutPtrsIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { slices, .. } = self;
        let slices = slices.into_vec().into_iter().map(IntoIterator::into_iter);
        ErasedSoaSliceMutPtrsIter::new(slices)
    }
}

pub struct ErasedSoaSliceMutPtrsIter {
    slices: Box<[ErasedFieldSliceMutPtrIter]>,
}

impl ErasedSoaSliceMutPtrsIter {
    #[inline]
    #[track_caller]
    fn new<I>(slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSliceMutPtrIter>,
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

impl Iterator for ErasedSoaSliceMutPtrsIter {
    type Item = ErasedSoaMutPtrs;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if ErasedSoaSliceMutPtrsIter::is_empty(self) {
            return None;
        }

        let ptrs = self.slices.iter_mut().flat_map(Iterator::next);
        Some(ErasedSoaMutPtrs::new(ptrs))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl DoubleEndedIterator for ErasedSoaSliceMutPtrsIter {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if ErasedSoaSliceMutPtrsIter::is_empty(self) {
            return None;
        }

        let ptrs = self
            .slices
            .iter_mut()
            .flat_map(DoubleEndedIterator::next_back);
        Some(ErasedSoaMutPtrs::new(ptrs))
    }
}

impl ExactSizeIterator for ErasedSoaSliceMutPtrsIter {
    #[inline]
    fn len(&self) -> usize {
        ErasedSoaSliceMutPtrsIter::len(self)
    }
}
