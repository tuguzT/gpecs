use core::{alloc::Layout, mem::MaybeUninit, ops::Range, ptr};

use crate::{
    error::{InsufficientAlignError, check_len, check_ptr_align, check_sufficient_align},
    field::{
        ErasedFieldPtr, ErasedFieldRef, ErasedFieldRefMut,
        error::{ErasedFieldIntoValueError, ErasedFieldPtrError, check_into_layout},
    },
    slice_item_ptr::{CastConstPtr, MutSliceItemPtr, SliceItemPtr},
    soa::field::FieldDescriptor,
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldMutPtr<T> {
    desc: FieldDescriptor,
    ptr: T,
}

impl<T> ErasedFieldMutPtr<T> {
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

impl<T> ErasedFieldMutPtr<T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    pub fn cast_const(self) -> ErasedFieldPtr<CastConstPtr<T>> {
        let Self { desc, ptr } = self;
        let ptr = ptr.cast_const();
        unsafe { ErasedFieldPtr::from_parts(desc, ptr) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldRef<'a, CastConstPtr<T>> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedFieldRefMut<'a, T> {
        unsafe { ErasedFieldRefMut::from_ptr(self) }
    }
}

impl<T, U> ErasedFieldMutPtr<T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub fn new(desc: FieldDescriptor, buffer: *mut [U]) -> Result<Self, ErasedFieldPtrError> {
        check_sufficient_align(desc.layout(), Layout::new::<U>())?;
        check_len(buffer.len() * size_of::<U>(), desc.layout().size())?;
        check_ptr_align(buffer.cast(), desc.layout())?;

        let buffer = ptr::slice_from_raw_parts_mut(buffer.cast(), buffer.len());
        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(desc, ptr) };
        Ok(me)
    }

    #[inline]
    pub fn dangling(desc: FieldDescriptor) -> Result<Self, InsufficientAlignError> {
        check_sufficient_align(desc.layout(), Layout::new::<U>())?;

        let data = ptr::without_provenance_mut(desc.layout().align());
        let buffer = ptr::slice_from_raw_parts_mut(data, 0);
        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(desc, ptr) };
        Ok(me)
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { desc, ptr } = self;

        let field_size = desc.layout().size().div_ceil(size_of::<U>());
        let ptr = unsafe { ptr.add(count * field_size) };
        unsafe { Self::from_parts(desc, ptr) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: ErasedFieldPtr<CastConstPtr<T>>) -> isize {
        let Self { desc, ptr } = self;

        let offset = unsafe { ptr.cast_const().offset_from(origin.ptr()) };
        let field_size = desc.layout().size().div_ceil(size_of::<U>()).cast_signed();
        offset
            .checked_div(field_size)
            .expect("erased field pointer should not be a ZST")
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap(self, with: Self) {
        let Self { ptr, .. } = self;

        let this_buffer_range = self.buffer_init_range();
        for i in 0..this_buffer_range.len() {
            let this = unsafe { ptr.add(i) };
            let with = unsafe { with.ptr.add(i) };
            unsafe { this.swap(with) }
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from(self, src: ErasedFieldPtr<CastConstPtr<T>>, count: usize) {
        let Self { desc, ptr } = self;

        let src = src.ptr();
        let count = count * desc.layout().size().div_ceil(size_of::<U>());
        unsafe { ptr.copy_from(src, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping(
        self,
        src: ErasedFieldPtr<CastConstPtr<T>>,
        count: usize,
    ) {
        let Self { desc, ptr } = self;

        let src = src.ptr();
        let count = count * desc.layout().size().div_ceil(size_of::<U>());
        unsafe { ptr.copy_from_nonoverlapping(src, count) }
    }

    #[inline]
    pub fn as_uninit_buffer(self) -> *const [MaybeUninit<U>] {
        let Self { ptr, .. } = self;
        ptr.slice().cast_const()
    }

    #[inline]
    pub fn as_mut_uninit_buffer(self) -> *mut [MaybeUninit<U>] {
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

        let len = desc.layout().size().div_ceil(size_of::<U>());
        let start = ptr.index();
        let end = start + len;
        start..end
    }

    #[inline]
    pub fn as_buffer(self) -> *const [U] {
        let data = self.as_ptr();
        let len = self.buffer_init_range().len();
        ptr::slice_from_raw_parts(data, len)
    }

    #[inline]
    pub fn as_mut_buffer(self) -> *mut [U] {
        let data = self.as_mut_ptr();
        let len = self.buffer_init_range().len();
        ptr::slice_from_raw_parts_mut(data, len)
    }

    #[inline]
    pub fn as_ptr(self) -> *const U {
        let Self { ptr, .. } = self;

        let offset = self.buffer_init_range().start;
        unsafe { ptr.slice().cast::<U>().add(offset).cast_const() }
    }

    #[inline]
    pub fn as_mut_ptr(self) -> *mut U {
        let Self { ptr, .. } = self;

        let offset = self.buffer_init_range().start;
        unsafe { ptr.slice().cast::<U>().add(offset) }
    }
}

impl<T, U, V> TryFrom<*mut V> for ErasedFieldMutPtr<T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(ptr: *mut V) -> Result<Self, Self::Error> {
        let desc = FieldDescriptor::of::<V>();
        check_sufficient_align(desc.layout(), Layout::new::<U>())?;

        let len = desc.layout().size().div_ceil(size_of::<U>());
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast(), len);
        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(desc, ptr) };
        Ok(me)
    }
}

impl<T, U, V> TryFrom<ErasedFieldMutPtr<T>> for *mut V
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = ErasedFieldIntoValueError<ErasedFieldMutPtr<T>>;

    #[inline]
    fn try_from(value: ErasedFieldMutPtr<T>) -> Result<Self, Self::Error> {
        let ErasedFieldMutPtr { desc, .. } = value;
        let value = check_into_layout::<V, _>(desc.layout(), value)?;

        let ptr = value.as_mut_ptr().cast();
        Ok(ptr)
    }
}
