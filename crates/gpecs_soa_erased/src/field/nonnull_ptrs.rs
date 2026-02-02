use core::{
    alloc::Layout,
    fmt::{self, Debug},
    ptr::{self, NonNull},
};

use crate::{
    error::{InsufficientAlignError, check_layout, check_sufficient_align},
    field::{ErasedFieldMutPtr, assert::check_into_layout, error::ErasedFieldIntoValueError},
    soa::field::FieldDescriptor,
    storage::AddressableUnit,
};

pub struct ErasedFieldNonNullPtr<A>
where
    A: AddressableUnit,
{
    desc: FieldDescriptor,
    ptr: NonNull<A>,
}

impl<A> ErasedFieldNonNullPtr<A>
where
    A: AddressableUnit,
{
    #[inline]
    pub fn new(ptr: ErasedFieldMutPtr<A>) -> Option<Self> {
        let (desc, buffer) = ptr.into_parts();
        let ptr = NonNull::new(buffer)?.cast();
        let me = unsafe { Self::from_parts(desc, ptr) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(ptr: ErasedFieldMutPtr<A>) -> Self {
        let (desc, buffer) = ptr.into_parts();
        let ptr = unsafe { NonNull::new_unchecked(buffer) }.cast();
        unsafe { Self::from_parts(desc, ptr) }
    }

    #[inline]
    pub unsafe fn from_parts(desc: FieldDescriptor, ptr: NonNull<A>) -> Self {
        Self { desc, ptr }
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

        let size = desc.layout().size().div_ceil(size_of::<A>());
        let data = unsafe { ptr.add(count * size) };
        let buffer = ptr::slice_from_raw_parts_mut(data.as_ptr(), size);

        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) };
        unsafe { Self::new_unchecked(ptr) }
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
            .div_ceil(size_of::<A>())
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
        let count = desc.layout().size().div_ceil(size_of::<A>());
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
        let count = count * desc.layout().size().div_ceil(size_of::<A>());
        unsafe { ptr::copy(src.as_ptr(), dst.as_ptr(), count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping(self, from: Self, count: usize) {
        let Self { desc, .. } = self;
        check_layout(from.descriptor().layout(), desc.layout()).expect("layouts should match");

        let src = from.as_ptr();
        let dst = self.as_ptr();
        let count = count * desc.layout().size().div_ceil(size_of::<A>());
        unsafe { ptr::copy_nonoverlapping(src.as_ptr(), dst.as_ptr(), count) }
    }

    #[inline]
    pub fn descriptor(self) -> FieldDescriptor {
        let Self { desc, .. } = self;
        desc
    }

    #[inline]
    pub fn as_buffer(self) -> NonNull<[A]> {
        let Self { desc, ptr } = self;

        let len = desc.layout().size().div_ceil(size_of::<A>());
        let buffer = ptr::slice_from_raw_parts_mut(ptr.as_ptr(), len);
        unsafe { NonNull::new_unchecked(buffer) }
    }

    #[inline]
    pub fn as_ptr(self) -> NonNull<A> {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, NonNull<[A]>) {
        let Self { desc, ptr } = self;

        let len = desc.layout().size().div_ceil(size_of::<A>());
        let buffer = ptr::slice_from_raw_parts_mut(ptr.as_ptr(), len);
        let buffer = unsafe { NonNull::new_unchecked(buffer) };
        (desc, buffer)
    }
}

#[expect(clippy::missing_fields_in_debug, reason = "buffer instead of ptr")]
impl<A> Debug for ErasedFieldNonNullPtr<A>
where
    A: AddressableUnit,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.desc;
        let buffer = &self.as_buffer();
        f.debug_struct("ErasedFieldNonNullPtr")
            .field("desc", desc)
            .field("buffer", buffer)
            .finish()
    }
}

impl<A> Clone for ErasedFieldNonNullPtr<A>
where
    A: AddressableUnit,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<A> Copy for ErasedFieldNonNullPtr<A> where A: AddressableUnit {}

impl<T, A> TryFrom<NonNull<T>> for ErasedFieldNonNullPtr<A>
where
    A: AddressableUnit,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(ptr: NonNull<T>) -> Result<Self, Self::Error> {
        let desc = FieldDescriptor::of::<T>();
        check_sufficient_align(desc.layout(), Layout::new::<A>())?;

        let len = desc.layout().size().div_ceil(size_of::<A>());
        let buffer = ptr::slice_from_raw_parts_mut(ptr.as_ptr().cast(), len);

        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) };
        let me = unsafe { Self::new_unchecked(ptr) };
        Ok(me)
    }
}

impl<T, A> TryFrom<ErasedFieldNonNullPtr<A>> for NonNull<T>
where
    A: AddressableUnit,
{
    type Error = ErasedFieldIntoValueError<ErasedFieldNonNullPtr<A>>;

    #[inline]
    fn try_from(value: ErasedFieldNonNullPtr<A>) -> Result<Self, Self::Error> {
        let ErasedFieldNonNullPtr { desc, .. } = value;
        let value = check_into_layout::<T, _>(desc.layout(), value)?;

        let ptr = value.as_ptr().cast();
        Ok(ptr)
    }
}
