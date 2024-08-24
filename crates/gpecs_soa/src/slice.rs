use core::{marker::PhantomData, mem::transmute, ptr::NonNull, slice};

use alloc::vec::Vec;

use crate::{multi_vec_ptrs, MultiVecPtrs};

#[repr(transparent)]
pub struct MultiSlice<T, U, V> {
    inner: MultiSliceInner<T, U, V, [()]>,
}

impl<T, U, V> MultiSlice<T, U, V> {
    #[inline]
    fn vec(&self) -> &Vec<u8> {
        unsafe { self.inner.data.as_ref() }
    }

    #[inline]
    fn vec_mut(&mut self) -> &mut Vec<u8> {
        unsafe { self.inner.data.as_mut() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.vec().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let size_of_all = size_of::<T>() + size_of::<U>() + size_of::<V>();
        self.vec()
            .capacity()
            .checked_div(size_of_all)
            .unwrap_or(usize::MAX)
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.vec().as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.vec_mut().as_mut_ptr()
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const T, *const U, *const V) {
        unsafe {
            let len = self.capacity();
            let ptr = self.as_ptr().cast_mut();

            let MultiVecPtrs {
                t_ptr,
                u_ptr,
                v_ptr,
                end,
            } = multi_vec_ptrs::<T, U, V>(ptr, len);
            debug_assert_eq!(end.cast_const(), self.as_ptr().add(self.vec().capacity()));

            (t_ptr, u_ptr, v_ptr)
        }
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (*mut T, *mut U, *mut V) {
        unsafe {
            let len = self.capacity();
            let ptr = self.as_mut_ptr();

            let MultiVecPtrs {
                t_ptr,
                u_ptr,
                v_ptr,
                end,
            } = multi_vec_ptrs::<T, U, V>(ptr, len);
            debug_assert_eq!(end, self.as_mut_ptr().add(self.vec().capacity()));

            (t_ptr, u_ptr, v_ptr)
        }
    }

    #[inline]
    pub fn as_slices(&self) -> (&[T], &[U], &[V]) {
        let (t_data, u_data, v_data) = self.as_ptrs();
        let len = self.len();

        unsafe {
            let t_slice = slice::from_raw_parts(t_data, len);
            let u_slice = slice::from_raw_parts(u_data, len);
            let v_slice = slice::from_raw_parts(v_data, len);
            (t_slice, u_slice, v_slice)
        }
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [U], &mut [V]) {
        let (t_data, u_data, v_data) = self.as_mut_ptrs();
        let len = self.len();

        unsafe {
            let t_slice = slice::from_raw_parts_mut(t_data, len);
            let u_slice = slice::from_raw_parts_mut(u_data, len);
            let v_slice = slice::from_raw_parts_mut(v_data, len);
            (t_slice, u_slice, v_slice)
        }
    }
}

#[repr(C)]
struct MultiSliceInner<T, U, V, Dst>
where
    Dst: ?Sized,
{
    data: NonNull<Vec<u8>>,
    phantom: PhantomData<(NonNull<T>, NonNull<U>, NonNull<V>)>,
    dst: Dst,
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn slice_from_raw_parts<T, U, V>(data: *const Vec<u8>) -> *const MultiSlice<T, U, V> {
    let inner: MultiSliceInner<T, U, V, [(); 0]> = MultiSliceInner {
        data: NonNull::new_unchecked(data.cast_mut()),
        phantom: PhantomData,
        dst: [],
    };
    let inner: *const MultiSliceInner<T, U, V, [()]> = &inner;
    transmute(inner)
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn slice_from_raw_parts_mut<T, U, V>(data: *mut Vec<u8>) -> *mut MultiSlice<T, U, V> {
    let mut inner: MultiSliceInner<T, U, V, [(); 0]> = MultiSliceInner {
        data: NonNull::new_unchecked(data),
        phantom: PhantomData,
        dst: [],
    };
    let inner: *mut MultiSliceInner<T, U, V, [()]> = &mut inner;
    transmute(inner)
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn from_raw_parts<'a, T, U, V>(data: *const Vec<u8>) -> &'a MultiSlice<T, U, V> {
    &*slice_from_raw_parts(data)
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn from_raw_parts_mut<'a, T, U, V>(data: *mut Vec<u8>) -> &'a mut MultiSlice<T, U, V> {
    &mut *slice_from_raw_parts_mut(data)
}
