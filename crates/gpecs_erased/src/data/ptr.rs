use core::{alloc::Layout, mem::MaybeUninit, ops::Range, ptr};

use crate::{
    data::{
        ErasedMutPtr, ErasedRef,
        error::{DataError, DowncastError, TryFromPtrError, check_downcast},
    },
    error::{InsufficientAlignError, check_len, check_ptr_align, check_sufficient_align},
    layout::bytes_to_items,
    ptr::slice::{CastMutPtr, ConstSliceItemPtr},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedPtr<T> {
    layout: Layout,
    ptr: T,
}

impl<T> ErasedPtr<T> {
    #[inline]
    pub unsafe fn from_parts(layout: Layout, ptr: T) -> Self {
        Self { layout, ptr }
    }

    #[inline]
    pub fn layout(self) -> Layout {
        let Self { layout, .. } = self;
        layout
    }

    #[inline]
    pub fn ptr(self) -> T {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (Layout, T) {
        let Self { layout, ptr } = self;
        (layout, ptr)
    }
}

impl<T> ErasedPtr<T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    pub fn cast_mut(self) -> ErasedMutPtr<CastMutPtr<T>> {
        let Self { layout, ptr } = self;

        let ptr = ptr.cast_mut();
        unsafe { ErasedMutPtr::from_parts(layout, ptr) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedRef<'a, T> {
        unsafe { ErasedRef::from_ptr(self) }
    }
}

impl<T, U> ErasedPtr<T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub fn new(layout: Layout, buffer: *const [U]) -> Result<Self, DataError> {
        check_ptr_align(buffer.cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<U>())?;

        let buffer_layout = Layout::array::<U>(buffer.len())?;
        check_len(buffer_layout.size(), layout.size())?;

        let buffer = ptr::slice_from_raw_parts(buffer.cast(), buffer.len());
        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(layout, ptr) };
        Ok(me)
    }

    #[inline]
    pub fn dangling(layout: Layout) -> Result<Self, InsufficientAlignError> {
        check_sufficient_align(layout, Layout::new::<U>())?;

        let data = ptr::without_provenance(layout.align());
        let buffer = ptr::slice_from_raw_parts(data, 0);
        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(layout, ptr) };
        Ok(me)
    }

    #[inline]
    pub fn downcast<V>(self) -> Result<*const V, DowncastError<Self>> {
        let Self { layout, .. } = self;
        let Self { ptr, .. } = check_downcast::<V, _>(layout, self)?;

        let ptr = ptr.as_item_ptr().cast();
        Ok(ptr)
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { layout, ptr } = self;

        let count = bytes_to_items::<U>(layout.size()).wrapping_mul(count);
        let ptr = unsafe { ptr.add(count) };
        unsafe { Self::from_parts(layout, ptr) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from(self, origin: Self) -> isize {
        let Self { layout, ptr } = self;

        let offset = unsafe { ptr.offset_from(origin.ptr()) };
        let len = bytes_to_items::<U>(layout.size()).cast_signed();
        offset
            .checked_div(len)
            .expect("erased field pointer should not be a ZST")
    }

    #[inline]
    pub fn as_uninit_buffer(self) -> *const [MaybeUninit<U>] {
        let Self { ptr, .. } = self;
        ptr.slice()
    }

    #[inline]
    pub fn byte_offset(self) -> usize {
        let Self { ptr, .. } = self;
        ptr.index().wrapping_mul(size_of::<U>())
    }

    #[inline]
    pub fn buffer_init_range(self) -> Range<usize> {
        let Self { layout, ptr } = self;

        let start = ptr.index();
        let end = start + bytes_to_items::<U>(layout.size());
        start..end
    }

    #[inline]
    pub fn as_buffer(self) -> *const [U] {
        let Self { layout, ptr } = self;

        let data = ptr.as_item_ptr().cast();
        let len = bytes_to_items::<U>(layout.size());
        ptr::slice_from_raw_parts(data, len)
    }

    #[inline]
    pub fn as_ptr(self) -> *const U {
        let Self { ptr, .. } = self;
        ptr.as_item_ptr().cast()
    }
}

impl<T, U, V> TryFrom<*const V> for ErasedPtr<T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = TryFromPtrError;

    #[inline]
    fn try_from(ptr: *const V) -> Result<Self, Self::Error> {
        let layout = Layout::new::<V>();
        check_ptr_align(ptr.cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<U>())?;

        let len = bytes_to_items::<U>(layout.size());
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), len);
        let ptr = unsafe { T::from_slice(buffer, 0) };

        let me = unsafe { Self::from_parts(layout, ptr) };
        Ok(me)
    }
}

impl<T, U, V> TryFrom<ErasedPtr<T>> for *const V
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = DowncastError<ErasedPtr<T>>;

    #[inline]
    fn try_from(ptr: ErasedPtr<T>) -> Result<Self, Self::Error> {
        ptr.downcast()
    }
}
