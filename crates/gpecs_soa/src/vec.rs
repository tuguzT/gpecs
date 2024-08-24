use alloc::{boxed::Box, collections::TryReserveError, vec::Vec};
use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    multi_vec_len_in_bytes,
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

    pub fn reserve(&mut self, additional: usize) {
        let old_capacity = self.capacity();
        if old_capacity == 0 {
            let additional = multi_vec_len_in_bytes::<T, U, V>(additional);
            self.inner.reserve(additional);
            return;
        }

        let additional = additional * (size_of::<T>() + size_of::<U>() + size_of::<V>());
        self.inner.reserve(additional);
        if old_capacity == self.capacity() {
            return;
        }

        todo!("move data to the right starting from the end")
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let size_of_all = size_of::<T>() + size_of::<U>() + size_of::<V>();
        self.inner.reserve_exact(additional * size_of_all);

        // TODO move data if needed
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let size_of_all = size_of::<T>() + size_of::<U>() + size_of::<V>();
        self.inner.try_reserve(additional * size_of_all)?;

        // TODO move data if needed
        Ok(())
    }

    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let size_of_all = size_of::<T>() + size_of::<U>() + size_of::<V>();
        self.inner.try_reserve_exact(additional * size_of_all)?;

        // TODO move data if needed
        Ok(())
    }

    pub fn shrink_to_fit(&mut self) {
        // TODO move data if needed

        unsafe {
            let old_len = self.inner.len();
            let new_len = self.inner.capacity();

            self.inner.set_len(new_len);
            self.inner.shrink_to_fit();
            self.inner.set_len(old_len);
        }
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        // TODO move data if needed

        unsafe {
            let old_len = self.inner.len();
            let new_len = self.inner.capacity().min(min_capacity);

            self.inner.set_len(new_len);
            self.inner.shrink_to(min_capacity);
            self.inner.set_len(old_len);
        }
    }

    pub fn into_boxed_slice(self) -> Box<MultiSlice<T, U, V>> {
        todo!()
    }

    pub fn truncate(&mut self, len: usize) {
        let _ = len;
        todo!()
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
