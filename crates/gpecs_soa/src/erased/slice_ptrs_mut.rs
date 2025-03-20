use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ptr,
};

use crate::traits::Soa;

use super::{
    assert::{assert_same_len, validate_layout},
    field::{ErasedFieldSliceMutPtr, ErasedFieldSliceMutPtrIter},
    ErasedSoaMutPtrs, ErasedSoaPtrs, ErasedSoaSlicePtrsIter,
};

pub struct ErasedSoaSliceMutPtrs<Fields> {
    len: usize,
    slices: Box<[ErasedFieldSliceMutPtr]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaSliceMutPtrs<Fields> {
    #[inline]
    pub fn new<I>(len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSliceMutPtr>,
    {
        let slices = slices
            .into_iter()
            .inspect(|slice| {
                validate_layout::<Fields, _>(slice.layout());
                assert_same_len(len, slice.len());
            })
            .collect();
        Self {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, slices: T::SliceMutPtrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let len = T::slice_ptrs_len_mut(context, slices.clone());
        let ptrs = T::mut_slice_ptrs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let slices = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size() * len;
                let slice = ptr::slice_from_raw_parts_mut(ptr, len);
                ErasedFieldSliceMutPtr::new(field_layout, slice)
            })
            .collect();
        Self {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::SliceMutPtrs
    where
        T: Soa<Fields = Fields>,
    {
        let Self { slices, len, .. } = self;

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>)
            .collect();
        assert_eq!(slices.len(), field_layouts.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .inspect(|(&field_layout, slice)| assert_eq!(field_layout, slice.layout()))
            .map(|(_, slice)| slice.as_ptr());
        let ptrs = T::ptrs_restore_mut(context, ptrs);
        T::slices_from_raw_parts_mut(context, ptrs, len)
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
    pub fn fields(&self) -> &[ErasedFieldSliceMutPtr] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }

    #[inline]
    pub fn fields_mut(&mut self) -> &mut [ErasedFieldSliceMutPtr] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }

    #[inline]
    pub fn into_fields(self) -> Box<[ErasedFieldSliceMutPtr]> {
        let Self { slices, .. } = self;
        slices
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaSlicePtrsIter<Fields> {
        let Self { slices, .. } = self;
        let slices = slices.iter().map(IntoIterator::into_iter);
        ErasedSoaSlicePtrsIter::new(slices)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedSoaSliceMutPtrsIter<Fields> {
        let Self { slices, .. } = self;
        let slices = slices.iter_mut().map(IntoIterator::into_iter);
        ErasedSoaSliceMutPtrsIter::new(slices)
    }
}

impl<Fields> Debug for ErasedSoaSliceMutPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { len, slices, .. } = self;

        f.debug_struct("ErasedSoaSliceMutPtrs")
            .field("len", len)
            .field("slices", slices)
            .finish()
    }
}

impl<Fields> PartialEq for ErasedSoaSliceMutPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            len,
            slices,
            phantom,
        } = self;

        *len == other.len && *slices == other.slices && *phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaSliceMutPtrs<Fields> {}

impl<Fields> Hash for ErasedSoaSliceMutPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self {
            len,
            slices,
            phantom,
        } = self;

        len.hash(state);
        slices.hash(state);
        phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaSliceMutPtrs<Fields> {
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

impl<Fields> IntoIterator for &ErasedSoaSliceMutPtrs<Fields> {
    type Item = ErasedSoaPtrs<Fields>;
    type IntoIter = ErasedSoaSlicePtrsIter<Fields>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Fields> IntoIterator for &mut ErasedSoaSliceMutPtrs<Fields> {
    type Item = ErasedSoaMutPtrs<Fields>;
    type IntoIter = ErasedSoaSliceMutPtrsIter<Fields>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<Fields> IntoIterator for ErasedSoaSliceMutPtrs<Fields> {
    type Item = ErasedSoaMutPtrs<Fields>;
    type IntoIter = ErasedSoaSliceMutPtrsIter<Fields>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { slices, .. } = self;
        let slices = slices.into_vec().into_iter().map(IntoIterator::into_iter);
        ErasedSoaSliceMutPtrsIter::new(slices)
    }
}

pub struct ErasedSoaSliceMutPtrsIter<Fields> {
    slices: Box<[ErasedFieldSliceMutPtrIter]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaSliceMutPtrsIter<Fields> {
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

        Self {
            #[allow(dropping_copy_types)]
            slices: slices
                .inspect(|iter| drop(assert_same_len(len, iter.len())))
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

impl<Fields> Iterator for ErasedSoaSliceMutPtrsIter<Fields> {
    type Item = ErasedSoaMutPtrs<Fields>;

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

impl<Fields> DoubleEndedIterator for ErasedSoaSliceMutPtrsIter<Fields> {
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

impl<Fields> ExactSizeIterator for ErasedSoaSliceMutPtrsIter<Fields> {
    #[inline]
    fn len(&self) -> usize {
        ErasedSoaSliceMutPtrsIter::len(self)
    }
}
