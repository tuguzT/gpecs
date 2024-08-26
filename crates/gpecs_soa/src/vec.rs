use alloc::{boxed::Box, collections::TryReserveError, vec::Vec};
use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr,
};

use crate::{
    ptr::{multi_vec_buffer_len, multi_vec_ptrs, slice_from_raw_parts_mut},
    slice::{from_raw_parts, from_raw_parts_mut, MultiSlice},
};

#[repr(transparent)]
pub struct MultiVec<T, U, V> {
    buffer: Vec<usize>,
    phantom: PhantomData<(Vec<T>, Vec<U>, Vec<V>)>,
}

impl<T, U, V> MultiVec<T, U, V> {
    pub const fn new() -> Self {
        Self {
            buffer: Vec::new(),
            phantom: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = multi_vec_buffer_len::<T, U, V>(capacity);
        let mut me = Self {
            buffer: Vec::with_capacity(capacity),
            phantom: PhantomData,
        };

        me.set_len_in_data(0);
        me
    }

    pub fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        let mut me = Self::new();
        me.try_reserve(capacity)?;
        Ok(me)
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn from_raw_parts(ptr: *mut usize, length: usize, capacity: usize) -> Self {
        let capacity = multi_vec_buffer_len::<T, U, V>(capacity);
        Self {
            buffer: Vec::from_raw_parts(ptr, length, capacity),
            phantom: PhantomData,
        }
    }

    pub fn into_raw_parts(self) -> (*mut usize, usize, usize) {
        let mut me = ManuallyDrop::new(self);
        (me.as_mut_ptr(), me.len(), me.capacity())
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn capacity_in_bytes(&self) -> usize {
        self.buffer.capacity() * size_of::<usize>()
    }

    pub fn capacity(&self) -> usize {
        let size_of_all = size_of::<T>() + size_of::<U>() + size_of::<V>();
        self.capacity_in_bytes()
            .saturating_sub(size_of::<usize>())
            .checked_div(size_of_all)
            .unwrap_or(usize::MAX)
    }

    fn move_right(&mut self, old_capacity: usize) {
        let new_capacity = self.capacity();
        if new_capacity <= old_capacity {
            return;
        }

        unsafe {
            let ptr = self.as_mut_ptr().cast();

            let old_len = old_capacity * size_of::<usize>();
            let old_ptrs = multi_vec_ptrs::<T, U, V>(ptr, old_len);

            let new_len = new_capacity * size_of::<usize>();
            let new_ptrs = multi_vec_ptrs::<T, U, V>(ptr, new_len);

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
            let ptr = self.as_mut_ptr().cast();

            let old_len = old_capacity * size_of::<usize>();
            let old_ptrs = multi_vec_ptrs::<T, U, V>(ptr, old_len);

            let new_len = new_capacity * size_of::<usize>();
            let new_ptrs = multi_vec_ptrs::<T, U, V>(ptr, new_len);

            ptr::copy(old_ptrs.t_ptr, new_ptrs.t_ptr, self.len());
            ptr::copy(old_ptrs.u_ptr, new_ptrs.u_ptr, self.len());
            ptr::copy(old_ptrs.v_ptr, new_ptrs.v_ptr, self.len());
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let old_capacity = self.capacity();
        if old_capacity == 0 {
            let additional = multi_vec_buffer_len::<T, U, V>(additional);
            self.buffer.reserve(additional);
            self.set_len_in_data(0);
            return;
        }

        let size_of_all = size_of::<T>() + size_of::<U>() + size_of::<V>();
        let additional_in_bytes = additional * size_of_all;
        let additional = additional_in_bytes / size_of::<usize>() + 1;
        self.buffer.reserve(additional);
        self.move_right(old_capacity);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let old_capacity = self.capacity();
        if old_capacity == 0 {
            let additional = multi_vec_buffer_len::<T, U, V>(additional);
            self.buffer.reserve_exact(additional);
            self.set_len_in_data(0);
            return;
        }

        let size_of_all = size_of::<T>() + size_of::<U>() + size_of::<V>();
        let additional_in_bytes = additional * size_of_all;
        let additional = additional_in_bytes / size_of::<usize>() + 1;
        self.buffer.reserve_exact(additional);
        self.move_right(old_capacity);
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let old_capacity = self.capacity();
        if old_capacity == 0 {
            let additional = multi_vec_buffer_len::<T, U, V>(additional);
            self.buffer.try_reserve(additional)?;
            self.set_len_in_data(0);
            return Ok(());
        }

        let size_of_all = size_of::<T>() + size_of::<U>() + size_of::<V>();
        let additional_in_bytes = additional * size_of_all;
        let additional = additional_in_bytes / size_of::<usize>() + 1;
        self.buffer.try_reserve(additional)?;
        self.move_right(old_capacity);
        Ok(())
    }

    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let old_capacity = self.capacity();
        if old_capacity == 0 {
            let additional = multi_vec_buffer_len::<T, U, V>(additional);
            self.buffer.try_reserve_exact(additional)?;
            self.set_len_in_data(0);
            return Ok(());
        }

        let size_of_all = size_of::<T>() + size_of::<U>() + size_of::<V>();
        let additional_in_bytes = additional * size_of_all;
        let additional = additional_in_bytes / size_of::<usize>() + 1;
        self.buffer.try_reserve_exact(additional)?;
        self.move_right(old_capacity);
        Ok(())
    }

    pub fn shrink_to_fit(&mut self) {
        if self.capacity() == self.len() {
            return;
        }

        let len = self.len();
        self.move_left(len);

        unsafe {
            let new_capacity = multi_vec_buffer_len::<T, U, V>(len);
            self.buffer.set_len(new_capacity);

            self.buffer.shrink_to_fit();
            self.buffer.set_len(len);
        }
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        if self.capacity() <= min_capacity {
            return;
        }

        let len = self.len();
        self.move_left(min_capacity);

        unsafe {
            let new_capacity = multi_vec_buffer_len::<T, U, V>(min_capacity);
            self.buffer.set_len(new_capacity);

            self.buffer.shrink_to_fit();
            self.buffer.set_len(len);
        }
    }

    pub fn into_boxed_slice(self) -> Box<MultiSlice<T, U, V>> {
        let mut me = ManuallyDrop::new(self);
        let data = me.as_mut_ptr();
        let capacity = me.capacity();

        unsafe {
            let raw = slice_from_raw_parts_mut(data, capacity);
            Box::from_raw(raw)
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

            self.set_len(new_len);

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

    pub fn as_ptr(&self) -> *const usize {
        self.buffer.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut usize {
        self.buffer.as_mut_ptr()
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.buffer.set_len(new_len);
        self.set_len_in_data(new_len);
    }

    fn set_len_in_data(&mut self, new_len: usize) {
        if self.capacity_in_bytes() == 0 {
            return;
        }

        unsafe {
            let len = self.as_mut_ptr().cast();
            *len = new_len;
        }
    }

    pub fn insert(&mut self, index: usize, elements: (T, U, V)) {
        #[cold]
        #[inline(never)]
        #[track_caller]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("insertion index (is {index}) should be <= len (is {len})");
        }

        let len = self.len();
        if index > len {
            assert_failed(index, len);
        }

        let capacity = self.capacity();
        if len == capacity {
            self.reserve(1);
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.as_mut_ptrs();
            let t_end = t_ptr.add(index);
            let u_end = u_ptr.add(index);
            let v_end = v_ptr.add(index);

            if index < len {
                ptr::copy(t_end, t_end.add(1), len - index);
                ptr::copy(u_end, u_end.add(1), len - index);
                ptr::copy(v_end, v_end.add(1), len - index);
            }
            ptr::write(t_end, elements.0);
            ptr::write(u_end, elements.1);
            ptr::write(v_end, elements.2);

            self.set_len(len + 1);
        }
    }

    pub fn push(&mut self, values: (T, U, V)) {
        let len = self.len();
        let capacity = self.capacity();
        if len == capacity {
            self.reserve(1);
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.as_mut_ptrs();
            let t_end = t_ptr.add(len);
            let u_end = u_ptr.add(len);
            let v_end = v_ptr.add(len);

            ptr::write(t_end, values.0);
            ptr::write(u_end, values.1);
            ptr::write(v_end, values.2);

            self.set_len(len + 1);
        }
    }

    pub fn swap_remove(&mut self, index: usize) -> (T, U, V) {
        #[cold]
        #[inline(never)]
        #[track_caller]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("swap_remove index (is {index}) should be < len (is {len})");
        }

        let len = self.len();
        if index >= len {
            assert_failed(index, len);
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.as_mut_ptrs();
            let t_value = ptr::read(t_ptr.add(index));
            let u_value = ptr::read(u_ptr.add(index));
            let v_value = ptr::read(v_ptr.add(index));

            ptr::copy(t_ptr.add(len - 1), t_ptr.add(index), 1);
            ptr::copy(u_ptr.add(len - 1), u_ptr.add(index), 1);
            ptr::copy(v_ptr.add(len - 1), v_ptr.add(index), 1);

            self.set_len(len - 1);
            (t_value, u_value, v_value)
        }
    }

    pub fn remove(&mut self, index: usize) -> (T, U, V) {
        #[cold]
        #[inline(never)]
        #[track_caller]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("removal index (is {index}) should be < len (is {len})");
        }

        let len = self.len();
        if index >= len {
            assert_failed(index, len);
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.as_mut_ptrs();
            let t_ptr = t_ptr.add(index);
            let u_ptr = u_ptr.add(index);
            let v_ptr = v_ptr.add(index);

            let t_value = ptr::read(t_ptr);
            let u_value = ptr::read(u_ptr);
            let v_value = ptr::read(v_ptr);

            ptr::copy(t_ptr.add(1), t_ptr, len - index - 1);
            ptr::copy(u_ptr.add(1), u_ptr, len - index - 1);
            ptr::copy(v_ptr.add(1), v_ptr, len - index - 1);

            self.set_len(len - 1);
            (t_value, u_value, v_value)
        }
    }

    pub fn pop(&mut self) -> Option<(T, U, V)> {
        let len = self.len();
        if len == 0 {
            return None;
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.as_mut_ptrs();
            let t_value = ptr::read(t_ptr.add(len - 1));
            let u_value = ptr::read(u_ptr.add(len - 1));
            let v_value = ptr::read(v_ptr.add(len - 1));

            self.set_len(len - 1);
            Some((t_value, u_value, v_value))
        }
    }
}

impl<T, U, V> Debug for MultiVec<T, U, V>
where
    T: Debug,
    U: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (t_slice, u_slice, v_slice) = self.as_slices();
        f.debug_struct("MultiVec")
            .field("t_slice", &t_slice)
            .field("u_slice", &u_slice)
            .field("v_slice", &v_slice)
            .finish()
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
        let data = self.as_ptr();
        let capacity = self.capacity();
        unsafe { from_raw_parts(data, capacity) }
    }
}

impl<T, U, V> DerefMut for MultiVec<T, U, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let data = self.as_mut_ptr();
        let capacity = self.capacity();
        unsafe { from_raw_parts_mut(data, capacity) }
    }
}

impl<T, U, V> Drop for MultiVec<T, U, V> {
    fn drop(&mut self) {
        if self.is_empty() {
            return;
        }

        let (t_ptr, u_ptr, v_ptr) = self.as_mut_ptrs();
        let len = self.len();

        let t_ptr = ptr::slice_from_raw_parts_mut(t_ptr, len);
        let u_ptr = ptr::slice_from_raw_parts_mut(u_ptr, len);
        let v_ptr = ptr::slice_from_raw_parts_mut(v_ptr, len);

        unsafe {
            ptr::drop_in_place(t_ptr);
            ptr::drop_in_place(u_ptr);
            ptr::drop_in_place(v_ptr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MultiVec;

    #[test]
    fn check_null_opt() {
        type MultiVec = super::MultiVec<u32, u16, u8>;
        assert_eq!(size_of::<Option<MultiVec>>(), size_of::<MultiVec>());
    }

    #[test]
    fn new() {
        let multi_vec = MultiVec::<u32, u16, u8>::new();
        assert!(multi_vec.is_empty());
        assert_eq!(multi_vec.capacity(), 0);

        let slice = multi_vec.as_slice();
        assert!(slice.is_empty());
        assert_eq!(slice.capacity(), 0);

        let boxed_slice = multi_vec.into_boxed_slice();
        assert!(boxed_slice.is_empty());
        assert_eq!(boxed_slice.capacity(), 0);
    }

    #[test]
    fn with_capacity() {
        let multi_vec = MultiVec::<u8, u64, u16>::with_capacity(10);
        assert!(multi_vec.is_empty());
        assert!(multi_vec.capacity() >= 10);

        let slice = multi_vec.as_slice();
        assert!(slice.is_empty());
        assert!(slice.capacity() >= 10);

        let boxed_slice = multi_vec.into_boxed_slice();
        assert!(boxed_slice.is_empty());
        assert!(boxed_slice.capacity() >= 10);
    }

    #[test]
    fn one_item() {
        let mut multi_vec = MultiVec::<u8, u32, u16>::new();
        multi_vec.push((1, 2, 3));
        assert_eq!(multi_vec.len(), 1);
        assert!(multi_vec.capacity() >= 1);

        let slice = multi_vec.as_slice();
        assert_eq!(slice.len(), 1);
        assert!(slice.capacity() >= 1);
        assert_eq!(
            slice.as_slices(),
            ([1].as_slice(), [2].as_slice(), [3].as_slice()),
        );

        let (t, u, v) = multi_vec.pop().expect("multi vector should not be empty");
        assert_eq!((t, u, v), (1, 2, 3));
        assert!(multi_vec.is_empty());
        assert!(multi_vec.capacity() >= 1);

        let boxed_slice = multi_vec.into_boxed_slice();
        assert!(boxed_slice.is_empty());
        assert!(boxed_slice.capacity() >= 1);
    }
}
