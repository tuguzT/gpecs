use core::{alloc::Layout, ptr};

use crate::{
    data::{
        ErasedMutPtr, ErasedMutSlice, ErasedNonNullSlicePtr, ErasedPtr, ErasedSlice,
        ErasedSlicePtr,
        error::{DataError, DowncastError, UpcastSliceError, check_downcast},
    },
    error::{InsufficientAlignError, check_len, check_ptr_align, check_sufficient_align},
    layout::bytes_to_items,
    ptr::slice::{ConstPtr, MutSliceItemPtr, NonNullPtr},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedMutSlicePtr<T> {
    len: usize,
    ptr: ErasedMutPtr<T>,
}

impl<T> ErasedMutSlicePtr<T> {
    #[inline]
    pub unsafe fn from_parts(ptr: ErasedMutPtr<T>, len: usize) -> Self {
        Self { len, ptr }
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
    pub fn layout(self) -> Layout {
        let Self { ptr, .. } = self;
        ptr.layout()
    }

    #[inline]
    pub fn into_parts(self) -> (ErasedMutPtr<T>, usize) {
        let Self { ptr, len } = self;
        (ptr, len)
    }

    #[inline]
    pub fn as_mut_ptr(self) -> ErasedMutPtr<T> {
        let Self { ptr, .. } = self;
        ptr
    }
}

impl<T> ErasedMutSlicePtr<T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    #[expect(
        clippy::not_unsafe_ptr_arg_deref,
        reason = "`T::from_slice` should not dereference input slice"
    )]
    pub fn new(layout: Layout, items: *mut [T::Item], len: usize) -> Result<Self, DataError> {
        check_ptr_align(items.cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let layout_of_items = Layout::array::<T::Item>(items.len())?;
        let expected_layout = layout.pad_to_align().repeat_packed(len)?;
        check_len(layout_of_items.size(), expected_layout.size())?;

        let ptr = unsafe { T::from_parts(items, 0) };
        let ptr = unsafe { ErasedMutPtr::from_parts(layout, ptr) };

        let me = unsafe { Self::from_parts(ptr, len) };
        Ok(me)
    }

    #[inline]
    pub fn dangling(layout: Layout) -> Result<Self, InsufficientAlignError> {
        let ptr = ErasedMutPtr::dangling(layout)?;
        let me = unsafe { Self::from_parts(ptr, 0) };
        Ok(me)
    }

    #[inline]
    pub fn upcast<V>(ptr: *mut [V]) -> Result<Self, UpcastSliceError> {
        let layout = Layout::new::<V>();
        check_ptr_align(ptr.cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let len = ptr.len();
        let slice_len = bytes_to_items::<T::Item>(Layout::array::<V>(len)?.size());
        let slice = ptr::slice_from_raw_parts_mut(ptr.cast(), slice_len);

        let ptr = unsafe { MutSliceItemPtr::from_parts(slice, 0) };
        let ptr = unsafe { ErasedMutPtr::from_parts(layout, ptr) };
        let me = unsafe { Self::from_parts(ptr, len) };
        Ok(me)
    }

    #[inline]
    pub fn downcast<V>(self) -> Result<*mut [V], DowncastError<Self>> {
        let layout = self.layout();
        let Self { ptr, len, .. } = check_downcast::<V, _>(layout, self)?;

        let data = ptr.as_mut_item_ptr().cast();
        let slice = ptr::slice_from_raw_parts_mut(data, len);
        Ok(slice)
    }

    #[inline]
    pub fn cast_const(self) -> ErasedSlicePtr<ConstPtr<T::Ptrs>> {
        let Self { ptr, len } = self;
        let ptr = ptr.cast_const();
        unsafe { ErasedSlicePtr::from_parts(ptr, len) }
    }

    #[inline]
    pub fn cast_nonnull(self) -> Option<ErasedNonNullSlicePtr<NonNullPtr<T::Ptrs>>> {
        ErasedNonNullSlicePtr::from_mut_slice_ptr(self)
    }

    #[inline]
    pub unsafe fn cast_nonnull_unchecked(self) -> ErasedNonNullSlicePtr<NonNullPtr<T::Ptrs>> {
        unsafe { ErasedNonNullSlicePtr::from_mut_slice_ptr_unchecked(self) }
    }

    #[inline]
    pub unsafe fn as_ref<'a>(self) -> Option<ErasedSlice<'a, NonNullPtr<T::Ptrs>>> {
        unsafe { self.cast_const().as_ref() }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> ErasedSlice<'a, NonNullPtr<T::Ptrs>> {
        unsafe { self.cast_const().as_ref_unchecked() }
    }

    #[inline]
    pub unsafe fn as_mut<'a>(self) -> Option<ErasedMutSlice<'a, NonNullPtr<T::Ptrs>>> {
        unsafe { ErasedMutSlice::from_mut_slice_ptr(self) }
    }

    #[inline]
    pub unsafe fn as_mut_unchecked<'a>(self) -> ErasedMutSlice<'a, NonNullPtr<T::Ptrs>> {
        unsafe { ErasedMutSlice::from_mut_slice_ptr_unchecked(self) }
    }

    #[inline]
    pub fn as_ptr(self) -> ErasedPtr<ConstPtr<T::Ptrs>> {
        let Self { ptr, .. } = self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_items(self) -> *const [T::Item] {
        let Self { ptr, len } = self;

        let items = ptr.as_items();
        let len = items.len().wrapping_mul(len);
        ptr::slice_from_raw_parts(items.cast(), len)
    }

    #[inline]
    pub fn as_mut_items(self) -> *mut [T::Item] {
        let Self { ptr, len } = self;

        let items = ptr.as_mut_items();
        let len = items.len().wrapping_mul(len);
        ptr::slice_from_raw_parts_mut(items.cast(), len)
    }

    #[inline]
    pub fn as_item_ptr(self) -> *const T::Item {
        let Self { ptr, .. } = self;
        ptr.as_item_ptr()
    }

    #[inline]
    pub fn as_mut_item_ptr(self) -> *mut T::Item {
        let Self { ptr, .. } = self;
        ptr.as_mut_item_ptr()
    }
}
