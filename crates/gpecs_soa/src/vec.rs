use alloc::{collections::TryReserveError, vec::Vec};
use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr,
};

use crate::{
    multi_vec_len_in_bytes, multi_vec_ptrs,
    slice::{from_raw_parts, from_raw_parts_mut, MultiSlice},
};

#[repr(transparent)]
pub struct MultiVec<T, U, V> {
    inner: Vec<u8>,
    phantom: PhantomData<(Vec<T>, Vec<U>, Vec<V>)>,
}

impl<T, U, V> MultiVec<T, U, V> {
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            phantom: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = multi_vec_len_in_bytes::<T, U, V>(capacity);
        Self {
            inner: Vec::with_capacity(capacity),
            phantom: PhantomData,
        }
    }

    pub fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        let mut me = Self::new();
        me.try_reserve(capacity)?;
        Ok(me)
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn from_raw_parts(ptr: *mut u8, length: usize, capacity: usize) -> Self {
        let capacity = multi_vec_len_in_bytes::<T, U, V>(capacity);
        Self {
            inner: Vec::from_raw_parts(ptr, length, capacity),
            phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        let size_of_all = size_of::<T>() + size_of::<U>() + size_of::<V>();
        self.inner
            .capacity()
            .checked_div(size_of_all)
            .unwrap_or(usize::MAX)
    }

    fn move_right(&mut self, old_capacity: usize) {
        let new_capacity = self.capacity();
        if new_capacity <= old_capacity {
            return;
        }

        unsafe {
            let old_ptrs = multi_vec_ptrs::<T, U, V>(self.as_mut_ptr(), old_capacity);
            let new_ptrs = multi_vec_ptrs::<T, U, V>(self.as_mut_ptr(), new_capacity);

            ptr::copy(old_ptrs.v_ptr, new_ptrs.v_ptr, self.len());
            ptr::copy(old_ptrs.u_ptr, new_ptrs.u_ptr, self.len());
            ptr::copy(old_ptrs.t_ptr, new_ptrs.t_ptr, self.len());
        }
    }

    fn move_left(&mut self, new_capacity: usize) {
        let old_capacity = self.capacity();
        if new_capacity >= old_capacity {
            return;
        }

        unsafe {
            let old_ptrs = multi_vec_ptrs::<T, U, V>(self.as_mut_ptr(), old_capacity);
            let new_ptrs = multi_vec_ptrs::<T, U, V>(self.as_mut_ptr(), new_capacity);

            ptr::copy(old_ptrs.t_ptr, new_ptrs.t_ptr, self.len());
            ptr::copy(old_ptrs.u_ptr, new_ptrs.u_ptr, self.len());
            ptr::copy(old_ptrs.v_ptr, new_ptrs.v_ptr, self.len());
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let old_capacity = self.capacity();
        if old_capacity == 0 {
            let additional = multi_vec_len_in_bytes::<T, U, V>(additional);
            self.inner.reserve(additional);
            return;
        }

        let additional = additional * (size_of::<T>() + size_of::<U>() + size_of::<V>());
        self.inner.reserve(additional);
        self.move_right(old_capacity);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let old_capacity = self.capacity();
        if old_capacity == 0 {
            let additional = multi_vec_len_in_bytes::<T, U, V>(additional);
            self.inner.reserve_exact(additional);
            return;
        }

        let additional = additional * (size_of::<T>() + size_of::<U>() + size_of::<V>());
        self.inner.reserve_exact(additional);
        self.move_right(old_capacity);
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let old_capacity = self.capacity();
        if old_capacity == 0 {
            let additional = multi_vec_len_in_bytes::<T, U, V>(additional);
            return self.inner.try_reserve(additional);
        }

        let additional = additional * (size_of::<T>() + size_of::<U>() + size_of::<V>());
        self.inner.try_reserve(additional)?;
        self.move_right(old_capacity);
        Ok(())
    }

    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let old_capacity = self.capacity();
        if old_capacity == 0 {
            let additional = multi_vec_len_in_bytes::<T, U, V>(additional);
            return self.inner.try_reserve_exact(additional);
        }

        let additional = additional * (size_of::<T>() + size_of::<U>() + size_of::<V>());
        self.inner.try_reserve_exact(additional)?;
        self.move_right(old_capacity);
        Ok(())
    }

    pub fn shrink_to_fit(&mut self) {
        if self.capacity() == self.len() {
            return;
        }

        let len = self.len();
        let new_byte_capacity = multi_vec_len_in_bytes::<T, U, V>(len);
        self.move_left(len);

        unsafe {
            self.inner.set_len(new_byte_capacity);
            self.inner.shrink_to_fit();
            self.inner.set_len(len);
        }
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        if self.capacity() <= min_capacity {
            return;
        }

        let len = self.len();
        let new_byte_capacity = multi_vec_len_in_bytes::<T, U, V>(min_capacity);
        self.move_left(min_capacity);

        unsafe {
            self.inner.set_len(new_byte_capacity);
            self.inner.shrink_to_fit();
            self.inner.set_len(len);
        }
    }

    pub fn truncate(&mut self, len: usize) {
        let new_len = len;
        let old_len = self.len();
        if new_len > old_len {
            return;
        }

        let remaining_len = old_len - new_len;
        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.as_mut_ptrs();
            let t_slice = ptr::slice_from_raw_parts_mut(t_ptr.add(new_len), remaining_len);
            let u_slice = ptr::slice_from_raw_parts_mut(u_ptr.add(new_len), remaining_len);
            let v_slice = ptr::slice_from_raw_parts_mut(v_ptr.add(new_len), remaining_len);

            self.inner.set_len(new_len);

            ptr::drop_in_place(t_slice);
            ptr::drop_in_place(u_slice);
            ptr::drop_in_place(v_slice);
        }
    }

    pub fn as_slice(&self) -> &MultiSlice<T, U, V> {
        self
    }

    pub fn as_mut_slice(&mut self) -> &mut MultiSlice<T, U, V> {
        self
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.inner.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.inner.as_mut_ptr()
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.inner.set_len(new_len);
    }
}

impl<T, U, V> Default for MultiVec<T, U, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, U, V> Deref for MultiVec<T, U, V> {
    type Target = MultiSlice<T, U, V>;

    fn deref(&self) -> &Self::Target {
        let Self { inner, .. } = self;
        unsafe { from_raw_parts(inner) }
    }
}

impl<T, U, V> DerefMut for MultiVec<T, U, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self { inner, .. } = self;
        unsafe { from_raw_parts_mut(inner) }
    }
}

#[cfg(test)]
mod tests {
    use super::MultiVec;

    #[test]
    fn new() {
        let multi_vec = MultiVec::<u32, u16, u8>::new();
        assert!(multi_vec.is_empty());
    }

    #[test]
    fn with_capacity() {
        let multi_vec = MultiVec::<u8, u64, u16>::with_capacity(10);
        assert!(multi_vec.is_empty());
        assert!(multi_vec.capacity() >= 10);
    }
}
