use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    iter,
    marker::PhantomData,
    slice,
};

use crate::traits::Soa;

use super::{
    assert::{assert_layouts, check_same_len, validate_layout},
    error::{ErasedSoaError, FromValueError, InvalidLayoutError},
    field::{ErasedFieldSliceIterMut, ErasedFieldSliceMut},
    ErasedSoaRefs, ErasedSoaRefsMut, ErasedSoaSlicesIter,
};

pub struct ErasedSoaSlicesMut<'a, Fields>
where
    Fields: 'a,
{
    len: usize,
    slices: Box<[ErasedFieldSliceMut<'a>]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaSlicesMut<'a, Fields> {
    #[inline]
    pub fn new<I>(len: usize, slices: I) -> Result<Self, ErasedSoaError>
    where
        I: IntoIterator<Item = ErasedFieldSliceMut<'a>>,
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
        Self {
            len,
            slices: slices.into_iter().collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(
        context: &T::Context,
        slices: T::SlicesMut<'a>,
    ) -> Result<Self, FromValueError<T::SlicesMut<'a>>>
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

        let len = T::slices_len_mut(context, &slices);
        let ptrs = T::mut_slice_refs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let slices = iter::zip(descriptors, ptrs).map(|(desc, ptr)| {
            let buffer = unsafe { slice::from_raw_parts_mut(ptr, desc.layout().size() * len) };
            unsafe { ErasedFieldSliceMut::new_unchecked(desc, buffer, len) }
        });
        let me = unsafe { Self::actual_new(len, slices) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::SlicesMut<'a>, FromValueError<Self>>
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
            Err(error) => return Err(FromValueError::new(self, error)),
        };

        let Self { slices, len, .. } = self;
        assert_eq!(slices.len(), descriptors.len());

        let ptrs = descriptors
            .iter()
            .zip(slices)
            .inspect(|(desc, slice)| assert_layouts(desc.layout(), slice.descriptor().layout()))
            .map(|(_, slice)| slice.into_buffer().as_mut_ptr());
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
    pub fn iter(&self) -> ErasedSoaSlicesIter<'_, Fields> {
        let Self { slices, .. } = self;
        let slices = slices.iter().map(IntoIterator::into_iter);
        ErasedSoaSlicesIter::new(slices)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedSoaSlicesIterMut<'_, Fields> {
        let Self { slices, .. } = self;
        let slices = slices.iter_mut().map(IntoIterator::into_iter);
        ErasedSoaSlicesIterMut::new(slices)
    }
}

impl<Fields> Debug for ErasedSoaSlicesMut<'_, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { len, slices, .. } = self;

        f.debug_struct("ErasedSoaSlicesMut")
            .field("len", len)
            .field("slices", slices)
            .finish()
    }
}

impl<'a, Fields> IntoIterator for &'a ErasedSoaSlicesMut<'_, Fields> {
    type Item = ErasedSoaRefs<'a, Fields>;
    type IntoIter = ErasedSoaSlicesIter<'a, Fields>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Fields> IntoIterator for &'a mut ErasedSoaSlicesMut<'_, Fields> {
    type Item = ErasedSoaRefsMut<'a, Fields>;
    type IntoIter = ErasedSoaSlicesIterMut<'a, Fields>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, Fields> IntoIterator for ErasedSoaSlicesMut<'a, Fields> {
    type Item = ErasedSoaRefsMut<'a, Fields>;
    type IntoIter = ErasedSoaSlicesIterMut<'a, Fields>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { slices, .. } = self;
        let slices = slices.into_vec().into_iter().map(IntoIterator::into_iter);
        ErasedSoaSlicesIterMut::new(slices)
    }
}

unsafe impl<Fields> Send for ErasedSoaSlicesMut<'_, Fields> where Fields: Send {}
unsafe impl<Fields> Sync for ErasedSoaSlicesMut<'_, Fields> where Fields: Sync {}

pub struct ErasedSoaSlicesIterMut<'a, Fields>
where
    Fields: 'a,
{
    slices: Box<[ErasedFieldSliceIterMut<'a>]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaSlicesIterMut<'a, Fields> {
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

impl<'a, Fields> Iterator for ErasedSoaSlicesIterMut<'a, Fields> {
    type Item = ErasedSoaRefsMut<'a, Fields>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if ErasedSoaSlicesIterMut::is_empty(self) {
            return None;
        }

        let refs = self.slices.iter_mut().flat_map(Iterator::next);
        let item = unsafe { ErasedSoaRefsMut::new_unchecked(refs) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<Fields> DoubleEndedIterator for ErasedSoaSlicesIterMut<'_, Fields> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if ErasedSoaSlicesIterMut::is_empty(self) {
            return None;
        }

        let refs = self
            .slices
            .iter_mut()
            .flat_map(DoubleEndedIterator::next_back);
        let item = unsafe { ErasedSoaRefsMut::new_unchecked(refs) };
        Some(item)
    }
}

impl<Fields> ExactSizeIterator for ErasedSoaSlicesIterMut<'_, Fields> {
    #[inline]
    fn len(&self) -> usize {
        ErasedSoaSlicesIterMut::len(self)
    }
}

unsafe impl<Fields> Send for ErasedSoaSlicesIterMut<'_, Fields> where Fields: Send {}
unsafe impl<Fields> Sync for ErasedSoaSlicesIterMut<'_, Fields> where Fields: Sync {}
