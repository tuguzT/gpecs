use alloc::{boxed::Box, vec};
use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    slice,
};

use crate::traits::Soa;

use super::validate_layout;

type ErasedFieldRefMut<'a> = &'a mut [u8];

pub struct ErasedSoaRefsMut<'a, Fields>
where
    Fields: 'a,
{
    pub(super) refs: Box<[(Layout, ErasedFieldRefMut<'a>)]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaRefsMut<'a, Fields> {
    #[inline]
    pub fn new<I>(refs: I) -> Self
    where
        I: IntoIterator<Item = (Layout, ErasedFieldRefMut<'a>)>,
    {
        let refs = refs
            .into_iter()
            .map(|(field_layout, r#ref)| {
                assert_eq!(field_layout.size(), r#ref.len());
                (field_layout.clone(), r#ref)
            })
            .collect();
        Self {
            refs,
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
                (field_layout.clone(), r#ref)
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
            .map(|(field_layout, (layout, r#ref))| {
                assert_eq!(field_layout, &layout);
                r#ref.as_mut_ptr()
            });
        let ptrs = T::ptrs_restore_mut(context, ptrs);
        unsafe { T::ptrs_to_refs_mut(context, ptrs) }
    }
}

impl<'a, Fields> AsRef<[(Layout, ErasedFieldRefMut<'a>)]> for ErasedSoaRefsMut<'a, Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldRefMut<'a>)] {
        let Self { refs, .. } = self;
        refs.as_ref()
    }
}

impl<'a, Fields> AsMut<[(Layout, ErasedFieldRefMut<'a>)]> for ErasedSoaRefsMut<'a, Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldRefMut<'a>)] {
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
    type Item = &'r (Layout, ErasedFieldRefMut<'a>);

    type IntoIter = slice::Iter<'r, (Layout, ErasedFieldRefMut<'a>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefsMut { refs, .. } = self;
        refs.iter()
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r mut ErasedSoaRefsMut<'a, Fields> {
    type Item = &'r mut (Layout, ErasedFieldRefMut<'a>);

    type IntoIter = slice::IterMut<'r, (Layout, ErasedFieldRefMut<'a>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefsMut { refs, .. } = self;
        refs.iter_mut()
    }
}

impl<'a, Fields> IntoIterator for ErasedSoaRefsMut<'a, Fields> {
    type Item = (Layout, ErasedFieldRefMut<'a>);

    type IntoIter = vec::IntoIter<(Layout, ErasedFieldRefMut<'a>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefsMut { refs, .. } = self;
        refs.into_vec().into_iter()
    }
}

unsafe impl<'a, Fields> Send for ErasedSoaRefsMut<'a, Fields> where Fields: Send {}
unsafe impl<'a, Fields> Sync for ErasedSoaRefsMut<'a, Fields> where Fields: Sync {}
