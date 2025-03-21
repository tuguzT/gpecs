use alloc::boxed::Box;
use core::{
    alloc::Layout,
    fmt::{self, Debug},
    mem::ManuallyDrop,
    ptr, slice,
};

use super::{
    super::{assert::validate_layout, byte::ErasedByte},
    assert::{assert_layout, assert_value_buffer_len},
    ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldRef, ErasedFieldRefMut,
};

pub struct ErasedField<Fields> {
    layout: Layout,
    buffer: Box<[ErasedByte<Fields>]>,
}

impl<Fields> ErasedField<Fields> {
    #[inline]
    #[track_caller]
    pub fn new(layout: Layout, buffer: &[u8]) -> Self {
        validate_layout::<Fields>(layout);
        assert_value_buffer_len(buffer.len(), layout.size());

        let buffer_len = buffer.len().div_ceil(size_of::<ErasedByte<Fields>>());
        let mut r#box = Box::new_uninit_slice(buffer_len);
        unsafe {
            ptr::copy_nonoverlapping(buffer.as_ptr(), r#box.as_mut_ptr().cast(), layout.size());
        }

        Self {
            layout,
            buffer: unsafe { r#box.assume_init() },
        }
    }

    #[inline]
    pub fn from<T>(value: T) -> Self {
        let value = ManuallyDrop::new(value); // TODO: dispose of that value later with drop fn
        let layout = Layout::new::<T>();
        let buffer = unsafe { slice::from_raw_parts(ptr::from_ref(&value).cast(), layout.size()) };
        Self::new(layout, buffer)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self) -> T {
        let Self { layout, buffer } = self;
        assert_layout::<T>(layout);

        unsafe { ptr::read(buffer.as_ptr().cast()) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn cast<T>(&self) -> &T {
        let Self { layout, buffer } = self;
        assert_layout::<T>(layout);

        let ptr = buffer.as_ptr().cast();
        unsafe { &*ptr }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn cast_mut<T>(&mut self) -> &mut T {
        let Self { layout, buffer } = self;
        assert_layout::<T>(layout);

        let ptr = buffer.as_mut_ptr().cast();
        unsafe { &mut *ptr }
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        let Self { layout, buffer } = self;

        let data = buffer.as_ptr().cast();
        unsafe { slice::from_raw_parts(data, layout.size()) }
    }

    #[inline]
    pub fn as_field_ref(&self) -> ErasedFieldRef<'_> {
        let Self { layout, .. } = *self;
        let buffer = self.buffer();
        ErasedFieldRef::new(layout, buffer)
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { buffer, .. } = self;
        buffer.as_ptr().cast()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { layout, .. } = *self;
        let buffer = ptr::from_ref(self.buffer());
        ErasedFieldPtr::new(layout, buffer)
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        let Self { layout, buffer } = self;

        let data = buffer.as_mut_ptr().cast();
        unsafe { slice::from_raw_parts_mut(data, layout.size()) }
    }

    #[inline]
    pub fn as_field_ref_mut(&mut self) -> ErasedFieldRefMut<'_> {
        let Self { layout, .. } = *self;
        let buffer = self.buffer_mut();
        ErasedFieldRefMut::new(layout, buffer)
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { buffer, .. } = self;
        buffer.as_mut_ptr().cast()
    }

    #[inline]
    pub fn as_field_ptr_mut(&mut self) -> ErasedFieldMutPtr {
        let Self { layout, .. } = *self;
        let buffer = ptr::from_mut(self.buffer_mut());
        ErasedFieldMutPtr::new(layout, buffer)
    }

    #[inline]
    pub fn into_buffer(self) -> Box<[u8]> {
        let (_, buffer) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, Box<[u8]>) {
        let Self { layout, buffer } = self;

        let data = buffer.as_ptr().cast();
        let buffer = unsafe { slice::from_raw_parts(data, layout.size()) };
        (layout, buffer.into())
    }
}

impl<Fields> Debug for ErasedField<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { layout, .. } = self;
        let buffer = self.buffer();
        f.debug_struct("ErasedField")
            .field("layout", layout)
            .field("buffer", &buffer)
            .finish()
    }
}

impl<Fields> PartialEq for ErasedField<Fields> {
    fn eq(&self, other: &Self) -> bool {
        let Self { layout, .. } = self;
        *layout == other.layout() && self.buffer() == other.buffer()
    }
}

impl<Fields> Eq for ErasedField<Fields> {}
