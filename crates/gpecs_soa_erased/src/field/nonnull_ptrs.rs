use core::{
    fmt::{self, Debug},
    ptr::{self, NonNull},
};

use crate::{
    error::{check_align, check_layout, check_len},
    field::{
        assert::check_into_layout,
        error::{ErasedFieldIntoValueError, ErasedFieldPtrError},
    },
    soa::field::FieldDescriptor,
};

#[derive(Clone, Copy)]
pub struct ErasedFieldNonNullPtr {
    desc: FieldDescriptor,
    ptr: NonNull<u8>,
}

impl ErasedFieldNonNullPtr {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: NonNull<[u8]>) -> Result<Self, ErasedFieldPtrError> {
        let ptr = buffer.cast();
        check_len(buffer.len(), desc.layout().size())?;
        check_align(ptr.as_ptr(), desc.layout())?;

        Ok(Self { desc, ptr })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: NonNull<[u8]>) -> Self {
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
        let buffer = unsafe { NonNull::new_unchecked(buffer) };
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { desc, ptr } = self;

        let size = desc.layout().size();
        let data = unsafe { ptr.add(count * size) };
        let buffer = ptr::slice_from_raw_parts_mut(data.as_ptr(), size);
        let buffer = unsafe { NonNull::new_unchecked(buffer) };
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
    #[track_caller]
    pub unsafe fn swap(self, with: Self) {
        let Self { desc, .. } = self;
        check_layout(with.descriptor().layout(), desc.layout()).expect("layouts should match");

        let a = self.as_ptr().as_ptr();
        let b = with.as_ptr().as_ptr();
        let count = desc.layout().size();
        for i in 0..count {
            unsafe { ptr::swap(a.add(i), b.add(i)) }
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from(self, from: Self, count: usize) {
        let Self { desc, .. } = self;
        check_layout(from.descriptor().layout(), desc.layout()).expect("layouts should match");

        let src = from.as_ptr();
        let dst = self.as_ptr();
        let count = count * desc.layout().size();
        unsafe { ptr::copy(src.as_ptr(), dst.as_ptr(), count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping(self, from: Self, count: usize) {
        let Self { desc, .. } = self;
        check_layout(from.descriptor().layout(), desc.layout()).expect("layouts should match");

        let count = count * desc.layout().size();
        let src = from.as_ptr();
        let dst = self.as_ptr();
        unsafe { ptr::copy_nonoverlapping(src.as_ptr(), dst.as_ptr(), count) }
    }

    #[inline]
    pub fn descriptor(self) -> FieldDescriptor {
        let Self { desc, .. } = self;
        desc
    }

    #[inline]
    pub fn as_buffer(self) -> NonNull<[u8]> {
        let Self { desc, ptr } = self;
        let ptr = ptr::slice_from_raw_parts_mut(ptr.as_ptr(), desc.layout().size());
        unsafe { NonNull::new_unchecked(ptr) }
    }

    #[inline]
    pub fn as_ptr(self) -> NonNull<u8> {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, NonNull<[u8]>) {
        let Self { desc, ptr } = self;
        let ptr = ptr::slice_from_raw_parts_mut(ptr.as_ptr(), desc.layout().size());
        let buffer = unsafe { NonNull::new_unchecked(ptr) };
        (desc, buffer)
    }
}

#[expect(clippy::missing_fields_in_debug, reason = "buffer instead of ptr")]
impl Debug for ErasedFieldNonNullPtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.desc;
        let buffer = &self.as_buffer();
        f.debug_struct("ErasedFieldNonNullPtr")
            .field("desc", desc)
            .field("buffer", buffer)
            .finish()
    }
}

impl<T> From<NonNull<T>> for ErasedFieldNonNullPtr {
    #[inline]
    fn from(ptr: NonNull<T>) -> Self {
        let desc = FieldDescriptor::of::<T>();
        let ptr = ptr::slice_from_raw_parts_mut(ptr.as_ptr().cast(), desc.layout().size());
        let buffer = unsafe { NonNull::new_unchecked(ptr) };
        unsafe { Self::new_unchecked(desc, buffer) }
    }
}

impl<T> TryFrom<ErasedFieldNonNullPtr> for NonNull<T> {
    type Error = ErasedFieldIntoValueError<ErasedFieldNonNullPtr>;

    #[inline]
    fn try_from(value: ErasedFieldNonNullPtr) -> Result<Self, Self::Error> {
        let ErasedFieldNonNullPtr { desc, .. } = value;
        let value = check_into_layout::<T, _>(desc.layout(), value)?;

        let ptr = value.as_ptr().cast();
        Ok(ptr)
    }
}
