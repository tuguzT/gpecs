use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    slice,
};

use alloc::{boxed::Box, vec};

use crate::traits::Soa;

use super::{assert_value_buffer_align, assert_value_buffer_len, validate_layout};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ErasedFieldRef<'a> {
    layout: Layout,
    buffer: &'a [u8],
}

impl<'a> ErasedFieldRef<'a> {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: &'a [u8]) -> Self {
        assert_value_buffer_len(buffer.len(), layout.size());
        assert_value_buffer_align(buffer.as_ptr(), layout.align());

        Self { layout, buffer }
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub fn buffer(&self) -> &'a [u8] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, &'a [u8]) {
        let Self { layout, buffer } = self;
        (layout, buffer)
    }
}

pub struct ErasedSoaRefs<'a, Fields>
where
    Fields: 'a,
{
    pub(super) refs: Box<[ErasedFieldRef<'a>]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaRefs<'a, Fields> {
    #[inline]
    pub fn new<I>(refs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldRef<'a>>,
    {
        Self {
            refs: refs.into_iter().collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, refs: T::Refs<'a>) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::refs_as_ptrs(context, refs);
        let ptrs = T::ptrs_erase(context, ptrs);
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let refs: Box<[_]> = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size();
                let buffer = unsafe { slice::from_raw_parts(ptr.cast(), len) };
                ErasedFieldRef::new(field_layout.clone(), buffer)
            })
            .collect();
        Self {
            refs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::Refs<'a>
    where
        T: Soa<Fields = Fields>,
    {
        let Self { refs, .. } = self;

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>)
            .collect();
        assert_eq!(field_layouts.len(), refs.len());

        let ptrs = field_layouts.iter().zip(refs).map(|(field_layout, r#ref)| {
            assert_eq!(*field_layout, r#ref.layout());
            r#ref.buffer().as_ptr()
        });
        let ptrs = T::ptrs_restore(context, ptrs);
        unsafe { T::ptrs_to_refs(context, ptrs) }
    }
}

impl<'a, Fields> AsRef<[ErasedFieldRef<'a>]> for ErasedSoaRefs<'a, Fields> {
    fn as_ref(&self) -> &[ErasedFieldRef<'a>] {
        let Self { refs, .. } = self;
        refs.as_ref()
    }
}

impl<'a, Fields> AsMut<[ErasedFieldRef<'a>]> for ErasedSoaRefs<'a, Fields> {
    fn as_mut(&mut self) -> &mut [ErasedFieldRef<'a>] {
        let Self { refs, .. } = self;
        refs.as_mut()
    }
}

impl<'a, Fields> Debug for ErasedSoaRefs<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { refs, .. } = self;
        f.debug_tuple("ErasedSoaRefs").field(refs).finish()
    }
}

impl<'a, Fields> Clone for ErasedSoaRefs<'a, Fields> {
    fn clone(&self) -> Self {
        let Self { refs, phantom } = self;
        Self {
            refs: refs.clone(),
            phantom: phantom.clone(),
        }
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r ErasedSoaRefs<'a, Fields> {
    type Item = &'r ErasedFieldRef<'a>;
    type IntoIter = slice::Iter<'r, ErasedFieldRef<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefs { refs, .. } = self;
        refs.iter()
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r mut ErasedSoaRefs<'a, Fields> {
    type Item = &'r mut ErasedFieldRef<'a>;
    type IntoIter = slice::IterMut<'r, ErasedFieldRef<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefs { refs, .. } = self;
        refs.iter_mut()
    }
}

impl<'a, Fields> IntoIterator for ErasedSoaRefs<'a, Fields> {
    type Item = ErasedFieldRef<'a>;
    type IntoIter = vec::IntoIter<ErasedFieldRef<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefs { refs, .. } = self;
        refs.into_vec().into_iter()
    }
}

unsafe impl<'a, Fields> Send for ErasedSoaRefs<'a, Fields> where Fields: Sync {}
unsafe impl<'a, Fields> Sync for ErasedSoaRefs<'a, Fields> where Fields: Sync {}
