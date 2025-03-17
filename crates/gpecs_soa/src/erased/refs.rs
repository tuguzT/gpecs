use core::{
    alloc::Layout,
    fmt::{self, Debug},
    marker::PhantomData,
    slice,
};

use alloc::{boxed::Box, vec};

use crate::traits::Soa;

use super::validate_layout;

type ErasedFieldRef<'a> = &'a [u8];

pub struct ErasedSoaRefs<'a, Fields>
where
    Fields: 'a,
{
    pub(super) refs: Box<[(Layout, ErasedFieldRef<'a>)]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaRefs<'a, Fields> {
    #[inline]
    pub fn new<I>(refs: I) -> Self
    where
        I: IntoIterator<Item = (Layout, ErasedFieldRef<'a>)>,
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
            .map(|(field_layout, ptr)| unsafe {
                let len = field_layout.size();
                (field_layout.clone(), slice::from_raw_parts(ptr.cast(), len))
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

        let ptrs = field_layouts
            .iter()
            .zip(refs)
            .map(|(field_layout, (layout, r#ref))| {
                assert_eq!(field_layout, &layout);
                r#ref.as_ptr()
            });
        let ptrs = T::ptrs_restore(context, ptrs);
        unsafe { T::ptrs_to_refs(context, ptrs) }
    }
}

impl<'a, Fields> AsRef<[(Layout, ErasedFieldRef<'a>)]> for ErasedSoaRefs<'a, Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldRef<'a>)] {
        let Self { refs, .. } = self;
        refs.as_ref()
    }
}

impl<'a, Fields> AsMut<[(Layout, ErasedFieldRef<'a>)]> for ErasedSoaRefs<'a, Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldRef<'a>)] {
        let Self { refs, .. } = self;
        refs.as_mut()
    }
}

impl<'a, Fields> Debug for ErasedSoaRefs<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ErasedSoaRefs").field(&self.refs).finish()
    }
}

impl<'a, Fields> Clone for ErasedSoaRefs<'a, Fields> {
    fn clone(&self) -> Self {
        Self {
            refs: self.refs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r ErasedSoaRefs<'a, Fields> {
    type Item = &'r (Layout, ErasedFieldRef<'a>);

    type IntoIter = slice::Iter<'r, (Layout, ErasedFieldRef<'a>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefs { refs, .. } = self;
        refs.iter()
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r mut ErasedSoaRefs<'a, Fields> {
    type Item = &'r mut (Layout, ErasedFieldRef<'a>);

    type IntoIter = slice::IterMut<'r, (Layout, ErasedFieldRef<'a>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefs { refs, .. } = self;
        refs.iter_mut()
    }
}

impl<'a, Fields> IntoIterator for ErasedSoaRefs<'a, Fields> {
    type Item = (Layout, ErasedFieldRef<'a>);

    type IntoIter = vec::IntoIter<(Layout, ErasedFieldRef<'a>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefs { refs, .. } = self;
        refs.into_vec().into_iter()
    }
}

unsafe impl<'a, Fields> Send for ErasedSoaRefs<'a, Fields> where Fields: Sync {}
unsafe impl<'a, Fields> Sync for ErasedSoaRefs<'a, Fields> where Fields: Sync {}
