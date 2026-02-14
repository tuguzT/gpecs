use core::{alloc::Layout, mem::MaybeUninit, ops::Range, ptr::NonNull};

use crate::{
    bytes_to_items::item_count,
    error::{InsufficientAlignError, check_sufficient_align},
    field::{
        ErasedFieldMutPtr,
        error::{ErasedFieldIntoValueError, check_into_layout},
    },
    slice_item_ptr::{MutSliceItemPtr, NonNullAsPtr, NonNullSliceItemPtr},
    soa::field::FieldDescriptor,
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldNonNullPtr<T> {
    desc: FieldDescriptor,
    ptr: T,
}

impl<T> ErasedFieldNonNullPtr<T> {
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

impl<T, U> ErasedFieldNonNullPtr<T>
where
    T: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub fn new(ptr: ErasedFieldMutPtr<NonNullAsPtr<T>>) -> Option<Self> {
        let (desc, ptr) = ptr.into_parts();

        let buffer = ptr.slice();
        let buffer = NonNull::new(buffer)?;
        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(desc, ptr) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(ptr: ErasedFieldMutPtr<NonNullAsPtr<T>>) -> Self {
        let (desc, ptr) = ptr.into_parts();

        let buffer = ptr.slice();
        let buffer = unsafe { NonNull::new_unchecked(buffer) };
        let ptr = unsafe { T::from_slice(buffer, 0) };

        unsafe { Self::from_parts(desc, ptr) }
    }

    #[inline]
    pub fn dangling(desc: FieldDescriptor) -> Result<Self, InsufficientAlignError> {
        let ptr = ErasedFieldMutPtr::dangling(desc)?;
        let me = unsafe { Self::new_unchecked(ptr) };
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
    #[track_caller]
    pub unsafe fn swap(self, with: Self) {
        let Self { desc, ptr } = self;

        for i in 0..item_count::<U>(desc) {
            let this = unsafe { ptr.add(i) }.as_ptr();
            let with = unsafe { with.ptr.add(i) }.as_ptr();
            unsafe { this.swap(with) }
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from(self, src: Self, count: usize) {
        let Self { desc, ptr } = self;

        let src = src.ptr().as_ptr().cast_const();
        let count = count * item_count::<U>(desc);
        unsafe { ptr.as_ptr().copy_from(src, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping(self, src: Self, count: usize) {
        let Self { desc, ptr } = self;

        let src = src.ptr().as_ptr().cast_const();
        let count = count * item_count::<U>(desc);
        unsafe { ptr.as_ptr().copy_from_nonoverlapping(src, count) }
    }

    #[inline]
    pub fn as_uninit_buffer(self) -> NonNull<[MaybeUninit<U>]> {
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
    pub fn as_buffer(self) -> NonNull<[U]> {
        let Self { desc, ptr } = self;

        let data = ptr.as_item_ptr().cast();
        let len = item_count::<U>(desc);
        NonNull::slice_from_raw_parts(data, len)
    }

    #[inline]
    pub fn as_ptr(self) -> NonNull<U> {
        let Self { ptr, .. } = self;
        ptr.as_item_ptr().cast()
    }
}

impl<T, U, V> TryFrom<NonNull<V>> for ErasedFieldNonNullPtr<T>
where
    T: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(ptr: NonNull<V>) -> Result<Self, Self::Error> {
        let desc = FieldDescriptor::of::<V>();
        check_sufficient_align(desc.layout(), Layout::new::<U>())?;

        let len = item_count::<U>(desc);
        let buffer = NonNull::slice_from_raw_parts(ptr.cast(), len);
        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(desc, ptr) };
        Ok(me)
    }
}

impl<T, U, V> TryFrom<ErasedFieldNonNullPtr<T>> for NonNull<V>
where
    T: NonNullSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = ErasedFieldIntoValueError<ErasedFieldNonNullPtr<T>>;

    #[inline]
    fn try_from(value: ErasedFieldNonNullPtr<T>) -> Result<Self, Self::Error> {
        let ErasedFieldNonNullPtr { desc, .. } = value;
        let value = check_into_layout::<V, _>(desc.layout(), value)?;

        let ptr = value.as_ptr().cast();
        Ok(ptr)
    }
}
