use alloc::boxed::Box;
use core::{iter::FusedIterator, slice};

use crate::{
    assert::{check_same_layout, check_same_len},
    error::LenMismatchError,
    field::{ErasedFieldSliceIterMut, ErasedFieldSliceMut},
    soa::traits::Soa,
};

use super::{error::IntoValueError, ErasedSoaRefs, ErasedSoaRefsMut, ErasedSoaSlicesIter};

#[derive(Debug)]
pub struct ErasedSoaSlicesMut<'a> {
    len: usize,
    slices: Box<[ErasedFieldSliceMut<'a>]>,
}

impl<'a> ErasedSoaSlicesMut<'a> {
    #[inline]
    pub fn new<I>(len: usize, slices: I) -> Result<Self, LenMismatchError>
    where
        I: IntoIterator<Item = ErasedFieldSliceMut<'a>>,
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
        I: IntoIterator<Item = ErasedFieldSliceMut<'a>>,
    {
        if cfg!(debug_assertions) {
            return Self::new(len, slices).expect("incorrect inputs");
        }
        unsafe { Self::actual_new(len, slices) }
    }

    #[inline]
    unsafe fn actual_new<I>(len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSliceMut<'a>>,
    {
        let slices = slices.into_iter().collect();
        Self { len, slices }
    }

    #[inline]
    pub fn from<'context, T>(
        context: &'context T::Context,
        slices: T::SlicesMut<'context, 'a>,
    ) -> Self
    where
        T: Soa,
    {
        let len = T::slices_len_mut(context, &slices);
        let ptrs = T::mut_slice_refs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let slices = T::field_descriptors(context)
            .into_iter()
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let desc = desc.as_ref().clone();
                let buffer = unsafe { slice::from_raw_parts_mut(ptr, desc.layout().size() * len) };
                unsafe { ErasedFieldSliceMut::new_unchecked(desc, buffer, len) }
            });
        unsafe { Self::new_unchecked(len, slices) }
    }

    #[inline]
    pub unsafe fn into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::SlicesMut<'_, 'a>, IntoValueError<Self>>
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
        let ptrs = slices
            .into_vec()
            .into_iter()
            .map(|slice| slice.into_buffer().as_mut_ptr());

        let ptrs = T::ptrs_restore_mut(context, ptrs);
        let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
        let slices = unsafe { T::slice_ptrs_to_slices_mut(context, slices) };
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
    pub fn field_slices(&self) -> &[ErasedFieldSliceMut<'a>] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }

    #[inline]
    pub fn into_field_slices(self) -> Box<[ErasedFieldSliceMut<'a>]> {
        let Self { slices, .. } = self;
        slices
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaSlicesIter<'_> {
        let Self { slices, .. } = self;
        let slices = slices.iter().map(IntoIterator::into_iter);
        ErasedSoaSlicesIter::new(slices)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedSoaSlicesIterMut<'_> {
        let Self { slices, .. } = self;
        let slices = slices.iter_mut().map(IntoIterator::into_iter);
        ErasedSoaSlicesIterMut::new(slices)
    }
}

impl<'a> IntoIterator for &'a ErasedSoaSlicesMut<'_> {
    type Item = ErasedSoaRefs<'a>;
    type IntoIter = ErasedSoaSlicesIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut ErasedSoaSlicesMut<'_> {
    type Item = ErasedSoaRefsMut<'a>;
    type IntoIter = ErasedSoaSlicesIterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a> IntoIterator for ErasedSoaSlicesMut<'a> {
    type Item = ErasedSoaRefsMut<'a>;
    type IntoIter = ErasedSoaSlicesIterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { slices, .. } = self;
        let slices = slices.into_vec().into_iter().map(IntoIterator::into_iter);
        ErasedSoaSlicesIterMut::new(slices)
    }
}

pub struct ErasedSoaSlicesIterMut<'a> {
    slices: Box<[ErasedFieldSliceIterMut<'a>]>,
}

impl<'a> ErasedSoaSlicesIterMut<'a> {
    #[inline]
    fn new<I>(slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSliceIterMut<'a>>,
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

impl<'a> Iterator for ErasedSoaSlicesIterMut<'a> {
    type Item = ErasedSoaRefsMut<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if ErasedSoaSlicesIterMut::is_empty(self) {
            return None;
        }

        let refs = self.slices.iter_mut().flat_map(Iterator::next);
        Some(ErasedSoaRefsMut::new(refs))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl DoubleEndedIterator for ErasedSoaSlicesIterMut<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if ErasedSoaSlicesIterMut::is_empty(self) {
            return None;
        }

        let refs = self
            .slices
            .iter_mut()
            .flat_map(DoubleEndedIterator::next_back);
        Some(ErasedSoaRefsMut::new(refs))
    }
}

impl ExactSizeIterator for ErasedSoaSlicesIterMut<'_> {
    #[inline]
    fn len(&self) -> usize {
        ErasedSoaSlicesIterMut::len(self)
    }
}

impl FusedIterator for ErasedSoaSlicesIterMut<'_> {}
