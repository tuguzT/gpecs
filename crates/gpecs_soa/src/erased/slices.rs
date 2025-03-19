use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    slice,
};

use crate::traits::Soa;

use super::{
    assert::validate_layout,
    field::{ErasedFieldSlice, ErasedFieldSliceIter},
    ErasedSoaRefs,
};

pub struct ErasedSoaSlices<'a, Fields>
where
    Fields: 'a,
{
    pub(super) len: usize,
    pub(super) slices: Box<[ErasedFieldSlice<'a>]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaSlices<'a, Fields> {
    #[inline]
    pub fn new<I>(len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSlice<'a>>,
    {
        Self {
            len,
            slices: slices
                .into_iter()
                .inspect(|slice| assert_eq!(slice.len(), len))
                .collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, slices: T::Slices<'a>) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let len = T::slices_len(context, &slices);
        let ptrs = T::slice_refs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase(context, ptrs);
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let slices = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size() * len;
                let slice = unsafe { slice::from_raw_parts(ptr, len) };
                ErasedFieldSlice::new(field_layout, slice)
            })
            .collect();
        Self {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::Slices<'a>
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
            .map(|(_, slice)| slice.into_buffer().as_ptr());
        let ptrs = T::ptrs_restore(context, ptrs);
        let slices = T::slices_from_raw_parts(context, ptrs, len);
        unsafe { T::slice_ptrs_to_slices(context, slices) }
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
    pub fn fields(&self) -> &[ErasedFieldSlice<'a>] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }

    #[inline]
    pub fn fields_mut(&mut self) -> &mut [ErasedFieldSlice<'a>] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }

    #[inline]
    pub fn into_fields(self) -> Box<[ErasedFieldSlice<'a>]> {
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
        Self {
            slices: slices.into_iter().collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { slices, .. } = self;
        let mut lens = slices.iter().map(ExactSizeIterator::len);

        let first = lens.next().expect("SoA should contain at least one field");
        assert!(lens.all(|len| len == first));
        first
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
        Some(ErasedSoaRefs::new(refs))
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
        Some(ErasedSoaRefs::new(refs))
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
