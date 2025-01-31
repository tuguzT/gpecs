use alloc::boxed::Box;
use core::{
    borrow::{Borrow, BorrowMut},
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr,
};

pub use crate::raw_vec::{TryReserveError, TryReserveErrorKind};

use crate::{
    ptr::{actual_capacity, ptrs, BufferData},
    raw_vec::RawSoaVec,
    slice::{from_raw_parts, from_raw_parts_mut, Iter, IterMut, SoaSlice},
    soa::Soa,
};

pub use self::into_iter::IntoIter;

mod into_iter;

pub struct SoaVec<T>
where
    T: Soa,
{
    buffer: RawSoaVec<T>,
    len: usize,
}

impl<T> SoaVec<T>
where
    T: Soa,
{
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
    pub unsafe fn from_raw_parts(ptr: *mut BufferData<T>, len: usize, capacity: usize) -> Self {
        Self {
            buffer: unsafe { RawSoaVec::from_raw_parts(ptr, capacity) },
            len,
        }
    }

    pub(crate) const unsafe fn from_capacity_in_bytes(
        ptr: *mut BufferData<T>,
        len: usize,
        capacity_in_bytes: usize,
    ) -> Self {
        Self {
            buffer: unsafe { RawSoaVec::from_capacity_in_bytes(ptr, capacity_in_bytes) },
            len,
        }
    }

    pub fn into_raw_parts(self) -> (*mut BufferData<T>, usize, usize) {
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
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    fn move_right(&mut self, old_capacity: usize) {
        let new_capacity = self.capacity();
        if new_capacity <= old_capacity {
            return;
        }

        unsafe {
            let ptr = self.as_mut_ptr();
            let old_ptrs = T::ptrs_cast_const(ptrs::<T>(ptr, old_capacity));
            let new_ptrs = ptrs::<T>(ptr, new_capacity);

            T::ptrs_copy_rev(old_ptrs, new_ptrs, self.len());
        }
    }

    fn move_left(&mut self, new_capacity: usize) {
        let old_capacity = self.capacity();
        if new_capacity >= old_capacity {
            return;
        }

        unsafe {
            let ptr = self.as_mut_ptr();
            let old_ptrs = T::ptrs_cast_const(ptrs::<T>(ptr, old_capacity));
            let new_ptrs = ptrs::<T>(ptr, new_capacity);

            T::ptrs_copy(old_ptrs, new_ptrs, self.len());
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

        let new_capacity = actual_capacity::<T>(self.len);
        self.move_left(new_capacity);
        self.buffer.shrink_to_fit(new_capacity);
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        if self.capacity() <= min_capacity {
            return;
        }

        let new_capacity = actual_capacity::<T>(cmp::max(self.len, min_capacity));
        self.move_left(new_capacity);
        self.buffer.shrink_to_fit(new_capacity);
    }

    pub fn into_boxed_slice(mut self) -> Box<SoaSlice<T>> {
        self.shrink_to_fit();
        let me = ManuallyDrop::new(self);

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
            let ptrs = T::ptrs_add_mut(self.as_mut_ptrs(), len);
            let slices = T::slices_from_raw_parts_mut(ptrs, remaining_len);

            self.set_len(len);
            T::slices_drop_in_place(slices);
        }
    }

    pub fn as_slice(&self) -> &SoaSlice<T> {
        self
    }

    pub fn as_mut_slice(&mut self) -> &mut SoaSlice<T> {
        self
    }

    pub const fn as_ptr(&self) -> *const BufferData<T> {
        self.buffer.ptr().cast_const()
    }

    pub const fn as_mut_ptr(&mut self) -> *mut BufferData<T> {
        self.buffer.ptr()
    }

    pub fn as_ptrs(&self) -> T::Ptrs {
        let ptrs = self.buffer.ptrs();
        T::ptrs_cast_const(ptrs)
    }

    pub fn as_mut_ptrs(&mut self) -> T::MutPtrs {
        self.buffer.ptrs()
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_> {
        let ptrs = self.as_ptrs();
        let len = self.len();

        let slices = T::slices_from_raw_parts(ptrs, len);
        unsafe { T::slices_as_refs(slices) }
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> T::SlicesMut<'_> {
        let ptrs = self.as_mut_ptrs();
        let len = self.len();

        let slices = T::slices_from_raw_parts_mut(ptrs, len);
        unsafe { T::mut_slices_as_refs(slices) }
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

    pub fn swap_remove(&mut self, index: usize) -> T {
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
            let ptrs = self.as_mut_ptrs();
            let value = {
                let ptrs = T::ptrs_add_mut(ptrs, index);
                T::ptrs_read(T::ptrs_cast_const(ptrs))
            };

            T::ptrs_copy(
                T::ptrs_add(T::ptrs_cast_const(ptrs), len - 1),
                T::ptrs_add_mut(ptrs, index),
                1,
            );

            self.set_len(len - 1);
            value
        }
    }

    pub fn insert(&mut self, index: usize, elements: T) {
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
            let ptrs = self.as_mut_ptrs();
            let ptrs = T::ptrs_add_mut(ptrs, index);

            if index < len {
                let src = T::ptrs_cast_const(ptrs);
                let dst = T::ptrs_add_mut(ptrs, 1);
                T::ptrs_copy(src, dst, len - index);
            }
            T::ptrs_write(ptrs, elements);

            self.set_len(len + 1);
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
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
            let ptrs = self.as_mut_ptrs();
            let ptrs = T::ptrs_add_mut(ptrs, index);

            let value = T::ptrs_read(T::ptrs_cast_const(ptrs));

            T::ptrs_copy(
                T::ptrs_add(T::ptrs_cast_const(ptrs), 1),
                ptrs,
                len - index - 1,
            );
            self.set_len(len - 1);

            value
        }
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(T::Refs<'_>) -> bool,
    {
        self.retain_mut(|refs| {
            let refs = T::mut_refs_as_refs(refs);
            f(refs)
        });
    }

    pub fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(T::RefsMut<'_>) -> bool,
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
        struct BackshiftOnDrop<'a, T>
        where
            T: Soa,
        {
            v: &'a mut SoaVec<T>,
            processed_len: usize,
            deleted_cnt: usize,
            original_len: usize,
        }

        impl<T> Drop for BackshiftOnDrop<'_, T>
        where
            T: Soa,
        {
            fn drop(&mut self) {
                if self.deleted_cnt > 0 {
                    // SAFETY: Trailing unchecked items must be valid since we never touch them.
                    unsafe {
                        let ptrs = self.v.as_mut_ptrs();
                        T::ptrs_copy(
                            T::ptrs_add(T::ptrs_cast_const(ptrs), self.processed_len),
                            T::ptrs_add_mut(ptrs, self.processed_len - self.deleted_cnt),
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

        fn process_loop<F, T, const DELETED: bool>(
            original_len: usize,
            f: &mut F,
            g: &mut BackshiftOnDrop<'_, T>,
        ) where
            T: Soa,
            F: FnMut(T::RefsMut<'_>) -> bool,
        {
            while g.processed_len != original_len {
                // SAFETY: Unchecked element must be valid.
                let cur = unsafe {
                    let ptrs = g.v.as_mut_ptrs();
                    T::ptrs_add_mut(ptrs, g.processed_len)
                };
                let res = {
                    let cur = unsafe { T::as_mut_refs(cur) };
                    !f(cur)
                };
                if res {
                    // Advance early to avoid double drop if `drop_in_place` panicked.
                    g.processed_len += 1;
                    g.deleted_cnt += 1;
                    // SAFETY: We never touch this element again after dropped.
                    unsafe {
                        T::ptrs_drop_in_place(cur);
                    }
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
                        let ptrs = g.v.as_mut_ptrs();
                        T::ptrs_copy_nonoverlapping(
                            T::ptrs_cast_const(cur),
                            T::ptrs_add_mut(ptrs, g.processed_len - g.deleted_cnt),
                            1,
                        );
                    }
                }
                g.processed_len += 1;
            }
        }

        // Stage 1: Nothing was deleted.
        process_loop::<F, T, false>(original_len, &mut f, &mut g);

        // Stage 2: Some elements were deleted.
        process_loop::<F, T, true>(original_len, &mut f, &mut g);

        // All item are processed. This can be optimized to `set_len` by LLVM.
        drop(g);
    }

    pub fn push(&mut self, values: T) {
        let len = self.len();
        if len == self.capacity() {
            self.buffer.grow_one();
        }

        unsafe {
            let ptrs = self.as_mut_ptrs();
            let ptrs = T::ptrs_add_mut(ptrs, len);

            T::ptrs_write(ptrs, values);
            self.set_len(len + 1);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let len = self.len();
        if len == 0 {
            return None;
        }

        unsafe {
            let ptrs = self.as_ptrs();
            let ptrs = T::ptrs_add(ptrs, len - 1);

            let value = T::ptrs_read(ptrs);
            self.set_len(len - 1);

            Some(value)
        }
    }

    pub fn clear(&mut self) {
        let slices = self.as_mut_slices();
        let slices = T::mut_slice_refs_as_ptrs(slices);

        unsafe {
            self.set_len(0);
            T::slices_drop_in_place(slices);
        }
    }
}

impl<T> Debug for SoaVec<T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("SoaVec").field(&slices).finish()
    }
}

impl<T> Default for SoaVec<T>
where
    T: Soa,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AsRef<SoaVec<T>> for SoaVec<T>
where
    T: Soa,
{
    fn as_ref(&self) -> &SoaVec<T> {
        self
    }
}

impl<T> AsRef<SoaSlice<T>> for SoaVec<T>
where
    T: Soa,
{
    fn as_ref(&self) -> &SoaSlice<T> {
        self
    }
}

impl<T> AsMut<SoaVec<T>> for SoaVec<T>
where
    T: Soa,
{
    fn as_mut(&mut self) -> &mut SoaVec<T> {
        self
    }
}

impl<T> AsMut<SoaSlice<T>> for SoaVec<T>
where
    T: Soa,
{
    fn as_mut(&mut self) -> &mut SoaSlice<T> {
        self
    }
}

impl<T> Borrow<SoaSlice<T>> for SoaVec<T>
where
    T: Soa,
{
    fn borrow(&self) -> &SoaSlice<T> {
        self
    }
}

impl<T> BorrowMut<SoaSlice<T>> for SoaVec<T>
where
    T: Soa,
{
    fn borrow_mut(&mut self) -> &mut SoaSlice<T> {
        self
    }
}

impl<T> Hash for SoaVec<T>
where
    T: Soa,
    for<'any> T::Slices<'any>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<T> Deref for SoaVec<T>
where
    T: Soa,
{
    type Target = SoaSlice<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { from_raw_parts(self.as_ptr(), self.len(), self.capacity()) }
    }
}

impl<T> DerefMut for SoaVec<T>
where
    T: Soa,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { from_raw_parts_mut(self.as_mut_ptr(), self.len(), self.capacity()) }
    }
}

impl<'a, T> IntoIterator for &'a SoaVec<T>
where
    T: Soa,
{
    type Item = T::Refs<'a>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SoaVec<T>
where
    T: Soa,
{
    type Item = T::RefsMut<'a>;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for SoaVec<T>
where
    T: Soa,
{
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl<T> Drop for SoaVec<T>
where
    T: Soa,
{
    fn drop(&mut self) {
        if self.is_empty() {
            return;
        }

        let ptrs = self.as_mut_ptrs();
        let len = self.len();

        let slices = T::slices_from_raw_parts_mut(ptrs, len);
        unsafe { T::slices_drop_in_place(slices) }
    }
}
