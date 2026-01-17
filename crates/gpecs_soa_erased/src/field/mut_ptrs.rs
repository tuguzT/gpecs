use core::{
    alloc::Layout,
    fmt::{self, Debug},
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
    ptr: *mut A,
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

        let ptr = buffer.cast();
        Ok(Self { desc, ptr })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: *mut [A]) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer).expect("incorrect inputs");
        }

        let ptr = buffer.cast();
        Self { desc, ptr }
    }

    #[inline]
    pub fn dangling(desc: FieldDescriptor) -> Result<Self, InsufficientAlignError> {
        check_sufficient_align(desc.layout(), Layout::new::<A>())?;

        let data = ptr::without_provenance_mut(desc.layout().align());
        let len = desc.layout().size().div_ceil(size_of::<A>());
        let buffer = ptr::slice_from_raw_parts_mut(data, len);

        let me = unsafe { Self::new_unchecked(desc, buffer) };
        Ok(me)
    }

    #[inline]
    pub fn cast_const(self) -> ErasedFieldPtr<A> {
        let Self { desc, ptr } = self;

        let len = desc.layout().size().div_ceil(size_of::<A>());
        let buffer = ptr::slice_from_raw_parts(ptr.cast_const(), len);
        unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { desc, ptr } = self;

        let size = desc.layout().size().div_ceil(size_of::<A>());
        let data = unsafe { ptr.add(count * size) };
        let buffer = ptr::slice_from_raw_parts_mut(data, size);
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: ErasedFieldPtr<A>) -> isize {
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
    pub fn as_buffer(self) -> *const [A] {
        let Self { desc, ptr } = self;

        let len = desc.layout().size().div_ceil(size_of::<A>());
        ptr::slice_from_raw_parts(ptr.cast_const(), len)
    }

    #[inline]
    pub fn as_mut_buffer(self) -> *mut [A] {
        let Self { desc, ptr } = self;

        let len = desc.layout().size().div_ceil(size_of::<A>());
        ptr::slice_from_raw_parts_mut(ptr, len)
    }

    #[inline]
    pub fn as_ptr(self) -> *const A {
        let Self { ptr, .. } = self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_mut_ptr(self) -> *mut A {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *mut [A]) {
        let Self { desc, ptr } = self;

        let len = desc.layout().size().div_ceil(size_of::<A>());
        let buffer = ptr::slice_from_raw_parts_mut(ptr, len);
        (desc, buffer)
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

#[expect(clippy::expl_impl_clone_on_copy, reason = "no auto-placed bounds")]
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

        let me = unsafe { Self::new_unchecked(desc, buffer) };
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
