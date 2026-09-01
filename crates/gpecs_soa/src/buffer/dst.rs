use core::ptr;

use crate::{
    buffer::{
        BufferData, BufferPrefix, buffer_layout, fields_are_zst, layout_is_dangling,
        ptr_to_buffer_context, ptr_to_buffer_prefix_unchecked, ptrs_from_buffer,
        ptrs_from_buffer_mut,
    },
    traits::{AllocSoaContext, AllocSoaTrusted, MutPtrs, Ptrs},
};

#[repr(transparent)]
pub struct DstBuffer<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    inner: [BufferData<T>],
}

impl<T> DstBuffer<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    pub unsafe fn ptr_from_raw_parts(data: *const u8, len: usize, capacity: usize) -> *const Self {
        let context = unsafe { ptr_to_buffer_context::<T>(data).as_ref_unchecked() };
        Self::assert_trait_safety_requirements(context);

        let len = Self::len_of_inner(context, len, capacity);
        let inner = ptr::slice_from_raw_parts(data.cast(), len);
        unsafe { Self::ptr_from_inner(inner) }
    }

    #[inline]
    pub unsafe fn ptr_from_raw_parts_mut(data: *mut u8, len: usize, capacity: usize) -> *mut Self {
        let context = unsafe { ptr_to_buffer_context::<T>(data).as_ref_unchecked() };
        Self::assert_trait_safety_requirements(context);

        let len = Self::len_of_inner(context, len, capacity);
        let inner = ptr::slice_from_raw_parts_mut(data.cast(), len);
        unsafe { Self::ptr_from_inner_mut(inner) }
    }

    unsafe fn ptr_from_inner(inner: *const [BufferData<T>]) -> *const Self {
        // Self is transparent over a slice of `BufferData<T>`
        inner as *const Self
    }

    unsafe fn ptr_from_inner_mut(inner: *mut [BufferData<T>]) -> *mut Self {
        // Self is transparent over a slice of `BufferData<T>`
        inner as *mut Self
    }

    fn assert_trait_safety_requirements(context: &T::Context) {
        let size_of_fields = size_of::<T::Fields>();
        let packed_size_of_fields = context
            .packed_size_of_fields()
            .expect("sum of sized of fields should be in bounds of `usize`");
        assert!(
            packed_size_of_fields <= size_of_fields,
            "sum of sizes of field layouts (is {packed_size_of_fields}) \
            should be less or equal to the size of `RawSoa::Fields` (is {size_of_fields})"
        );

        let buffer_align = context.buffer_align();
        let align_of_fields = align_of::<T::Fields>();
        assert!(
            buffer_align <= align_of_fields,
            "each alignment from field layouts (largest is {buffer_align}) \
            should be less or equal to the alignment of `RawSoa::Fields` (is {align_of_fields})"
        );
    }

    fn len_of_inner(context: &T::Context, len: usize, capacity: usize) -> usize {
        let buffer_layout = buffer_layout::<T>(context, capacity)
            .expect("layout size should not exceed `isize::MAX`");

        if layout_is_dangling(buffer_layout) {
            return len;
        }
        buffer_layout.size() / size_of::<BufferData<T>>()
    }

    #[inline]
    pub fn ptr_as_ptr(this: *const Self) -> *const u8 {
        Self::ptr_as_inner(this).cast()
    }

    #[inline]
    pub fn ptr_as_mut_ptr(this: *mut Self) -> *mut u8 {
        Self::ptr_as_inner_mut(this).cast()
    }

    fn ptr_as_inner(this: *const Self) -> *const [BufferData<T>] {
        // Self is transparent over a slice of `BufferData<T>`
        this as *const [BufferData<T>]
    }

    fn ptr_as_inner_mut(this: *mut Self) -> *mut [BufferData<T>] {
        // Self is transparent over a slice of `BufferData<T>`
        this as *mut [BufferData<T>]
    }

    #[inline]
    pub unsafe fn ptr_to_context(this: *const Self) -> *const T::Context {
        let buffer = Self::ptr_as_ptr(this);
        unsafe { ptr_to_buffer_context::<T>(buffer) }
    }

    #[inline]
    pub unsafe fn ptr_to_len(this: *const Self) -> Option<*const usize> {
        let prefix = unsafe { Self::ptr_to_prefix(this) }?;
        let len = unsafe { &raw const (*prefix).len };
        Some(len)
    }

    #[inline]
    pub unsafe fn ptr_to_capacity(this: *const Self) -> Option<*const usize> {
        let prefix = unsafe { Self::ptr_to_prefix(this) }?;
        let capacity = unsafe { &raw const (*prefix).capacity };
        Some(capacity)
    }

    unsafe fn ptr_to_prefix(this: *const Self) -> Option<*const BufferPrefix<T>> {
        if Self::ptr_is_dangling(this) {
            return None;
        }

        let buffer = Self::ptr_as_ptr(this);
        let prefix = unsafe { ptr_to_buffer_prefix_unchecked::<T>(buffer) };
        Some(prefix)
    }

    fn ptr_is_dangling(this: *const Self) -> bool {
        size_of::<BufferData<T>>() == 0 || Self::ptr_as_inner(this).len() == 0
    }

    #[inline]
    pub unsafe fn ptr_len(this: *const Self) -> usize {
        let len = unsafe { Self::ptr_to_len(this) };
        let Some(len) = len else {
            return Self::ptr_as_inner(this).len();
        };
        unsafe { len.read() }
    }

    #[inline]
    pub unsafe fn ptr_capacity(this: *const Self) -> usize {
        let context = unsafe { Self::ptr_to_context(this).as_ref_unchecked() };
        if fields_are_zst::<T>(context) {
            return usize::MAX;
        }

        let capacity = unsafe { Self::ptr_to_capacity(this) };
        let Some(capacity) = capacity else {
            return 0;
        };
        unsafe { capacity.read() }
    }

    #[inline]
    pub unsafe fn ptr_as_ptrs<'a>(this: *const Self) -> Ptrs<'a, T>
    where
        T::Context: 'a,
    {
        let ptr = Self::ptr_as_ptr(this);
        let context = unsafe { Self::ptr_to_context(this).as_ref_unchecked() };
        let capacity = unsafe { Self::ptr_capacity(this) };
        unsafe { ptrs_from_buffer::<T>(context, ptr, capacity) }
    }

    #[inline]
    pub unsafe fn ptr_as_mut_ptrs<'a>(this: *mut Self) -> MutPtrs<'a, T>
    where
        T::Context: 'a,
    {
        let ptr = Self::ptr_as_mut_ptr(this);
        let context = unsafe { Self::ptr_to_context(this).as_ref_unchecked() };
        let capacity = unsafe { Self::ptr_capacity(this) };
        unsafe { ptrs_from_buffer_mut::<T>(context, ptr, capacity) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { inner } = self;
        inner.as_ptr().cast()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { inner } = self;
        inner.as_mut_ptr().cast()
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let this = ptr::from_ref(self);
        unsafe { Self::ptr_to_context(this).as_ref_unchecked() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let this = ptr::from_ref(self);
        unsafe { Self::ptr_len(this) }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let this = ptr::from_ref(self);
        unsafe { Self::ptr_capacity(this) }
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&T::Context, Ptrs<'_, T>) {
        let this = ptr::from_ref(self);
        let ptrs = unsafe { Self::ptr_as_ptrs(this) };
        (self.context(), ptrs)
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&T::Context, MutPtrs<'_, T>) {
        let this = ptr::from_mut(self);
        let ptrs = unsafe { Self::ptr_as_mut_ptrs(this) };
        (self.context(), ptrs)
    }
}
