use alloc::boxed::Box;
use core::{
    alloc::Layout,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ptr,
};

use crate::traits::Soa;

use super::{assert_buffer_align, assert_layout, assert_value_buffer_len, validate_layout};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ErasedFieldMutPtr {
    layout: Layout,
    buffer: *mut [u8],
}

impl ErasedFieldMutPtr {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: *mut [u8]) -> Self {
        assert_value_buffer_len(buffer.len(), layout.size());
        assert_buffer_align(buffer.cast(), layout.align());

        Self { layout, buffer }
    }

    #[inline]
    pub fn from<T>(ptr: *mut T) -> Self {
        let layout = Layout::new::<T>();
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast(), layout.size());
        Self::new(layout, buffer)
    }

    #[inline]
    #[track_caller]
    pub fn into<T>(self) -> *mut T {
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
    pub fn buffer(&self) -> *mut [u8] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut u8 {
        let Self { buffer, .. } = self;
        buffer.cast()
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, *mut [u8]) {
        let Self { layout, buffer } = self;
        (layout, buffer)
    }
}

pub struct ErasedSoaMutPtrs<Fields> {
    pub(super) ptrs: Box<[ErasedFieldMutPtr]>,
    pub(super) phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaMutPtrs<Fields> {
    #[inline]
    pub fn new<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldMutPtr>,
    {
        Self {
            ptrs: ptrs.into_iter().collect(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, ptrs: T::MutPtrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::ptrs_erase_mut(context, ptrs);
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let ptrs: Box<[_]> = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(ptr, len);
                ErasedFieldMutPtr::new(field_layout, ptr)
            })
            .collect();
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::MutPtrs
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
        T::ptrs_restore_mut(context, ptrs)
    }

    #[inline]
    pub fn fields(&self) -> &[ErasedFieldMutPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }

    #[inline]
    pub fn fields_mut(&mut self) -> &mut [ErasedFieldMutPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut()
    }

    #[inline]
    pub fn into_fields(self) -> Box<[ErasedFieldMutPtr]> {
        let Self { ptrs, .. } = self;
        ptrs
    }
}

impl<Fields> Debug for ErasedSoaMutPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_tuple("ErasedSoaMutPtrs").field(ptrs).finish()
    }
}

impl<Fields> PartialEq for ErasedSoaMutPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        let Self { ptrs, phantom } = self;
        *ptrs == other.ptrs && *phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaMutPtrs<Fields> {}

impl<Fields> Hash for ErasedSoaMutPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { ptrs, phantom } = self;
        ptrs.hash(state);
        phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaMutPtrs<Fields> {
    fn clone(&self) -> Self {
        let Self { ptrs, phantom } = self;
        Self {
            ptrs: ptrs.clone(),
            phantom: phantom.clone(),
        }
    }
}
