use alloc::{boxed::Box, vec};
use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    slice,
};

use crate::traits::Soa;

use super::validate_layout;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ErasedFieldRefMut<'a> {
    layout: Layout,
    buffer: &'a mut [u8],
}

impl<'a> ErasedFieldRefMut<'a> {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: &'a mut [u8]) -> Self {
        let buffer_len = buffer.len();
        let layout_size = layout.size();
        assert!(
            buffer_len == layout_size,
            "buffer len {buffer_len} should match layout size {layout_size}",
        );

        let layout_align = layout.align();
        assert!(
            buffer.as_ptr().align_offset(layout_align) == 0,
            "buffer should be aligned to {layout_align}",
        );

        Self { layout, buffer }
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        let Self { buffer, .. } = self;
        buffer
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        let Self { buffer, .. } = self;
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, &'a mut [u8]) {
        let Self { layout, buffer } = self;
        (layout, buffer)
    }
}

pub struct ErasedSoaRefsMut<'a, Fields>
where
    Fields: 'a,
{
    pub(super) refs: Box<[ErasedFieldRefMut<'a>]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaRefsMut<'a, Fields> {
    #[inline]
    pub fn new<I>(refs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldRefMut<'a>>,
    {
        Self {
            refs: refs.into_iter().collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, refs: T::RefsMut<'a>) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::mut_refs_as_ptrs(context, refs);
        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let refs = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let data = ptr.cast();
                let len = field_layout.size();
                let r#ref = unsafe { slice::from_raw_parts_mut(data, len) };
                ErasedFieldRefMut::new(field_layout.clone(), r#ref)
            })
            .collect();
        Self {
            refs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::RefsMut<'a>
    where
        T: Soa<Fields = Fields>,
    {
        let Self { refs, .. } = self;

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>)
            .collect();
        assert_eq!(field_layouts.len(), refs.len());

        let ptrs = field_layouts
            .iter()
            .zip(refs)
            .map(|(field_layout, mut r#ref)| {
                assert_eq!(*field_layout, r#ref.layout());
                r#ref.buffer_mut().as_mut_ptr()
            });
        let ptrs = T::ptrs_restore_mut(context, ptrs);
        unsafe { T::ptrs_to_refs_mut(context, ptrs) }
    }
}

impl<'a, Fields> AsRef<[ErasedFieldRefMut<'a>]> for ErasedSoaRefsMut<'a, Fields> {
    fn as_ref(&self) -> &[ErasedFieldRefMut<'a>] {
        let Self { refs, .. } = self;
        refs.as_ref()
    }
}

impl<'a, Fields> AsMut<[ErasedFieldRefMut<'a>]> for ErasedSoaRefsMut<'a, Fields> {
    fn as_mut(&mut self) -> &mut [ErasedFieldRefMut<'a>] {
        let Self { refs, .. } = self;
        refs.as_mut()
    }
}

impl<'a, Fields> Debug for ErasedSoaRefsMut<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ErasedSoaRefsMut").field(&self.refs).finish()
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r ErasedSoaRefsMut<'a, Fields> {
    type Item = &'r ErasedFieldRefMut<'a>;

    type IntoIter = slice::Iter<'r, ErasedFieldRefMut<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefsMut { refs, .. } = self;
        refs.iter()
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r mut ErasedSoaRefsMut<'a, Fields> {
    type Item = &'r mut ErasedFieldRefMut<'a>;

    type IntoIter = slice::IterMut<'r, ErasedFieldRefMut<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefsMut { refs, .. } = self;
        refs.iter_mut()
    }
}

impl<'a, Fields> IntoIterator for ErasedSoaRefsMut<'a, Fields> {
    type Item = ErasedFieldRefMut<'a>;

    type IntoIter = vec::IntoIter<ErasedFieldRefMut<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefsMut { refs, .. } = self;
        refs.into_vec().into_iter()
    }
}

unsafe impl<'a, Fields> Send for ErasedSoaRefsMut<'a, Fields> where Fields: Send {}
unsafe impl<'a, Fields> Sync for ErasedSoaRefsMut<'a, Fields> where Fields: Sync {}
