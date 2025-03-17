use core::{
    alloc::Layout,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ptr, slice,
};

use alloc::{boxed::Box, vec};

use crate::traits::Soa;

use super::validate_layout;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ErasedFieldPtr {
    layout: Layout,
    buffer: *const [u8],
}

impl ErasedFieldPtr {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: *const [u8]) -> Self {
        // TODO: return checks when the source of wrong pointer arithmetic is found
        // let buffer_len = buffer.len();
        // let layout_size = layout.size();
        // assert!(
        //     buffer_len == layout_size,
        //     "buffer len {buffer_len} should match layout size {layout_size}",
        // );

        // let layout_align = layout.align();
        // assert!(
        //     buffer.cast::<u8>().align_offset(layout_align) == 0,
        //     "buffer should be aligned to {layout_align}",
        // );

        Self { layout, buffer }
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
                let ptr = ptr::slice_from_raw_parts(ptr.cast(), len);
                ErasedFieldPtr::new(field_layout, ptr)
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

        let ptrs = field_layouts.iter().zip(ptrs).map(|(field_layout, ptr)| {
            assert_eq!(*field_layout, ptr.layout());
            ptr.buffer().cast()
        });
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
        f.debug_tuple("ErasedSoaPtrs").field(&self.ptrs).finish()
    }
}

impl<Fields> PartialEq for ErasedSoaPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.ptrs == other.ptrs && self.phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaPtrs<Fields> {}

impl<Fields> Hash for ErasedSoaPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.ptrs.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaPtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            ptrs: self.ptrs.clone(),
            phantom: self.phantom.clone(),
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
