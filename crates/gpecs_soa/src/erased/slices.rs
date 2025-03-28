use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    slice,
};

use crate::traits::Soa;

use super::{
    assert::{check_same_layout, check_same_len, validate_layout},
    error::{ErasedSoaError, FromValueError, IntoValueError, InvalidLayoutError},
    field::{ErasedFieldSlice, ErasedFieldSliceIter},
    ErasedSoaRefs,
};

pub struct ErasedSoaSlices<'a, Fields>
where
    Fields: 'a,
{
    len: usize,
    slices: Box<[ErasedFieldSlice<'a>]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaSlices<'a, Fields> {
    #[inline]
    pub fn new<I>(len: usize, slices: I) -> Result<Self, ErasedSoaError>
    where
        I: IntoIterator<Item = ErasedFieldSlice<'a>>,
    {
        let slices = slices
            .into_iter()
            .map(|slice| {
                validate_layout::<Fields>(slice.descriptor().layout())?;
                check_same_len(slice.len(), len)?;
                Ok(slice)
            })
            .collect::<Result<Box<[_]>, ErasedSoaError>>()?;
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
        Self {
            len,
            slices: slices.into_iter().collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(
        context: &T::Context,
        slices: T::Slices<'a>,
    ) -> Result<Self, FromValueError<T::Slices<'a>>>
    where
        T: Soa<Fields = Fields>,
    {
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .map(|desc| {
                validate_layout::<Fields>(desc.as_ref().layout())?;
                Ok(desc.as_ref().clone())
            })
            .collect::<Result<Box<[_]>, InvalidLayoutError>>();
        let descriptors = match descriptors {
            Ok(descriptors) => descriptors,
            Err(error) => return Err(FromValueError::new(slices, error)),
        };

        let len = T::slices_len(context, &slices);
        let ptrs = T::slice_refs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase(context, ptrs);
        let slices = descriptors
            .into_vec()
            .into_iter()
            .zip(ptrs)
            .map(|(desc, ptr)| {
                let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size() * len) };
                unsafe { ErasedFieldSlice::new_unchecked(desc, buffer, len) }
            });
        let me = unsafe { Self::actual_new(len, slices) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> Result<T::Slices<'a>, IntoValueError<Self>>
    where
        T: Soa<Fields = Fields>,
    {
        let Self { slices, .. } = &self;
        let result = T::field_descriptors(context)
            .into_iter()
            .zip(slices)
            .try_fold(0, |len, (desc, slice)| {
                validate_layout::<Fields>(desc.as_ref().layout())?;
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
    pub fn iter(&self) -> ErasedSoaSlicesIter<'_, Fields> {
        let Self { slices, .. } = self;
        let slices = slices.iter().map(IntoIterator::into_iter);
        ErasedSoaSlicesIter::new(slices)
    }
}

impl<Fields> Debug for ErasedSoaSlices<'_, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { len, slices, .. } = self;

        f.debug_struct("ErasedSoaSlices")
            .field("len", len)
            .field("slices", slices)
            .finish()
    }
}

impl<Fields> Clone for ErasedSoaSlices<'_, Fields> {
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

impl<'a, Fields> IntoIterator for &'a ErasedSoaSlices<'_, Fields> {
    type Item = ErasedSoaRefs<'a, Fields>;
    type IntoIter = ErasedSoaSlicesIter<'a, Fields>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Fields> IntoIterator for ErasedSoaSlices<'a, Fields> {
    type Item = ErasedSoaRefs<'a, Fields>;
    type IntoIter = ErasedSoaSlicesIter<'a, Fields>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { slices, .. } = self;
        let slices = slices.into_vec().into_iter().map(IntoIterator::into_iter);
        ErasedSoaSlicesIter::new(slices)
    }
}

unsafe impl<Fields> Send for ErasedSoaSlices<'_, Fields> where Fields: Sync {}
unsafe impl<Fields> Sync for ErasedSoaSlices<'_, Fields> where Fields: Sync {}

pub struct ErasedSoaSlicesIter<'a, Fields>
where
    Fields: 'a,
{
    slices: Box<[ErasedFieldSliceIter<'a>]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaSlicesIter<'a, Fields> {
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
        Self {
            slices,
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

impl<'a, Fields> Iterator for ErasedSoaSlicesIter<'a, Fields> {
    type Item = ErasedSoaRefs<'a, Fields>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if ErasedSoaSlicesIter::is_empty(self) {
            return None;
        }

        let refs = self.slices.iter_mut().flat_map(Iterator::next);
        let item = unsafe { ErasedSoaRefs::new_unchecked(refs) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<Fields> DoubleEndedIterator for ErasedSoaSlicesIter<'_, Fields> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if ErasedSoaSlicesIter::is_empty(self) {
            return None;
        }

        let refs = self
            .slices
            .iter_mut()
            .flat_map(DoubleEndedIterator::next_back);
        let item = unsafe { ErasedSoaRefs::new_unchecked(refs) };
        Some(item)
    }
}

impl<Fields> ExactSizeIterator for ErasedSoaSlicesIter<'_, Fields> {
    #[inline]
    fn len(&self) -> usize {
        ErasedSoaSlicesIter::len(self)
    }
}

unsafe impl<Fields> Send for ErasedSoaSlicesIter<'_, Fields> where Fields: Sync {}
unsafe impl<Fields> Sync for ErasedSoaSlicesIter<'_, Fields> where Fields: Sync {}
