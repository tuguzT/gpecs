use core::{
    alloc::Layout,
    fmt::{self, Debug},
    mem::MaybeUninit,
    ops::Range,
    ptr,
};

use crate::{
    error::{
        InsufficientAlignError, check_layout, check_len, check_ptr_align, check_sufficient_align,
    },
    field::{
        ErasedFieldMutPtr, ErasedFieldRef,
        error::{ErasedFieldIntoValueError, ErasedFieldPtrError, check_into_layout},
    },
    soa::field::FieldDescriptor,
    storage::AddressableUnit,
};

pub struct ErasedFieldPtr<A>
where
    A: AddressableUnit,
{
    desc: FieldDescriptor,
    buffer: *const [MaybeUninit<A>],
    byte_offset: usize,
}

impl<A> ErasedFieldPtr<A>
where
    A: AddressableUnit,
{
    #[inline]
    pub fn new(desc: FieldDescriptor, buffer: *const [A]) -> Result<Self, ErasedFieldPtrError> {
        check_sufficient_align(desc.layout(), Layout::new::<A>())?;
        check_len(buffer.len() * size_of::<A>(), desc.layout().size())?;
        check_ptr_align(buffer.cast(), desc.layout())?;

        let buffer = ptr::slice_from_raw_parts(buffer.cast(), buffer.len());
        let me = unsafe { Self::from_parts(desc, buffer, 0) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(
        desc: FieldDescriptor,
        buffer: *const [MaybeUninit<A>],
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
        check_sufficient_align(desc.layout(), Layout::new::<A>())?;

        let data = ptr::without_provenance(desc.layout().align());
        let buffer = ptr::slice_from_raw_parts(data, 0);

        let me = unsafe { Self::from_parts(desc, buffer, 0) };
        Ok(me)
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedFieldMutPtr<A> {
        let Self {
            desc,
            buffer,
            byte_offset,
        } = self;

        let buffer = buffer.cast_mut();
        unsafe { ErasedFieldMutPtr::from_parts(desc, buffer, byte_offset) }
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
    pub unsafe fn offset_from(self, origin: ErasedFieldPtr<A>) -> isize {
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
    pub unsafe fn deref<'a>(self) -> ErasedFieldRef<'a, A> {
        unsafe { ErasedFieldRef::from_ptr(self) }
    }

    #[inline]
    pub fn descriptor(self) -> FieldDescriptor {
        let Self { desc, .. } = self;
        desc
    }

    #[inline]
    pub fn as_uninit_buffer(self) -> *const [MaybeUninit<A>] {
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
    pub fn as_buffer(self) -> *const [A] {
        let data = self.as_ptr();
        let len = self.buffer_init_range().len();
        ptr::slice_from_raw_parts(data, len)
    }

    #[inline]
    pub fn as_ptr(self) -> *const A {
        let Self { buffer, .. } = self;

        let offset = self.buffer_init_range().start;
        unsafe { buffer.cast::<A>().add(offset) }
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *const [MaybeUninit<A>], usize) {
        let Self {
            desc,
            buffer,
            byte_offset,
        } = self;
        (desc, buffer, byte_offset)
    }
}

#[expect(clippy::missing_fields_in_debug, reason = "buffer instead of ptr")]
impl<A> Debug for ErasedFieldPtr<A>
where
    A: AddressableUnit,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.desc;
        let buffer = &self.as_buffer();
        f.debug_struct("ErasedFieldPtr")
            .field("desc", desc)
            .field("buffer", buffer)
            .finish()
    }
}

impl<A> Clone for ErasedFieldPtr<A>
where
    A: AddressableUnit,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<A> Copy for ErasedFieldPtr<A> where A: AddressableUnit {}

impl<T, A> TryFrom<*const T> for ErasedFieldPtr<A>
where
    A: AddressableUnit,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(ptr: *const T) -> Result<Self, Self::Error> {
        let desc = FieldDescriptor::of::<T>();
        check_sufficient_align(desc.layout(), Layout::new::<A>())?;

        let len = desc.layout().size().div_ceil(size_of::<A>());
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), len);

        let me = unsafe { Self::from_parts(desc, buffer, 0) };
        Ok(me)
    }
}

impl<T, A> TryFrom<ErasedFieldPtr<A>> for *const T
where
    A: AddressableUnit,
{
    type Error = ErasedFieldIntoValueError<ErasedFieldPtr<A>>;

    #[inline]
    fn try_from(value: ErasedFieldPtr<A>) -> Result<Self, Self::Error> {
        let ErasedFieldPtr { desc, .. } = value;
        let value = check_into_layout::<T, _>(desc.layout(), value)?;

        let ptr = value.as_ptr().cast();
        Ok(ptr)
    }
}
