use core::{
    alloc::Layout,
    fmt::{self, Debug},
    mem::MaybeUninit,
    ptr,
};

use crate::{
    error::{InsufficientAlignError, check_ptr_align, check_sufficient_align},
    field::{
        ErasedFieldPtr, ErasedFieldSlice, ErasedFieldSliceMutPtr,
        assert::{check_into_layout, check_slice_buffer_len},
        error::{ErasedFieldIntoValueError, ErasedFieldSlicePtrError},
    },
    soa::field::FieldDescriptor,
    storage::AddressableUnit,
};

pub struct ErasedFieldSlicePtr<A>
where
    A: AddressableUnit,
{
    ptr: ErasedFieldPtr<A>,
    len: usize,
}

impl<A> ErasedFieldSlicePtr<A>
where
    A: AddressableUnit,
{
    #[inline]
    pub fn new(
        desc: FieldDescriptor,
        buffer: *const [A],
        len: usize,
    ) -> Result<Self, ErasedFieldSlicePtrError> {
        check_sufficient_align(desc.layout(), Layout::new::<A>())?;
        check_slice_buffer_len(buffer.len() * size_of::<A>(), desc.layout().size(), len)?;
        check_ptr_align(buffer.cast(), desc.layout())?;

        let buffer = ptr::slice_from_raw_parts(buffer.cast(), buffer.len());
        let me = unsafe { Self::from_parts(desc, buffer, 0, len) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(
        desc: FieldDescriptor,
        buffer: *const [MaybeUninit<A>],
        byte_offset: usize,
        len: usize,
    ) -> Self {
        let ptr = unsafe { ErasedFieldPtr::from_parts(desc, buffer, byte_offset) };
        unsafe { Self::from_ptr(ptr, len) }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedFieldPtr<A>, len: usize) -> Self {
        Self { ptr, len }
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedFieldSliceMutPtr<A> {
        let Self { ptr, len } = self;
        unsafe { ErasedFieldSliceMutPtr::from_ptr(ptr.cast_mut(), len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldSlice<'a, A> {
        unsafe { ErasedFieldSlice::from_ptr(self) }
    }

    #[inline]
    pub fn len(self) -> usize {
        let Self { len, .. } = self;
        len
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn descriptor(self) -> FieldDescriptor {
        let Self { ptr, .. } = self;
        ptr.descriptor()
    }

    #[inline]
    pub fn as_uninit_buffer(self) -> *const [MaybeUninit<A>] {
        let Self { ptr, .. } = self;
        ptr.as_uninit_buffer()
    }

    #[inline]
    pub fn byte_offset(self) -> usize {
        let Self { ptr, .. } = self;
        ptr.byte_offset()
    }

    #[inline]
    pub fn as_buffer(self) -> *const [A] {
        let Self { ptr, len } = self;
        let buffer = ptr.as_buffer();
        ptr::slice_from_raw_parts(buffer.cast(), len * buffer.len())
    }

    #[inline]
    pub fn as_ptr(self) -> *const A {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn as_field_ptr(self) -> ErasedFieldPtr<A> {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *const [MaybeUninit<A>], usize, usize) {
        let Self { ptr, len } = self;
        let (desc, buffer, byte_offset) = ptr.into_parts();

        let buffer = ptr::slice_from_raw_parts(buffer.cast(), len * buffer.len());
        (desc, buffer, byte_offset, len)
    }
}

#[expect(
    clippy::missing_fields_in_debug,
    reason = "buffer & len instead of ptr"
)]
impl<A> Debug for ErasedFieldSlicePtr<A>
where
    A: AddressableUnit,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let buffer = &self.as_buffer();
        let len = &self.len;
        f.debug_struct("ErasedFieldSlicePtr")
            .field("desc", desc)
            .field("buffer", buffer)
            .field("len", len)
            .finish()
    }
}

impl<A> Clone for ErasedFieldSlicePtr<A>
where
    A: AddressableUnit,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<A> Copy for ErasedFieldSlicePtr<A> where A: AddressableUnit {}

impl<T, A> TryFrom<*const [T]> for ErasedFieldSlicePtr<A>
where
    A: AddressableUnit,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(ptr: *const [T]) -> Result<Self, Self::Error> {
        let desc = FieldDescriptor::of::<T>();
        check_sufficient_align(desc.layout(), Layout::new::<A>())?;

        let len = ptr.len();
        let buffer_len = desc.layout().size().div_ceil(size_of::<A>()) * len;
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), buffer_len);

        let me = unsafe { Self::from_parts(desc, buffer, 0, len) };
        Ok(me)
    }
}

impl<T, A> TryFrom<ErasedFieldSlicePtr<A>> for *const [T]
where
    A: AddressableUnit,
{
    type Error = ErasedFieldIntoValueError<ErasedFieldSlicePtr<A>>;

    #[inline]
    fn try_from(value: ErasedFieldSlicePtr<A>) -> Result<Self, Self::Error> {
        let value = check_into_layout::<T, _>(value.descriptor().layout(), value)?;
        let ErasedFieldSlicePtr { ptr, len, .. } = value;

        let data = ptr.as_ptr().cast();
        let slice = ptr::slice_from_raw_parts(data, len);
        Ok(slice)
    }
}

#[inline]
pub unsafe fn field_slice_from_raw_parts<A>(
    data: ErasedFieldPtr<A>,
    len: usize,
) -> ErasedFieldSlicePtr<A>
where
    A: AddressableUnit,
{
    unsafe { ErasedFieldSlicePtr::from_ptr(data, len) }
}
