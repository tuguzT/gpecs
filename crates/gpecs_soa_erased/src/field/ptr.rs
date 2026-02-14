use core::{alloc::Layout, mem::MaybeUninit, ops::Range, ptr};

use crate::{
    bytes_to_items::item_count,
    error::{InsufficientAlignError, check_len, check_ptr_align, check_sufficient_align},
    field::{
        ErasedFieldMutPtr, ErasedFieldRef,
        error::{ErasedFieldIntoValueError, ErasedFieldPtrError, check_into_layout},
    },
    slice_item_ptr::{CastMutPtr, ConstSliceItemPtr},
    soa::field::FieldDescriptor,
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldPtr<T> {
    desc: FieldDescriptor,
    ptr: T,
}

impl<T> ErasedFieldPtr<T> {
    #[inline]
    pub unsafe fn from_parts(desc: FieldDescriptor, ptr: T) -> Self {
        Self { desc, ptr }
    }

    #[inline]
    pub fn descriptor(self) -> FieldDescriptor {
        let Self { desc, .. } = self;
        desc
    }

    #[inline]
    pub fn ptr(self) -> T {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, T) {
        let Self { desc, ptr } = self;
        (desc, ptr)
    }
}

impl<T> ErasedFieldPtr<T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    pub fn cast_mut(self) -> ErasedFieldMutPtr<CastMutPtr<T>> {
        let Self { desc, ptr } = self;
        let ptr = ptr.cast_mut();
        unsafe { ErasedFieldMutPtr::from_parts(desc, ptr) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldRef<'a, T> {
        unsafe { ErasedFieldRef::from_ptr(self) }
    }
}

impl<T, U> ErasedFieldPtr<T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub fn new(desc: FieldDescriptor, buffer: *const [U]) -> Result<Self, ErasedFieldPtrError> {
        check_sufficient_align(desc.layout(), Layout::new::<U>())?;
        check_len(buffer.len() * size_of::<U>(), desc.layout().size())?;
        check_ptr_align(buffer.cast(), desc.layout())?;

        let buffer = ptr::slice_from_raw_parts(buffer.cast(), buffer.len());
        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(desc, ptr) };
        Ok(me)
    }

    #[inline]
    pub fn dangling(desc: FieldDescriptor) -> Result<Self, InsufficientAlignError> {
        check_sufficient_align(desc.layout(), Layout::new::<U>())?;

        let data = ptr::without_provenance(desc.layout().align());
        let buffer = ptr::slice_from_raw_parts(data, 0);
        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(desc, ptr) };
        Ok(me)
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { desc, ptr } = self;

        let count = count * item_count::<U>(desc);
        let ptr = unsafe { ptr.add(count) };
        unsafe { Self::from_parts(desc, ptr) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: Self) -> isize {
        let Self { desc, ptr } = self;

        let offset = unsafe { ptr.offset_from(origin.ptr()) };
        let len = item_count::<U>(desc).cast_signed();
        offset
            .checked_div(len)
            .expect("erased field pointer should not be a ZST")
    }

    #[inline]
    pub fn as_uninit_buffer(self) -> *const [MaybeUninit<U>] {
        let Self { ptr, .. } = self;
        ptr.slice()
    }

    #[inline]
    pub fn byte_offset(self) -> usize {
        let Self { ptr, .. } = self;
        ptr.index() * size_of::<U>()
    }

    #[inline]
    pub fn buffer_init_range(self) -> Range<usize> {
        let Self { desc, ptr } = self;

        let start = ptr.index();
        let end = start + item_count::<U>(desc);
        start..end
    }

    #[inline]
    pub fn as_buffer(self) -> *const [U] {
        let Self { desc, ptr } = self;

        let data = ptr.as_item_ptr().cast();
        let len = item_count::<U>(desc);
        ptr::slice_from_raw_parts(data, len)
    }

    #[inline]
    pub fn as_ptr(self) -> *const U {
        let Self { ptr, .. } = self;
        ptr.as_item_ptr().cast()
    }
}

impl<T, U, V> TryFrom<*const V> for ErasedFieldPtr<T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(ptr: *const V) -> Result<Self, Self::Error> {
        let desc = FieldDescriptor::of::<V>();
        check_sufficient_align(desc.layout(), Layout::new::<U>())?;

        let len = item_count::<U>(desc);
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), len);
        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(desc, ptr) };
        Ok(me)
    }
}

impl<T, U, V> TryFrom<ErasedFieldPtr<T>> for *const V
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = ErasedFieldIntoValueError<ErasedFieldPtr<T>>;

    #[inline]
    fn try_from(value: ErasedFieldPtr<T>) -> Result<Self, Self::Error> {
        let ErasedFieldPtr { desc, .. } = value;
        let value = check_into_layout::<V, _>(desc.layout(), value)?;

        let ptr = value.as_ptr().cast();
        Ok(ptr)
    }
}
