use core::{
    fmt::{self, Debug},
    ptr,
};

use crate::{
    error::{check_align, check_layout, check_len},
    field::{
        ErasedFieldPtr, ErasedFieldRef, ErasedFieldRefMut,
        assert::check_into_layout,
        error::{ErasedFieldIntoValueError, ErasedFieldPtrError},
    },
    soa::field::FieldDescriptor,
};

#[derive(Clone, Copy)]
pub struct ErasedFieldMutPtr {
    desc: FieldDescriptor,
    ptr: *mut u8,
}

impl ErasedFieldMutPtr {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: *mut [u8]) -> Result<Self, ErasedFieldPtrError> {
        let ptr = buffer.cast();
        check_len(buffer.len(), desc.layout().size())?;
        check_align(ptr, desc.layout())?;

        Ok(Self { desc, ptr })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: *mut [u8]) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer).expect("incorrect inputs");
        }

        let ptr = buffer.cast();
        Self { desc, ptr }
    }

    #[inline]
    pub fn dangling(desc: FieldDescriptor) -> Self {
        let data = ptr::without_provenance_mut(desc.layout().align());
        let buffer = ptr::slice_from_raw_parts_mut(data, desc.layout().size());
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn cast_const(self) -> ErasedFieldPtr {
        let Self { desc, ptr } = self;
        let buffer = ptr::slice_from_raw_parts(ptr.cast_const(), desc.layout().size());
        unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { desc, ptr } = self;

        let size = desc.layout().size();
        let data = unsafe { ptr.add(count * size) };
        let buffer = ptr::slice_from_raw_parts_mut(data, size);
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: ErasedFieldPtr) -> isize {
        let Self { desc, ptr } = self;
        check_layout(origin.descriptor().layout(), desc.layout()).expect("layouts should match");

        let offset = unsafe { ptr.offset_from(origin.as_ptr()) };
        let field_size = desc
            .layout()
            .size()
            .try_into()
            .expect("layout size should not exceed `isize::MAX`");
        offset
            .checked_div(field_size)
            .expect("erased field pointer should not be a ZST")
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap(self, with: Self) {
        let Self { desc, .. } = self;
        check_layout(with.descriptor().layout(), desc.layout()).expect("layouts should match");

        let a = self.as_mut_ptr();
        let b = with.as_mut_ptr();
        let count = desc.layout().size();
        for i in 0..count {
            unsafe { ptr::swap(a.add(i), b.add(i)) }
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from(self, from: ErasedFieldPtr, count: usize) {
        let Self { desc, .. } = self;
        check_layout(from.descriptor().layout(), desc.layout()).expect("layouts should match");

        let src = from.as_ptr();
        let dst = self.as_mut_ptr();
        let count = count * desc.layout().size();
        unsafe { ptr::copy(src, dst, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping(self, from: ErasedFieldPtr, count: usize) {
        let Self { desc, .. } = self;
        check_layout(from.descriptor().layout(), desc.layout()).expect("layouts should match");

        let src = from.as_ptr();
        let dst = self.as_mut_ptr();
        let count = count * desc.layout().size();
        unsafe { ptr::copy_nonoverlapping(src, dst, count) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldRef<'a> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedFieldRefMut<'a> {
        unsafe { ErasedFieldRefMut::from_ptr(self) }
    }

    #[inline]
    pub fn descriptor(self) -> FieldDescriptor {
        let Self { desc, .. } = self;
        desc
    }

    #[inline]
    pub fn as_buffer(self) -> *const [u8] {
        let Self { desc, ptr } = self;
        ptr::slice_from_raw_parts(ptr.cast_const(), desc.layout().size())
    }

    #[inline]
    pub fn as_mut_buffer(self) -> *mut [u8] {
        let Self { desc, ptr } = self;
        ptr::slice_from_raw_parts_mut(ptr, desc.layout().size())
    }

    #[inline]
    pub fn as_ptr(self) -> *const u8 {
        let Self { ptr, .. } = self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_mut_ptr(self) -> *mut u8 {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *mut [u8]) {
        let Self { desc, ptr } = self;
        let buffer = ptr::slice_from_raw_parts_mut(ptr, desc.layout().size());
        (desc, buffer)
    }
}

#[expect(clippy::missing_fields_in_debug, reason = "buffer instead of ptr")]
impl Debug for ErasedFieldMutPtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.desc;
        let buffer = &self.as_buffer();
        f.debug_struct("ErasedFieldMutPtr")
            .field("desc", desc)
            .field("buffer", buffer)
            .finish()
    }
}

impl<T> From<*mut T> for ErasedFieldMutPtr {
    #[inline]
    fn from(ptr: *mut T) -> Self {
        let desc = FieldDescriptor::of::<T>();
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast(), desc.layout().size());
        unsafe { Self::new_unchecked(desc, buffer) }
    }
}

impl<T> TryFrom<ErasedFieldMutPtr> for *mut T {
    type Error = ErasedFieldIntoValueError<ErasedFieldMutPtr>;

    #[inline]
    fn try_from(value: ErasedFieldMutPtr) -> Result<Self, Self::Error> {
        let ErasedFieldMutPtr { desc, .. } = value;
        let value = check_into_layout::<T, _>(desc.layout(), value)?;

        let ptr = value.as_mut_ptr().cast();
        Ok(ptr)
    }
}
