use core::{
    fmt::{self, Debug},
    ptr,
};

use crate::{
    error::{check_align, check_layout, check_len},
    field::{
        ErasedFieldMutPtr, ErasedFieldRef,
        assert::check_into_layout,
        error::{ErasedFieldIntoValueError, ErasedFieldPtrError},
    },
    soa::field::FieldDescriptor,
};

#[derive(Clone, Copy)]
pub struct ErasedFieldPtr {
    desc: FieldDescriptor,
    ptr: *const u8,
}

impl ErasedFieldPtr {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: *const [u8]) -> Result<Self, ErasedFieldPtrError> {
        let ptr = buffer.cast();
        check_len(buffer.len(), desc.layout().size())?;
        check_align(ptr, desc.layout())?;

        Ok(Self { desc, ptr })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: *const [u8]) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer).expect("incorrect inputs");
        }

        let ptr = buffer.cast();
        Self { desc, ptr }
    }

    #[inline]
    pub fn dangling(desc: FieldDescriptor) -> Self {
        let data = ptr::without_provenance(desc.layout().align());
        let buffer = ptr::slice_from_raw_parts(data, desc.layout().size());
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedFieldMutPtr {
        let Self { desc, ptr } = self;
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast_mut(), desc.layout().size());
        unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { desc, ptr } = self;

        let size = desc.layout().size();
        let data = unsafe { ptr.add(count * size) };
        let buffer = ptr::slice_from_raw_parts(data, size);
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: Self) -> isize {
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
    pub unsafe fn deref<'a>(self) -> ErasedFieldRef<'a> {
        unsafe { ErasedFieldRef::from_ptr(self) }
    }

    #[inline]
    pub fn descriptor(self) -> FieldDescriptor {
        let Self { desc, .. } = self;
        desc
    }

    #[inline]
    pub fn as_buffer(self) -> *const [u8] {
        let Self { desc, ptr } = self;
        ptr::slice_from_raw_parts(ptr, desc.layout().size())
    }

    #[inline]
    pub fn as_ptr(self) -> *const u8 {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *const [u8]) {
        let Self { desc, ptr } = self;
        let buffer = ptr::slice_from_raw_parts(ptr, desc.layout().size());
        (desc, buffer)
    }
}

#[expect(clippy::missing_fields_in_debug, reason = "buffer instead of ptr")]
impl Debug for ErasedFieldPtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.desc;
        let buffer = &self.as_buffer();
        f.debug_struct("ErasedFieldPtr")
            .field("desc", desc)
            .field("buffer", buffer)
            .finish()
    }
}

impl<T> From<*const T> for ErasedFieldPtr {
    #[inline]
    fn from(ptr: *const T) -> Self {
        let desc = FieldDescriptor::of::<T>();
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), desc.layout().size());
        unsafe { Self::new_unchecked(desc, buffer) }
    }
}

impl<T> TryFrom<ErasedFieldPtr> for *const T {
    type Error = ErasedFieldIntoValueError<ErasedFieldPtr>;

    #[inline]
    fn try_from(value: ErasedFieldPtr) -> Result<Self, Self::Error> {
        let ErasedFieldPtr { desc, .. } = value;
        let value = check_into_layout::<T, _>(desc.layout(), value)?;

        let ptr = value.as_ptr().cast();
        Ok(ptr)
    }
}
