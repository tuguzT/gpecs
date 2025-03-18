use alloc::{boxed::Box, vec};
use core::{
    alloc::Layout,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ptr, slice,
};

use crate::traits::Soa;

use super::{assert_buffer_align, assert_layout, assert_value_buffer_len, validate_layout};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ErasedFieldPtr {
    layout: Layout,
    buffer: *const [u8],
}

impl ErasedFieldPtr {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: *const [u8]) -> Self {
        assert_value_buffer_len(buffer.len(), layout.size());
        assert_buffer_align(buffer.cast(), layout.align());

        Self { layout, buffer }
    }

    #[inline]
    pub fn from<T>(ptr: *const T) -> Self {
        let layout = Layout::new::<T>();
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), layout.size());
        Self::new(layout, buffer)
    }

    #[inline]
    #[track_caller]
    pub fn into<T>(self) -> *const T {
        let Self { layout, buffer } = self;
        assert_layout::<T>(&layout);

        buffer.cast()
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub fn buffer(&self) -> *const [u8] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { buffer, .. } = self;
        buffer.cast()
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, *const [u8]) {
        let Self { layout, buffer } = self;
        (layout, buffer)
    }
}

pub struct ErasedSoaPtrs<Fields> {
    pub(super) ptrs: Box<[ErasedFieldPtr]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaPtrs<Fields> {
    #[inline]
    pub fn new<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldPtr>,
    {
        Self {
            ptrs: ptrs.into_iter().collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, ptrs: T::Ptrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::ptrs_erase(context, ptrs);
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let ptrs = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size();
                let buffer = ptr::slice_from_raw_parts(ptr, len);
                ErasedFieldPtr::new(field_layout, buffer)
            })
            .collect();
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::Ptrs
    where
        T: Soa<Fields = Fields>,
    {
        let Self { ptrs, .. } = self;

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>)
            .collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .inspect(|(&field_layout, ptr)| assert_eq!(field_layout, ptr.layout()))
            .map(|(_, ptr)| ptr.as_ptr());
        T::ptrs_restore(context, ptrs)
    }
}

impl<Fields> AsRef<[ErasedFieldPtr]> for ErasedSoaPtrs<Fields> {
    fn as_ref(&self) -> &[ErasedFieldPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }
}

impl<Fields> AsMut<[ErasedFieldPtr]> for ErasedSoaPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [ErasedFieldPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut()
    }
}

impl<Fields> Debug for ErasedSoaPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_tuple("ErasedSoaPtrs").field(ptrs).finish()
    }
}

impl<Fields> PartialEq for ErasedSoaPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        let Self { ptrs, phantom } = self;
        *ptrs == other.ptrs && *phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaPtrs<Fields> {}

impl<Fields> Hash for ErasedSoaPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { ptrs, phantom } = self;
        ptrs.hash(state);
        phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaPtrs<Fields> {
    fn clone(&self) -> Self {
        let Self { ptrs, phantom } = self;
        Self {
            ptrs: ptrs.clone(),
            phantom: phantom.clone(),
        }
    }
}

impl<'a, Fields> IntoIterator for &'a ErasedSoaPtrs<Fields> {
    type Item = &'a ErasedFieldPtr;
    type IntoIter = slice::Iter<'a, ErasedFieldPtr>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaPtrs { ptrs, .. } = self;
        ptrs.iter()
    }
}

impl<'a, Fields> IntoIterator for &'a mut ErasedSoaPtrs<Fields> {
    type Item = &'a mut ErasedFieldPtr;
    type IntoIter = slice::IterMut<'a, ErasedFieldPtr>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaPtrs { ptrs, .. } = self;
        ptrs.iter_mut()
    }
}

impl<Fields> IntoIterator for ErasedSoaPtrs<Fields> {
    type Item = ErasedFieldPtr;
    type IntoIter = vec::IntoIter<ErasedFieldPtr>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaPtrs { ptrs, .. } = self;
        ptrs.into_vec().into_iter()
    }
}
