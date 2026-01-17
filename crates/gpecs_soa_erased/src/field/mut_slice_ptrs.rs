use core::{
    alloc::Layout,
    fmt::{self, Debug},
    ptr,
};

use crate::{
    error::{InsufficientAlignError, check_ptr_align, check_sufficient_align},
    field::{
        ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldSlice, ErasedFieldSliceMut,
        ErasedFieldSlicePtr,
        assert::{check_into_layout, check_slice_buffer_len},
        error::{ErasedFieldIntoValueError, ErasedFieldSlicePtrError},
    },
    soa::field::FieldDescriptor,
    storage::AddressableUnit,
};

pub struct ErasedFieldSliceMutPtr<A>
where
    A: AddressableUnit,
{
    ptr: ErasedFieldMutPtr<A>,
    len: usize,
}

impl<A> ErasedFieldSliceMutPtr<A>
where
    A: AddressableUnit,
{
    #[inline]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        desc: FieldDescriptor,
        buffer: *mut [A],
        len: usize,
    ) -> Result<Self, ErasedFieldSlicePtrError> {
        check_sufficient_align(desc.layout(), Layout::new::<A>())?;
        check_slice_buffer_len(buffer.len() * size_of::<A>(), desc.layout().size(), len)?;
        check_ptr_align(buffer.cast(), desc.layout())?;

        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) };
        let me = unsafe { Self::from_ptr(ptr, len) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: *mut [A], len: usize) -> Self {
        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) };
        unsafe { Self::from_ptr(ptr, len) }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedFieldMutPtr<A>, len: usize) -> Self {
        Self { ptr, len }
    }

    #[inline]
    pub fn cast_const(self) -> ErasedFieldSlicePtr<A> {
        let Self { ptr, len } = self;
        unsafe { ErasedFieldSlicePtr::from_ptr(ptr.cast_const(), len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldSlice<'a, A> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedFieldSliceMut<'a, A> {
        unsafe { ErasedFieldSliceMut::from_ptr(self) }
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
    pub fn as_buffer(self) -> *const [A] {
        let Self { ptr, len } = self;
        let buffer = ptr.as_buffer();
        ptr::slice_from_raw_parts(buffer.cast(), len * buffer.len())
    }

    #[inline]
    pub fn as_mut_buffer(self) -> *mut [A] {
        let Self { ptr, len } = self;
        let buffer = ptr.as_mut_buffer();
        ptr::slice_from_raw_parts_mut(buffer.cast(), len * buffer.len())
    }

    #[inline]
    pub fn as_ptr(self) -> *const A {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(self) -> *mut A {
        let Self { ptr, .. } = self;
        ptr.as_mut_ptr()
    }

    #[inline]
    pub fn as_field_ptr(self) -> ErasedFieldPtr<A> {
        let Self { ptr, .. } = self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_mut_field_ptr(self) -> ErasedFieldMutPtr<A> {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *mut [A], usize) {
        let Self { ptr, len } = self;
        let (desc, buffer) = ptr.into_parts();
        let buffer = ptr::slice_from_raw_parts_mut(buffer.cast(), len * buffer.len());
        (desc, buffer, len)
    }
}

#[expect(
    clippy::missing_fields_in_debug,
    reason = "buffer & len instead of ptr"
)]
impl<A> Debug for ErasedFieldSliceMutPtr<A>
where
    A: AddressableUnit,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let buffer = &self.as_buffer();
        let len = &self.len;
        f.debug_struct("ErasedFieldSliceMutPtr")
            .field("desc", desc)
            .field("buffer", buffer)
            .field("len", len)
            .finish()
    }
}

#[expect(clippy::expl_impl_clone_on_copy, reason = "no auto-placed bounds")]
impl<A> Clone for ErasedFieldSliceMutPtr<A>
where
    A: AddressableUnit,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<A> Copy for ErasedFieldSliceMutPtr<A> where A: AddressableUnit {}

impl<T, A> TryFrom<*mut [T]> for ErasedFieldSliceMutPtr<A>
where
    A: AddressableUnit,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(ptr: *mut [T]) -> Result<Self, Self::Error> {
        let desc = FieldDescriptor::of::<T>();
        check_sufficient_align(desc.layout(), Layout::new::<A>())?;

        let len = ptr.len();
        let buffer_len = desc.layout().size().div_ceil(size_of::<A>()) * len;
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast(), buffer_len);

        let me = unsafe { Self::new_unchecked(desc, buffer, len) };
        Ok(me)
    }
}

impl<T, A> TryFrom<ErasedFieldSliceMutPtr<A>> for *mut [T]
where
    A: AddressableUnit,
{
    type Error = ErasedFieldIntoValueError<ErasedFieldSliceMutPtr<A>>;

    #[inline]
    fn try_from(value: ErasedFieldSliceMutPtr<A>) -> Result<Self, Self::Error> {
        let value = check_into_layout::<T, _>(value.descriptor().layout(), value)?;
        let ErasedFieldSliceMutPtr { ptr, len, .. } = value;

        let data = ptr.as_mut_ptr().cast();
        let slice = ptr::slice_from_raw_parts_mut(data, len);
        Ok(slice)
    }
}

#[inline]
pub unsafe fn field_slice_from_raw_parts_mut<A>(
    data: ErasedFieldMutPtr<A>,
    len: usize,
) -> ErasedFieldSliceMutPtr<A>
where
    A: AddressableUnit,
{
    unsafe { ErasedFieldSliceMutPtr::from_ptr(data, len) }
}
