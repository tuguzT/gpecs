use core::{
    alloc::Layout,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};

use alloc::{boxed::Box, vec};

use crate::traits::Soa;

use super::{assert_value_buffer_align, assert_value_buffer_len, validate_layout};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ErasedFieldNonNullPtr {
    layout: Layout,
    buffer: NonNull<[u8]>,
}

impl ErasedFieldNonNullPtr {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: NonNull<[u8]>) -> Self {
        assert_value_buffer_len(buffer.len(), layout.size());
        assert_value_buffer_align(buffer.as_ptr().cast(), layout.align());

        Self { layout, buffer }
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub fn buffer(&self) -> NonNull<[u8]> {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, NonNull<[u8]>) {
        let Self { layout, buffer } = self;
        (layout, buffer)
    }
}

pub struct ErasedSoaNonNullPtrs<Fields> {
    pub(super) ptrs: Box<[ErasedFieldNonNullPtr]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaNonNullPtrs<Fields> {
    #[inline]
    pub fn new<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldNonNullPtr>,
    {
        Self {
            ptrs: ptrs.into_iter().collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, ptrs: T::NonNullPtrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::nonnull_to_ptrs(context, ptrs);
        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let ptrs = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(ptr.cast(), len);
                let buffer = unsafe { NonNull::new_unchecked(ptr) };
                ErasedFieldNonNullPtr::new(field_layout.clone(), buffer)
            })
            .collect();
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::NonNullPtrs
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
            ptr.buffer().as_ptr().cast()
        });
        let ptrs = T::ptrs_restore_mut(context, ptrs);
        unsafe { T::ptrs_to_nonnull(context, ptrs) }
    }
}

impl<Fields> AsRef<[ErasedFieldNonNullPtr]> for ErasedSoaNonNullPtrs<Fields> {
    fn as_ref(&self) -> &[ErasedFieldNonNullPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }
}

impl<Fields> AsMut<[ErasedFieldNonNullPtr]> for ErasedSoaNonNullPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [ErasedFieldNonNullPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut()
    }
}

impl<Fields> Debug for ErasedSoaNonNullPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_tuple("ErasedSoaNonNullPtrs").field(ptrs).finish()
    }
}

impl<Fields> PartialEq for ErasedSoaNonNullPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        let Self { ptrs, phantom } = self;
        *ptrs == other.ptrs && *phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaNonNullPtrs<Fields> {}

impl<Fields> Hash for ErasedSoaNonNullPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { ptrs, phantom } = self;
        ptrs.hash(state);
        phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaNonNullPtrs<Fields> {
    fn clone(&self) -> Self {
        let Self { ptrs, phantom } = self;
        Self {
            ptrs: ptrs.clone(),
            phantom: phantom.clone(),
        }
    }
}

impl<'a, Fields> IntoIterator for &'a ErasedSoaNonNullPtrs<Fields> {
    type Item = &'a ErasedFieldNonNullPtr;
    type IntoIter = slice::Iter<'a, ErasedFieldNonNullPtr>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaNonNullPtrs { ptrs, .. } = self;
        ptrs.iter()
    }
}

impl<'a, Fields> IntoIterator for &'a mut ErasedSoaNonNullPtrs<Fields> {
    type Item = &'a mut ErasedFieldNonNullPtr;
    type IntoIter = slice::IterMut<'a, ErasedFieldNonNullPtr>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaNonNullPtrs { ptrs, .. } = self;
        ptrs.iter_mut()
    }
}

impl<Fields> IntoIterator for ErasedSoaNonNullPtrs<Fields> {
    type Item = ErasedFieldNonNullPtr;
    type IntoIter = vec::IntoIter<ErasedFieldNonNullPtr>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaNonNullPtrs { ptrs, .. } = self;
        ptrs.into_vec().into_iter()
    }
}
