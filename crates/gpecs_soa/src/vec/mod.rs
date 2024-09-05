use alloc::boxed::Box;
use core::{
    borrow::{Borrow, BorrowMut},
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::{self, addr_of, addr_of_mut},
    slice,
};

pub use crate::raw_vec::{TryReserveError, TryReserveErrorKind};

use crate::{
    ptr::{min_size_of, ptrs, slice_from_len_in_bytes_mut},
    raw_vec::RawSoaVec,
    slice::{from_len_in_bytes, from_len_in_bytes_mut, Iter, IterMut, SoaSlice},
};

pub use self::into_iter::IntoIter;

mod into_iter;

pub struct SoaVec<T, U, V> {
    buffer: RawSoaVec<T, U, V>,
    len: usize,
}

impl<T, U, V> SoaVec<T, U, V> {
    pub const fn new() -> Self {
        Self {
            buffer: RawSoaVec::new(),
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut me = Self {
            buffer: RawSoaVec::with_capacity(capacity),
            len: 0,
        };

        me.set_len_in_buffer(0);
        me
    }

    pub fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        let mut me = Self {
            buffer: RawSoaVec::try_with_capacity(capacity)?,
            len: 0,
        };

        me.set_len_in_buffer(0);
        Ok(me)
    }

    #[allow(clippy::missing_safety_doc)]
    pub const unsafe fn from_raw_parts(ptr: *mut u8, length: usize, capacity: usize) -> Self {
        Self {
            buffer: unsafe { RawSoaVec::from_raw_parts(ptr, capacity) },
            len: length,
        }
    }

    pub(crate) const unsafe fn from_capacity_in_bytes(
        ptr: *mut u8,
        length: usize,
        capacity_in_bytes: usize,
    ) -> Self {
        Self {
            buffer: unsafe { RawSoaVec::from_capacity_in_bytes(ptr, capacity_in_bytes) },
            len: length,
        }
    }

    pub fn into_raw_parts(self) -> (*mut u8, usize, usize) {
        let mut me = ManuallyDrop::new(self);
        (me.as_mut_ptr(), me.len(), me.capacity())
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub const fn capacity_in_bytes(&self) -> usize {
        self.buffer.capacity_in_bytes()
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    fn move_right(&mut self, old_capacity: usize) {
        let new_capacity = self.capacity();
        if new_capacity <= old_capacity {
            return;
        }

        unsafe {
            let ptr = self.as_mut_ptr();
            let old_ptrs = ptrs::<T, U, V>(ptr, old_capacity);
            let new_ptrs = ptrs::<T, U, V>(ptr, new_capacity);

            ptr::copy(old_ptrs.2, new_ptrs.2, self.len());
            ptr::copy(old_ptrs.1, new_ptrs.1, self.len());
            ptr::copy(old_ptrs.0, new_ptrs.0, self.len());
        }
    }

    fn move_left(&mut self, new_capacity: usize) {
        let old_capacity = self.capacity();
        if new_capacity >= old_capacity {
            return;
        }

        unsafe {
            let ptr = self.as_mut_ptr();
            let old_ptrs = ptrs::<T, U, V>(ptr, old_capacity);
            let new_ptrs = ptrs::<T, U, V>(ptr, new_capacity);

            ptr::copy(old_ptrs.0, new_ptrs.0, self.len());
            ptr::copy(old_ptrs.1, new_ptrs.1, self.len());
            ptr::copy(old_ptrs.2, new_ptrs.2, self.len());
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        if !self.buffer.needs_to_grow(self.len, additional) {
            return;
        }

        let old_capacity = self.capacity();
        self.buffer.reserve(self.len, additional);

        match old_capacity {
            0 => self.set_len_in_buffer(0),
            _ => self.move_right(old_capacity),
        }
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        if !self.buffer.needs_to_grow(self.len, additional) {
            return;
        }

        let old_capacity = self.capacity();
        self.buffer.reserve_exact(self.len, additional);

        match old_capacity {
            0 => self.set_len_in_buffer(0),
            _ => self.move_right(old_capacity),
        }
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        if !self.buffer.needs_to_grow(self.len, additional) {
            return Ok(());
        }

        let old_capacity = self.capacity();
        self.buffer.try_reserve(self.len, additional)?;

        match old_capacity {
            0 => self.set_len_in_buffer(0),
            _ => self.move_right(old_capacity),
        };
        Ok(())
    }

    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        if !self.buffer.needs_to_grow(self.len, additional) {
            return Ok(());
        }

        let old_capacity = self.capacity();
        self.buffer.try_reserve_exact(self.len, additional)?;

        match old_capacity {
            0 => self.set_len_in_buffer(0),
            _ => self.move_right(old_capacity),
        };
        Ok(())
    }

    pub fn shrink_to_fit(&mut self) {
        if self.capacity() <= self.len {
            return;
        }

        self.move_left(self.len);
        self.buffer.shrink_to_fit(self.len);
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        if self.capacity() <= min_capacity {
            return;
        }

        let new_capacity = cmp::max(self.len, min_capacity);
        self.move_left(new_capacity);
        self.buffer.shrink_to_fit(new_capacity);
    }

    pub fn into_boxed_slice(mut self) -> Box<SoaSlice<T, U, V>> {
        self.shrink_to_fit();
        let mut me = ManuallyDrop::new(self);

        if min_size_of::<T, U, V>() == 0 && me.len > 0 {
            let (data, len_in_bytes) = match me.capacity_in_bytes() {
                0 => (Box::into_raw(Box::new(me.len)).cast(), size_of::<usize>()),
                _ => (me.as_mut_ptr(), me.capacity_in_bytes()),
            };
            let slice = slice_from_len_in_bytes_mut(data, len_in_bytes);
            return unsafe { Box::from_raw(slice) };
        }

        unsafe {
            let buffer = ptr::read(&me.buffer);
            let len = me.len;
            buffer.into_box(len)
        }
    }

    pub fn truncate(&mut self, len: usize) {
        if len > self.len {
            return;
        }

        let remaining_len = self.len - len;
        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.as_mut_ptrs();
            let t_slice = ptr::slice_from_raw_parts_mut(t_ptr.add(len), remaining_len);
            let u_slice = ptr::slice_from_raw_parts_mut(u_ptr.add(len), remaining_len);
            let v_slice = ptr::slice_from_raw_parts_mut(v_ptr.add(len), remaining_len);

            self.set_len(len);

            ptr::drop_in_place(t_slice);
            ptr::drop_in_place(u_slice);
            ptr::drop_in_place(v_slice);
        }
    }

    pub fn as_slice(&self) -> &SoaSlice<T, U, V> {
        self
    }

    pub fn as_mut_slice(&mut self) -> &mut SoaSlice<T, U, V> {
        self
    }

    pub const fn as_ptr(&self) -> *const u8 {
        self.buffer.ptr().cast_const()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buffer.ptr()
    }

    pub fn as_ptrs(&self) -> (*const T, *const U, *const V) {
        let (t_ptr, u_ptr, v_ptr) = self.buffer.ptrs();
        (t_ptr, u_ptr, v_ptr)
    }

    pub fn as_mut_ptrs(&mut self) -> (*mut T, *mut U, *mut V) {
        self.buffer.ptrs()
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

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());

        self.len = new_len;
        self.set_len_in_buffer(new_len);
    }

    fn set_len_in_buffer(&mut self, new_len: usize) {
        if self.capacity_in_bytes() == 0 {
            return;
        }

        unsafe {
            let len = self.as_mut_ptr().cast();
            *len = new_len;
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

        if len == self.capacity() {
            self.buffer.grow_one();
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.as_mut_ptrs();
            let t_ptr = t_ptr.add(index);
            let u_ptr = u_ptr.add(index);
            let v_ptr = v_ptr.add(index);

            if index < len {
                ptr::copy(t_ptr, t_ptr.add(1), len - index);
                ptr::copy(u_ptr, u_ptr.add(1), len - index);
                ptr::copy(v_ptr, v_ptr.add(1), len - index);
            }
            ptr::write(t_ptr, elements.0);
            ptr::write(u_ptr, elements.1);
            ptr::write(v_ptr, elements.2);

            self.set_len(len + 1);
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

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut((&T, &U, &V)) -> bool,
    {
        self.retain_mut(|(t, u, v)| f((t, u, v)));
    }

    pub fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut((&mut T, &mut U, &mut V)) -> bool,
    {
        let original_len = self.len();
        // Avoid double drop if the drop guard is not executed,
        // since we may make some holes during the process.
        unsafe { self.set_len(0) };

        // Vec: [Kept, Kept, Hole, Hole, Hole, Hole, Unchecked, Unchecked]
        //      |<-              processed len   ->| ^- next to check
        //                  |<-  deleted cnt     ->|
        //      |<-              original_len                          ->|
        // Kept: Elements which predicate returns true on.
        // Hole: Moved or dropped element slot.
        // Unchecked: Unchecked valid elements.
        //
        // This drop guard will be invoked when predicate or `drop` of element panicked.
        // It shifts unchecked elements to cover holes and `set_len` to the correct length.
        // In cases when predicate and `drop` never panick, it will be optimized out.
        struct BackshiftOnDrop<'a, T, U, V> {
            v: &'a mut SoaVec<T, U, V>,
            processed_len: usize,
            deleted_cnt: usize,
            original_len: usize,
        }

        impl<T, U, V> Drop for BackshiftOnDrop<'_, T, U, V> {
            fn drop(&mut self) {
                if self.deleted_cnt > 0 {
                    // SAFETY: Trailing unchecked items must be valid since we never touch them.
                    unsafe {
                        let (t_ptr, u_ptr, v_ptr) = self.v.as_mut_ptrs();
                        ptr::copy(
                            t_ptr.add(self.processed_len),
                            t_ptr.add(self.processed_len - self.deleted_cnt),
                            self.original_len - self.processed_len,
                        );
                        ptr::copy(
                            u_ptr.add(self.processed_len),
                            u_ptr.add(self.processed_len - self.deleted_cnt),
                            self.original_len - self.processed_len,
                        );
                        ptr::copy(
                            v_ptr.add(self.processed_len),
                            v_ptr.add(self.processed_len - self.deleted_cnt),
                            self.original_len - self.processed_len,
                        );
                    }
                }
                // SAFETY: After filling holes, all items are in contiguous memory.
                unsafe {
                    self.v.set_len(self.original_len - self.deleted_cnt);
                }
            }
        }

        let mut g = BackshiftOnDrop {
            v: self,
            processed_len: 0,
            deleted_cnt: 0,
            original_len,
        };

        fn process_loop<F, T, U, V, const DELETED: bool>(
            original_len: usize,
            f: &mut F,
            g: &mut BackshiftOnDrop<'_, T, U, V>,
        ) where
            F: FnMut((&mut T, &mut U, &mut V)) -> bool,
        {
            while g.processed_len != original_len {
                // SAFETY: Unchecked element must be valid.
                let (t_cur, u_cur, v_cur) = unsafe {
                    let (t_ptr, u_ptr, v_ptr) = g.v.as_mut_ptrs();
                    (
                        &mut *t_ptr.add(g.processed_len),
                        &mut *u_ptr.add(g.processed_len),
                        &mut *v_ptr.add(g.processed_len),
                    )
                };
                if !f((t_cur, u_cur, v_cur)) {
                    // Advance early to avoid double drop if `drop_in_place` panicked.
                    g.processed_len += 1;
                    g.deleted_cnt += 1;
                    // SAFETY: We never touch this element again after dropped.
                    unsafe {
                        ptr::drop_in_place(t_cur);
                        ptr::drop_in_place(u_cur);
                        ptr::drop_in_place(v_cur);
                    };
                    // We already advanced the counter.
                    if DELETED {
                        continue;
                    } else {
                        break;
                    }
                }
                if DELETED {
                    // SAFETY: `deleted_cnt` > 0, so the hole slot must not overlap with current element.
                    // We use copy for move, and never touch this element again.
                    unsafe {
                        let (t_ptr, u_ptr, v_ptr) = g.v.as_mut_ptrs();
                        ptr::copy_nonoverlapping(
                            t_cur,
                            t_ptr.add(g.processed_len - g.deleted_cnt),
                            1,
                        );
                        ptr::copy_nonoverlapping(
                            u_cur,
                            u_ptr.add(g.processed_len - g.deleted_cnt),
                            1,
                        );
                        ptr::copy_nonoverlapping(
                            v_cur,
                            v_ptr.add(g.processed_len - g.deleted_cnt),
                            1,
                        );
                    }
                }
                g.processed_len += 1;
            }
        }

        // Stage 1: Nothing was deleted.
        process_loop::<F, T, U, V, false>(original_len, &mut f, &mut g);

        // Stage 2: Some elements were deleted.
        process_loop::<F, T, U, V, true>(original_len, &mut f, &mut g);

        // All item are processed. This can be optimized to `set_len` by LLVM.
        drop(g);
    }

    pub fn push(&mut self, values: (T, U, V)) {
        let len = self.len();
        if len == self.capacity() {
            self.buffer.grow_one();
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.as_mut_ptrs();
            let t_ptr = t_ptr.add(len);
            let u_ptr = u_ptr.add(len);
            let v_ptr = v_ptr.add(len);

            ptr::write(t_ptr, values.0);
            ptr::write(u_ptr, values.1);
            ptr::write(v_ptr, values.2);

            self.set_len(len + 1);
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

    pub fn clear(&mut self) {
        let (t_ptr, u_ptr, v_ptr) = self.as_mut_slices();
        let (t_ptr, u_ptr, v_ptr) = (t_ptr as *mut [_], u_ptr as *mut [_], v_ptr as *mut [_]);

        unsafe {
            self.set_len(0);
            ptr::drop_in_place(t_ptr);
            ptr::drop_in_place(u_ptr);
            ptr::drop_in_place(v_ptr);
        }
    }
}

impl<T, U, V> Debug for SoaVec<T, U, V>
where
    T: Debug,
    U: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (t_slice, u_slice, v_slice) = self.as_slices();
        f.debug_struct("SoaVec")
            .field("t_slice", &t_slice)
            .field("u_slice", &u_slice)
            .field("v_slice", &v_slice)
            .finish()
    }
}

impl<T, U, V> Default for SoaVec<T, U, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, U, V> AsRef<SoaVec<T, U, V>> for SoaVec<T, U, V> {
    fn as_ref(&self) -> &SoaVec<T, U, V> {
        self
    }
}

impl<T, U, V> AsRef<SoaSlice<T, U, V>> for SoaVec<T, U, V> {
    fn as_ref(&self) -> &SoaSlice<T, U, V> {
        self
    }
}

impl<T, U, V> AsMut<SoaVec<T, U, V>> for SoaVec<T, U, V> {
    fn as_mut(&mut self) -> &mut SoaVec<T, U, V> {
        self
    }
}

impl<T, U, V> AsMut<SoaSlice<T, U, V>> for SoaVec<T, U, V> {
    fn as_mut(&mut self) -> &mut SoaSlice<T, U, V> {
        self
    }
}

impl<T, U, V> Borrow<SoaSlice<T, U, V>> for SoaVec<T, U, V> {
    fn borrow(&self) -> &SoaSlice<T, U, V> {
        self
    }
}

impl<T, U, V> BorrowMut<SoaSlice<T, U, V>> for SoaVec<T, U, V> {
    fn borrow_mut(&mut self) -> &mut SoaSlice<T, U, V> {
        self
    }
}

impl<T, U, V> Hash for SoaVec<T, U, V>
where
    T: Hash,
    U: Hash,
    V: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<T, U, V> Deref for SoaVec<T, U, V> {
    type Target = SoaSlice<T, U, V>;

    fn deref(&self) -> &Self::Target {
        let (data, len_in_bytes) = match min_size_of::<T, U, V>() {
            0 => (addr_of!(self.len).cast(), size_of::<usize>()),
            _ => (self.as_ptr(), self.capacity_in_bytes()),
        };
        unsafe { from_len_in_bytes(data, len_in_bytes) }
    }
}

impl<T, U, V> DerefMut for SoaVec<T, U, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let (data, len_in_bytes) = match min_size_of::<T, U, V>() {
            0 => (addr_of_mut!(self.len).cast(), size_of::<usize>()),
            _ => (self.as_mut_ptr(), self.capacity_in_bytes()),
        };
        unsafe { from_len_in_bytes_mut(data, len_in_bytes) }
    }
}

impl<'a, T, U, V> IntoIterator for &'a SoaVec<T, U, V> {
    type Item = (&'a T, &'a U, &'a V);
    type IntoIter = Iter<'a, T, U, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, U, V> IntoIterator for &'a mut SoaVec<T, U, V> {
    type Item = (&'a mut T, &'a mut U, &'a mut V);
    type IntoIter = IterMut<'a, T, U, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, U, V> IntoIterator for SoaVec<T, U, V> {
    type Item = (T, U, V);
    type IntoIter = IntoIter<T, U, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl<T, U, V> Drop for SoaVec<T, U, V> {
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
    use super::SoaVec;

    #[test]
    fn check_null_opt() {
        type SoaVec = super::SoaVec<u32, u16, u8>;
        assert_eq!(size_of::<Option<SoaVec>>(), size_of::<SoaVec>());
    }

    #[test]
    fn new() {
        let vec = SoaVec::<u32, u16, u8>::new();
        assert!(vec.is_empty());
        assert_eq!(vec.capacity(), 0);

        let slice = vec.as_slice();
        assert!(slice.is_empty());
        assert_eq!(slice.capacity(), 0);

        let boxed_slice = vec.into_boxed_slice();
        assert!(boxed_slice.is_empty());
        assert_eq!(boxed_slice.capacity(), 0);

        let vec = boxed_slice.into_vec();
        assert!(vec.is_empty());
        assert_eq!(vec.capacity(), 0);

        let boxed_slice = vec.into_boxed_slice();
        assert!(boxed_slice.is_empty());
        assert_eq!(boxed_slice.capacity(), 0);

        let into_iter = boxed_slice.into_iter();
        assert!(into_iter.is_empty());
    }

    #[test]
    fn with_capacity() {
        let vec = SoaVec::<u8, u64, u16>::with_capacity(10);
        assert!(vec.is_empty());
        assert!(vec.capacity() >= 10);

        let slice = vec.as_slice();
        assert!(slice.is_empty());
        assert!(slice.capacity() >= 10);

        let boxed_slice = vec.into_boxed_slice();
        assert!(boxed_slice.is_empty());
        assert_eq!(boxed_slice.capacity(), 0);

        let vec = boxed_slice.into_vec();
        assert!(vec.is_empty());
        assert_eq!(vec.capacity(), 0);

        let boxed_slice = vec.into_boxed_slice();
        assert!(boxed_slice.is_empty());
        assert_eq!(boxed_slice.capacity(), 0);

        let into_iter = boxed_slice.into_iter();
        assert!(into_iter.is_empty());
    }

    #[test]
    fn one_item() {
        let mut vec = SoaVec::<u8, u32, u16>::new();
        vec.push((1, 2, 3));
        assert_eq!(vec.len(), 1);
        assert!(vec.capacity() >= 1);

        let slice = vec.as_slice();
        assert_eq!(slice.len(), 1);
        assert!(slice.capacity() >= 1);
        assert_eq!(
            slice.as_slices(),
            ([1].as_slice(), [2].as_slice(), [3].as_slice()),
        );
        assert_eq!(slice.get(0), Some((&1, &2, &3)));

        let mut iter = vec.iter();
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some((&1, &2, &3)));
        assert_eq!(iter.next(), None);

        let (t, u, v) = vec.pop().expect("multi vector should not be empty");
        assert_eq!((t, u, v), (1, 2, 3));
        assert!(vec.is_empty());
        assert!(vec.capacity() >= 1);
        assert_eq!(vec.get(0), None);

        let boxed_slice = vec.into_boxed_slice();
        assert!(boxed_slice.is_empty());
        assert_eq!(boxed_slice.capacity(), 0);

        let vec = boxed_slice.into_vec();
        assert!(vec.is_empty());
        assert_eq!(vec.capacity(), 0);

        let boxed_slice = vec.into_boxed_slice();
        assert!(boxed_slice.is_empty());
        assert_eq!(boxed_slice.capacity(), 0);

        let into_iter = boxed_slice.into_iter();
        assert!(into_iter.is_empty());
    }

    #[test]
    fn three_items() {
        let mut vec = SoaVec::<u16, String, u128>::new();
        vec.insert(0, (1, "2".to_owned(), 3));
        vec.insert(0, (4, "5".to_owned(), 6));
        vec.insert(1, (7, "8".to_owned(), 9));

        assert_eq!(vec.len(), 3);
        assert!(vec.capacity() >= 3);

        let slice = vec.as_slice();
        assert_eq!(slice.len(), 3);
        assert!(slice.capacity() >= 3);
        assert_eq!(
            slice.as_slices(),
            (
                [4, 7, 1].as_slice(),
                ["5".to_owned(), "8".to_owned(), "2".to_owned()].as_slice(),
                [6, 9, 3].as_slice(),
            ),
        );
        assert_eq!(slice.get(0), Some((&4, &"5".to_owned(), &6)));
        assert_eq!(slice.get(1), Some((&7, &"8".to_owned(), &9)));
        assert_eq!(slice.get(2), Some((&1, &"2".to_owned(), &3)));
        assert_eq!(
            slice.get(0..),
            Some((
                [4, 7, 1].as_slice(),
                ["5".to_owned(), "8".to_owned(), "2".to_owned()].as_slice(),
                [6, 9, 3].as_slice(),
            )),
        );

        for (t, _, _) in &mut vec {
            *t += 1;
        }

        let mut iter = vec.iter_mut();
        assert_eq!(iter.len(), 3);

        assert_eq!(iter.next(), Some((&mut 5, &mut "5".to_owned(), &mut 6)));
        assert_eq!(iter.len(), 2);

        assert_eq!(
            iter.next_back(),
            Some((&mut 2, &mut "2".to_owned(), &mut 3)),
        );
        assert_eq!(iter.len(), 1);

        assert_eq!(iter.next(), Some((&mut 8, &mut "8".to_owned(), &mut 9)));
        assert_eq!(iter.len(), 0);

        assert_eq!(iter.next_back(), None);

        let (t, u, v) = vec.swap_remove(1);
        assert_eq!((t, u, v), (8, "8".to_owned(), 9));
        assert_eq!(vec.len(), 2);
        assert!(vec.capacity() >= 3);

        let (t, u, v) = vec.pop().expect("multi vector should not be empty");
        assert_eq!((t, u, v), (2, "2".to_owned(), 3));
        assert_eq!(vec.len(), 1);
        assert!(vec.capacity() >= 3);

        let (t, u, v) = vec.remove(0);
        assert_eq!((t, u, v), (5, "5".to_owned(), 6));
        assert!(vec.is_empty());
        assert!(vec.capacity() >= 3);

        vec.push((0, "0".to_owned(), 0));
        vec.push((0, "0".to_owned(), 0));
        vec.push((0, "0".to_owned(), 0));
        vec.reserve(1);
        assert!(vec.capacity() >= 4);
        vec.reserve_exact(6);
        assert!(vec.capacity() >= 9);

        vec.shrink_to(6);
        assert!(vec.capacity() >= 6);
        vec.shrink_to(0);
        assert!(vec.capacity() >= 3);

        vec.clear();
        assert!(vec.is_empty());
        assert!(vec.capacity() >= 3);

        vec.push((1, "2".to_owned(), 3));
        vec.push((4, "5".to_owned(), 6));
        vec.push((7, "8".to_owned(), 9));
        vec.retain_mut(|(x, _, _)| {
            if *x <= 3 {
                *x += 1;
                true
            } else {
                false
            }
        });
        assert_eq!(vec.len(), 1);
        assert!(vec.capacity() >= 3);
        assert_eq!(
            vec.as_slices(),
            ([2].as_slice(), ["2".to_owned()].as_slice(), [3].as_slice()),
        );

        let boxed_slice = vec.into_boxed_slice();
        assert_eq!(boxed_slice.len(), 1);
        assert_eq!(boxed_slice.capacity(), 1);
        assert_eq!(boxed_slice.get(0), Some((&2, &"2".to_owned(), &3)));

        let vec = boxed_slice.into_vec();
        assert_eq!(vec.len(), 1);
        assert!(vec.capacity() >= 1);
        assert_eq!(vec.get(0), Some((&2, &"2".to_owned(), &3)));

        let boxed_slice = vec.into_boxed_slice();
        assert_eq!(boxed_slice.len(), 1);
        assert_eq!(boxed_slice.capacity(), 1);
        assert_eq!(boxed_slice.get(0), Some((&2, &"2".to_owned(), &3)));

        let mut into_iter = boxed_slice.into_iter();
        assert_eq!(into_iter.len(), 1);
        assert_eq!(into_iter.next_back(), Some((2, "2".to_owned(), 3)));
        assert_eq!(into_iter.next(), None);
        assert_eq!(into_iter.next_back(), None);
    }

    #[test]
    fn three_items_zst() {
        #[derive(Debug, PartialEq, Eq)]
        struct ZST1;

        #[derive(Debug, PartialEq, Eq)]
        struct ZST2(());

        #[derive(Debug, PartialEq, Eq)]
        struct ZST3 {
            empty: (),
        }

        let mut vec = SoaVec::<ZST1, ZST2, ZST3>::new();
        vec.insert(0, (ZST1, ZST2(()), ZST3 { empty: () }));
        vec.insert(0, (ZST1, ZST2(()), ZST3 { empty: () }));
        vec.insert(1, (ZST1, ZST2(()), ZST3 { empty: () }));

        assert_eq!(vec.len(), 3);
        assert!(vec.capacity() >= 3);

        let slice = vec.as_slice();
        assert_eq!(slice.len(), 3);
        assert!(slice.capacity() >= 3);
        assert_eq!(
            slice.as_slices(),
            (
                [ZST1, ZST1, ZST1].as_slice(),
                [ZST2(()), ZST2(()), ZST2(())].as_slice(),
                [ZST3 { empty: () }, ZST3 { empty: () }, ZST3 { empty: () }].as_slice(),
            ),
        );
        assert_eq!(slice.get(0), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
        assert_eq!(slice.get(1), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
        assert_eq!(slice.get(2), Some((&ZST1, &ZST2(()), &ZST3 { empty: () })));
        assert_eq!(
            slice.get(0..),
            Some((
                [ZST1, ZST1, ZST1].as_slice(),
                [ZST2(()), ZST2(()), ZST2(())].as_slice(),
                [ZST3 { empty: () }, ZST3 { empty: () }, ZST3 { empty: () }].as_slice(),
            )),
        );

        let mut iter = vec.iter_mut();
        assert_eq!(iter.len(), 3);

        assert!(iter.next().is_some());
        assert_eq!(iter.len(), 2);

        assert!(iter.next_back().is_some());
        assert_eq!(iter.len(), 1);

        assert!(iter.next().is_some());
        assert_eq!(iter.len(), 0);

        assert!(iter.next_back().is_none());

        let (t, u, v) = vec.swap_remove(1);
        assert_eq!((t, u, v), (ZST1, ZST2(()), ZST3 { empty: () }));
        assert_eq!(vec.len(), 2);
        assert!(vec.capacity() >= 3);

        let (t, u, v) = vec.pop().expect("multi vector should not be empty");
        assert_eq!((t, u, v), (ZST1, ZST2(()), ZST3 { empty: () }));
        assert_eq!(vec.len(), 1);
        assert!(vec.capacity() >= 3);

        let (t, u, v) = vec.remove(0);
        assert_eq!((t, u, v), (ZST1, ZST2(()), ZST3 { empty: () }));
        assert!(vec.is_empty());
        assert!(vec.capacity() >= 3);

        vec.push((ZST1, ZST2(()), ZST3 { empty: () }));
        vec.push((ZST1, ZST2(()), ZST3 { empty: () }));
        vec.push((ZST1, ZST2(()), ZST3 { empty: () }));
        vec.reserve(1);
        assert!(vec.capacity() >= 4);
        vec.reserve_exact(6);
        assert!(vec.capacity() >= 9);

        vec.shrink_to(6);
        assert!(vec.capacity() >= 6);
        vec.shrink_to(0);
        assert!(vec.capacity() >= 3);

        vec.clear();
        assert!(vec.is_empty());
        assert!(vec.capacity() >= 3);

        vec.push((ZST1, ZST2(()), ZST3 { empty: () }));
        vec.push((ZST1, ZST2(()), ZST3 { empty: () }));
        vec.push((ZST1, ZST2(()), ZST3 { empty: () }));

        let mut idx = 0;
        vec.retain(|_| {
            let current = idx;
            idx += 1;
            current % 2 == 0
        });
        assert_eq!(vec.len(), 2);
        assert!(vec.capacity() >= 3);

        let boxed_slice = vec.into_boxed_slice();
        assert_eq!(boxed_slice.len(), 2);
        assert_eq!(boxed_slice.capacity(), usize::MAX);
        assert_eq!(
            boxed_slice.get(..),
            Some((
                [ZST1, ZST1].as_slice(),
                [ZST2(()), ZST2(())].as_slice(),
                [ZST3 { empty: () }, ZST3 { empty: () }].as_slice(),
            )),
        );

        let vec = boxed_slice.into_vec();
        assert_eq!(vec.len(), 2);
        assert!(vec.capacity() >= 2);

        let boxed_slice = vec.into_boxed_slice();
        assert_eq!(boxed_slice.len(), 2);
        assert_eq!(boxed_slice.capacity(), usize::MAX);

        let mut into_iter = boxed_slice.into_iter();
        assert_eq!(into_iter.len(), 2);
        assert_eq!(
            into_iter.next_back(),
            Some((ZST1, ZST2(()), ZST3 { empty: () })),
        );
        assert_eq!(into_iter.next(), Some((ZST1, ZST2(()), ZST3 { empty: () })));
        assert_eq!(into_iter.next_back(), None);
        assert_eq!(into_iter.next(), None);
    }
}
