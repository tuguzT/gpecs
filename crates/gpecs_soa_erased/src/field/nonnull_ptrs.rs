use core::{
    alloc::Layout,
    fmt::{self, Debug},
    mem::MaybeUninit,
    ops::Range,
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
    buffer: NonNull<[MaybeUninit<A>]>,
    byte_offset: usize,
}

impl<A> ErasedFieldNonNullPtr<A>
where
    A: AddressableUnit,
{
    #[inline]
    pub fn new(ptr: ErasedFieldMutPtr<A>) -> Option<Self> {
        let desc = ptr.descriptor();
        let buffer = ptr.as_mut_buffer();

        let buffer = ptr::slice_from_raw_parts_mut(buffer.cast(), buffer.len());
        let buffer = NonNull::new(buffer)?;
        let me = unsafe { Self::from_parts(desc, buffer, 0) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(ptr: ErasedFieldMutPtr<A>) -> Self {
        let desc = ptr.descriptor();
        let buffer = ptr.as_mut_buffer();

        let buffer = ptr::slice_from_raw_parts_mut(buffer.cast(), buffer.len());
        let buffer = unsafe { NonNull::new_unchecked(buffer) };
        unsafe { Self::from_parts(desc, buffer, 0) }
    }

    #[inline]
    pub unsafe fn from_parts(
        desc: FieldDescriptor,
        buffer: NonNull<[MaybeUninit<A>]>,
        byte_offset: usize,
    ) -> Self {
        Self {
            desc,
            buffer,
            byte_offset,
        }
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
        let Self {
            desc,
            buffer,
            byte_offset,
        } = self;

        let byte_offset = unsafe { byte_offset.unchecked_add(count * desc.layout().size()) };
        unsafe { Self::from_parts(desc, buffer, byte_offset) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: Self) -> isize {
        let Self {
            desc,
            buffer,
            byte_offset,
        } = self;

        assert_eq!(buffer, origin.as_uninit_buffer());
        check_layout(origin.descriptor().layout(), desc.layout()).expect("layouts should match");

        let byte_offset = byte_offset.cast_signed();
        let origin_byte_offset = origin.byte_offset().cast_signed();
        let offset = byte_offset.wrapping_sub(origin_byte_offset);

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
    pub fn as_uninit_buffer(self) -> NonNull<[MaybeUninit<A>]> {
        let Self { buffer, .. } = self;
        buffer
    }

    #[inline]
    pub fn byte_offset(self) -> usize {
        let Self { byte_offset, .. } = self;
        byte_offset
    }

    #[inline]
    pub fn buffer_init_range(self) -> Range<usize> {
        let (desc, _, byte_offset) = self.into_parts();

        let len = desc.layout().size().div_ceil(size_of::<A>());
        let start = byte_offset.div_ceil(size_of::<A>());
        let end = start + len;
        start..end
    }

    #[inline]
    pub fn as_buffer(self) -> NonNull<[A]> {
        let data = self.as_ptr().as_ptr();
        let len = self.buffer_init_range().len();
        let buffer = ptr::slice_from_raw_parts_mut(data, len);
        unsafe { NonNull::new_unchecked(buffer) }
    }

    #[inline]
    pub fn as_ptr(self) -> NonNull<A> {
        let Self { buffer, .. } = self;

        let offset = self.buffer_init_range().start;
        unsafe { buffer.cast::<A>().add(offset) }
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, NonNull<[MaybeUninit<A>]>, usize) {
        let Self {
            desc,
            buffer,
            byte_offset,
        } = self;
        (desc, buffer, byte_offset)
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

        let buffer = unsafe { NonNull::new_unchecked(buffer) };
        let me = unsafe { Self::from_parts(desc, buffer, 0) };
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
