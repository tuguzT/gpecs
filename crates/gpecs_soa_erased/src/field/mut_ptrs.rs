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
        ErasedFieldPtr, ErasedFieldRef, ErasedFieldRefMut,
        assert::check_into_layout,
        error::{ErasedFieldIntoValueError, ErasedFieldPtrError},
    },
    soa::field::FieldDescriptor,
    storage::AddressableUnit,
};

pub struct ErasedFieldMutPtr<A>
where
    A: AddressableUnit,
{
    desc: FieldDescriptor,
    buffer: *mut [MaybeUninit<A>],
    byte_offset: usize,
}

impl<A> ErasedFieldMutPtr<A>
where
    A: AddressableUnit,
{
    #[inline]
    pub fn new(desc: FieldDescriptor, buffer: *mut [A]) -> Result<Self, ErasedFieldPtrError> {
        check_sufficient_align(desc.layout(), Layout::new::<A>())?;
        check_len(buffer.len() * size_of::<A>(), desc.layout().size())?;
        check_ptr_align(buffer.cast(), desc.layout())?;

        let buffer = ptr::slice_from_raw_parts_mut(buffer.cast(), buffer.len());
        let me = unsafe { Self::from_parts(desc, buffer, 0) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(
        desc: FieldDescriptor,
        buffer: *mut [MaybeUninit<A>],
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

        let data = ptr::without_provenance_mut(desc.layout().align());
        let buffer = ptr::slice_from_raw_parts_mut(data, 0);

        let me = unsafe { Self::from_parts(desc, buffer, 0) };
        Ok(me)
    }

    #[inline]
    pub fn cast_const(self) -> ErasedFieldPtr<A> {
        let Self {
            desc,
            buffer,
            byte_offset,
        } = self;

        let buffer = buffer.cast_const();
        unsafe { ErasedFieldPtr::from_parts(desc, buffer, byte_offset) }
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

        assert_eq!(buffer.cast_const(), origin.as_uninit_buffer());
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

        let a = self.as_mut_ptr();
        let b = with.as_mut_ptr();
        let count = desc.layout().size().div_ceil(size_of::<A>());
        for i in 0..count {
            unsafe { ptr::swap(a.add(i), b.add(i)) }
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from(self, from: ErasedFieldPtr<A>, count: usize) {
        let Self { desc, .. } = self;
        check_layout(from.descriptor().layout(), desc.layout()).expect("layouts should match");

        let src = from.as_ptr();
        let dst = self.as_mut_ptr();
        let count = count * desc.layout().size().div_ceil(size_of::<A>());
        unsafe { ptr::copy(src, dst, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping(self, from: ErasedFieldPtr<A>, count: usize) {
        let Self { desc, .. } = self;
        check_layout(from.descriptor().layout(), desc.layout()).expect("layouts should match");

        let src = from.as_ptr();
        let dst = self.as_mut_ptr();
        let count = count * desc.layout().size().div_ceil(size_of::<A>());
        unsafe { ptr::copy_nonoverlapping(src, dst, count) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldRef<'a, A> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedFieldRefMut<'a, A> {
        unsafe { ErasedFieldRefMut::from_ptr(self) }
    }

    #[inline]
    pub fn descriptor(self) -> FieldDescriptor {
        let Self { desc, .. } = self;
        desc
    }

    #[inline]
    pub fn as_uninit_buffer(self) -> *const [MaybeUninit<A>] {
        let Self { buffer, .. } = self;
        buffer.cast_const()
    }

    #[inline]
    pub fn as_mut_uninit_buffer(self) -> *mut [MaybeUninit<A>] {
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
    pub fn as_mut_buffer(self) -> *mut [A] {
        let data = self.as_mut_ptr();
        let len = self.buffer_init_range().len();
        ptr::slice_from_raw_parts_mut(data, len)
    }

    #[inline]
    pub fn as_ptr(self) -> *const A {
        let Self { buffer, .. } = self;

        let offset = self.buffer_init_range().start;
        unsafe { buffer.cast::<A>().add(offset).cast_const() }
    }

    #[inline]
    pub fn as_mut_ptr(self) -> *mut A {
        let Self { buffer, .. } = self;

        let offset = self.buffer_init_range().start;
        unsafe { buffer.cast::<A>().add(offset) }
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *mut [MaybeUninit<A>], usize) {
        let Self {
            desc,
            buffer,
            byte_offset,
        } = self;
        (desc, buffer, byte_offset)
    }
}

#[expect(clippy::missing_fields_in_debug, reason = "buffer instead of ptr")]
impl<A> Debug for ErasedFieldMutPtr<A>
where
    A: AddressableUnit,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.desc;
        let buffer = &self.as_buffer();
        f.debug_struct("ErasedFieldMutPtr")
            .field("desc", desc)
            .field("buffer", buffer)
            .finish()
    }
}

impl<A> Clone for ErasedFieldMutPtr<A>
where
    A: AddressableUnit,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<A> Copy for ErasedFieldMutPtr<A> where A: AddressableUnit {}

impl<T, A> TryFrom<*mut T> for ErasedFieldMutPtr<A>
where
    A: AddressableUnit,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(ptr: *mut T) -> Result<Self, Self::Error> {
        let desc = FieldDescriptor::of::<T>();
        check_sufficient_align(desc.layout(), Layout::new::<A>())?;

        let len = desc.layout().size().div_ceil(size_of::<A>());
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast(), len);

        let me = unsafe { Self::from_parts(desc, buffer, 0) };
        Ok(me)
    }
}

impl<T, A> TryFrom<ErasedFieldMutPtr<A>> for *mut T
where
    A: AddressableUnit,
{
    type Error = ErasedFieldIntoValueError<ErasedFieldMutPtr<A>>;

    #[inline]
    fn try_from(value: ErasedFieldMutPtr<A>) -> Result<Self, Self::Error> {
        let ErasedFieldMutPtr { desc, .. } = value;
        let value = check_into_layout::<T, _>(desc.layout(), value)?;

        let ptr = value.as_mut_ptr().cast();
        Ok(ptr)
    }
}
