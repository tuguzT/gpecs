use alloc::boxed::Box;
use core::{iter::FusedIterator, slice};

use crate::{
    assert::{check_same_layout, check_same_len},
    error::LenMismatchError,
    field::{ErasedFieldSlice, ErasedFieldSliceIter},
    soa::traits::Soa,
};

use super::{error::IntoValueError, ErasedSoaRefs};

#[derive(Debug, Clone)]
pub struct ErasedSoaSlices<'a> {
    len: usize,
    slices: Box<[ErasedFieldSlice<'a>]>,
}

impl<'a> ErasedSoaSlices<'a> {
    #[inline]
    pub fn new<I>(len: usize, slices: I) -> Result<Self, LenMismatchError>
    where
        I: IntoIterator<Item = ErasedFieldSlice<'a>>,
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
        I: IntoIterator<Item = ErasedFieldSlice<'a>>,
    {
        if cfg!(debug_assertions) {
            return Self::new(len, slices).expect("incorrect inputs");
        }
        unsafe { Self::actual_new(len, slices) }
    }

    #[inline]
    unsafe fn actual_new<I>(len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSlice<'a>>,
    {
        let slices = slices.into_iter().collect();
        Self { len, slices }
    }

    #[inline]
    pub fn from<'context, T>(context: &'context T::Context, slices: T::Slices<'context, 'a>) -> Self
    where
        T: Soa,
    {
        let len = T::slices_len(context, &slices);
        let ptrs = T::slices_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase(context, ptrs);
        let slices = T::field_descriptors(context)
            .into_iter()
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let desc = desc.as_ref().clone();
                let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size() * len) };
                unsafe { ErasedFieldSlice::new_unchecked(desc, buffer, len) }
            });
        unsafe { Self::new_unchecked(len, slices) }
    }

    #[inline]
    pub unsafe fn into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::Slices<'_, 'a>, IntoValueError<Self>>
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
            .map(|slice| slice.into_buffer().as_ptr());

        let ptrs = T::ptrs_restore(context, ptrs);
        let slices = T::slices_from_raw_parts(context, ptrs, len);
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
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
    pub fn field_slices(&self) -> &[ErasedFieldSlice<'a>] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }

    #[inline]
    pub fn into_field_slices(self) -> Box<[ErasedFieldSlice<'a>]> {
        let Self { slices, .. } = self;
        slices
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaSlicesIter<'_> {
        let Self { slices, .. } = self;
        let slices = slices.iter().map(IntoIterator::into_iter);
        ErasedSoaSlicesIter::new(slices)
    }
}

impl<'a> IntoIterator for &'a ErasedSoaSlices<'_> {
    type Item = ErasedSoaRefs<'a>;
    type IntoIter = ErasedSoaSlicesIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for ErasedSoaSlices<'a> {
    type Item = ErasedSoaRefs<'a>;
    type IntoIter = ErasedSoaSlicesIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { slices, .. } = self;
        let slices = slices.into_vec().into_iter().map(IntoIterator::into_iter);
        ErasedSoaSlicesIter::new(slices)
    }
}

pub struct ErasedSoaSlicesIter<'a> {
    slices: Box<[ErasedFieldSliceIter<'a>]>,
}

impl<'a> ErasedSoaSlicesIter<'a> {
    #[inline]
    pub(super) fn new<I>(slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSliceIter<'a>>,
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

impl<'a> Iterator for ErasedSoaSlicesIter<'a> {
    type Item = ErasedSoaRefs<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if ErasedSoaSlicesIter::is_empty(self) {
            return None;
        }

        let refs = self.slices.iter_mut().flat_map(Iterator::next);
        Some(ErasedSoaRefs::new(refs))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl DoubleEndedIterator for ErasedSoaSlicesIter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if ErasedSoaSlicesIter::is_empty(self) {
            return None;
        }

        let refs = self
            .slices
            .iter_mut()
            .flat_map(DoubleEndedIterator::next_back);
        Some(ErasedSoaRefs::new(refs))
    }
}

impl ExactSizeIterator for ErasedSoaSlicesIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        ErasedSoaSlicesIter::len(self)
    }
}

impl FusedIterator for ErasedSoaSlicesIter<'_> {}
