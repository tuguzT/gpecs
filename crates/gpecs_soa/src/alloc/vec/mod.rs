use core::{
    borrow::{Borrow, BorrowMut},
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    mem::{ManuallyDrop, forget},
    ops::{Deref, DerefMut, Index, IndexMut, RangeBounds},
    ptr::{self, addr_of},
};
use core_alloc::boxed::Box;

pub use super::raw_vec::{TryReserveError, TryReserveErrorKind};

use crate::{
    layout::{BufferData, buffer_layout, capacity_from, should_allocate},
    ptr::{BufferDataPtrMut, ptrs_from_buffer, ptrs_from_buffer_mut},
    slice::{
        IndexHelper, IndexHelperMut, Iter, IterMut, RawIter, RawIterMut, SoaSlice, SoaSliceMutPtrs,
        SoaSlicePtrs, SoaSlicePtrsIndex, SoaSlices, SoaSlicesMut, from_raw_parts,
        from_raw_parts_mut, range,
    },
    traits::{
        MutPtrs, Ptrs, RawSoaContext, SliceMutPtrs, SlicePtrs, Soa, SoaRead, SoaToOwned,
        SoaTrustedFields, SoaWrite,
    },
};

use super::{raw_vec::RawSoaVec, set_len_on_drop::SetLenOnDrop};

pub use self::{drain::Drain, into_iter::IntoIter};

mod drain;
mod into_iter;
mod partial_eq;
mod partial_ord;

pub struct SoaVec<T>
where
    T: Soa + ?Sized,
{
    buffer: RawSoaVec<T>,
    len: usize,
}

impl<T> SoaVec<T>
where
    T: Soa + ?Sized,
{
    #[inline]
    #[must_use]
    pub fn new() -> Self
    where
        T::Context: Default,
    {
        Self::with_capacity(0)
    }

    #[inline]
    #[must_use]
    pub fn with_context(context: T::Context) -> Self {
        Self::with_context_and_capacity(context, 0)
    }

    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self
    where
        T::Context: Default,
    {
        Self::with_context_and_capacity(Default::default(), capacity)
    }

    #[inline]
    pub fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError>
    where
        T::Context: Default,
    {
        Self::try_with_context_and_capacity(Default::default(), capacity)
    }

    #[inline]
    pub fn with_context_and_capacity(context: T::Context, capacity: usize) -> Self {
        let mut me = Self {
            buffer: RawSoaVec::with_capacity(context, capacity),
            len: 0,
        };

        me.set_len_in_buffer(0);
        me
    }

    #[inline]
    pub fn try_with_context_and_capacity(
        context: T::Context,
        capacity: usize,
    ) -> Result<Self, TryReserveError> {
        let mut me = Self {
            buffer: RawSoaVec::try_with_capacity(context, capacity)?,
            len: 0,
        };

        me.set_len_in_buffer(0);
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut BufferData<T>, len: usize, capacity: usize) -> Self {
        let buffer = unsafe { RawSoaVec::from_raw_parts(ptr, capacity) };
        Self { buffer, len }
    }

    #[inline]
    pub fn into_raw_parts(self) -> (*mut BufferData<T>, usize, usize) {
        let mut me = ManuallyDrop::new(self);
        (me.as_mut_ptr(), me.len(), me.capacity())
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { buffer, .. } = self;
        buffer.capacity()
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let Self { buffer, .. } = self;
        buffer.context()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const BufferData<T> {
        let Self { buffer, .. } = self;
        buffer.as_mut_ptr().cast_const()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut BufferData<T> {
        let Self { buffer, .. } = self;
        buffer.as_mut_ptr()
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'_, T> {
        let (_, ptrs) = self.as_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&T::Context, Ptrs<'_, T>) {
        let Self { buffer, .. } = self;

        let context = buffer.context();
        let ptrs = buffer.as_mut_ptrs();
        let ptrs = context.ptrs_cast_const(ptrs);
        (context, ptrs)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'_, T> {
        let (_, ptrs) = self.as_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&T::Context, MutPtrs<'_, T>) {
        let Self { buffer, .. } = self;

        let context = buffer.context();
        let ptrs = buffer.as_mut_ptrs();
        (context, ptrs)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'_, T> {
        let (_, slices) = self.as_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&T::Context, SlicePtrs<'_, T>) {
        let len = self.len();
        let (context, ptrs) = self.as_ptrs_with_context();

        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub fn as_slice_mut_ptrs(&mut self) -> SliceMutPtrs<'_, T> {
        let (_, slices) = self.as_slice_mut_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_mut_ptrs_with_context(&mut self) -> (&T::Context, SliceMutPtrs<'_, T>) {
        let len = self.len();
        let (context, ptrs) = self.as_mut_ptrs_with_context();

        let slices = context.slice_mut_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub fn slice_ptrs(&self) -> SoaSlicePtrs<'_, T> {
        let (context, slices) = self.as_slice_ptrs_with_context();
        SoaSlicePtrs::new(context, slices)
    }

    #[inline]
    pub fn slice_mut_ptrs(&mut self) -> SoaSliceMutPtrs<'_, T> {
        let (context, slices) = self.as_slice_mut_ptrs_with_context();
        SoaSliceMutPtrs::new(context, slices)
    }

    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());

        self.len = new_len;
        self.set_len_in_buffer(new_len);
    }

    #[inline]
    fn set_len_in_buffer(&mut self, new_len: usize) {
        let context = self.context();
        let capacity = self.capacity();
        if !should_allocate::<T>(context, capacity) {
            return;
        }

        unsafe {
            let len = self.as_mut_ptr().ptr_to_len_mut();
            ptr::write(len, new_len);
        }
    }

    #[inline]
    fn move_right(&mut self, old_capacity: usize) {
        let new_capacity = self.capacity();
        if new_capacity <= old_capacity {
            return;
        }

        let ptr = self.as_mut_ptr();
        let context = self.context();

        let old_ptrs = unsafe { ptrs_from_buffer::<T>(context, ptr, old_capacity) };
        let new_ptrs = self.buffer.as_mut_ptrs();

        let len = self.len();
        unsafe { context.ptrs_copy_rev(old_ptrs, new_ptrs, len) }
    }

    #[inline]
    fn move_left(&mut self, new_capacity: usize) {
        let old_capacity = self.capacity();
        if new_capacity >= old_capacity {
            return;
        }

        let ptr = self.as_mut_ptr();
        let context = self.context();

        let old_ptrs = self.as_ptrs();
        let new_ptrs = unsafe { ptrs_from_buffer_mut::<T>(context, ptr, new_capacity) };

        let len = self.len();
        unsafe { context.ptrs_copy(old_ptrs, new_ptrs, len) }
    }

    pub fn reserve(&mut self, additional: usize) {
        let len = self.len();
        let old_capacity = self.capacity();
        let Self { buffer, .. } = self;

        if !buffer.needs_to_grow(len, additional) {
            return;
        }
        buffer.reserve(len, additional);

        match old_capacity {
            0 => self.set_len_in_buffer(0),
            _ => self.move_right(old_capacity),
        }
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let len = self.len();
        let old_capacity = self.capacity();
        let Self { buffer, .. } = self;

        if !buffer.needs_to_grow(len, additional) {
            return;
        }
        buffer.reserve_exact(len, additional);

        match old_capacity {
            0 => self.set_len_in_buffer(0),
            _ => self.move_right(old_capacity),
        }
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let len = self.len();
        let old_capacity = self.capacity();
        let Self { buffer, .. } = self;

        if !buffer.needs_to_grow(len, additional) {
            return Ok(());
        }
        buffer.try_reserve(len, additional)?;

        match old_capacity {
            0 => self.set_len_in_buffer(0),
            _ => self.move_right(old_capacity),
        }
        Ok(())
    }

    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let len = self.len();
        let old_capacity = self.capacity();
        let Self { buffer, .. } = self;

        if !buffer.needs_to_grow(len, additional) {
            return Ok(());
        }
        buffer.try_reserve_exact(len, additional)?;

        match old_capacity {
            0 => self.set_len_in_buffer(0),
            _ => self.move_right(old_capacity),
        }
        Ok(())
    }

    pub fn shrink_to_fit(&mut self) {
        let len = self.len();
        if self.capacity() <= len {
            return;
        }

        let context = self.context();
        let new_capacity = actual_capacity::<T>(context, len);
        self.move_left(new_capacity);
        self.buffer.shrink_to_fit(new_capacity);
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        if self.capacity() <= min_capacity {
            return;
        }

        let context = self.context();
        let new_capacity = actual_capacity::<T>(context, cmp::max(self.len(), min_capacity));
        self.move_left(new_capacity);
        self.buffer.shrink_to_fit(new_capacity);
    }

    pub fn truncate(&mut self, len: usize) {
        let old_len = self.len();
        if len > old_len {
            return;
        }

        unsafe {
            self.set_len(len);
        }

        let (context, ptrs) = self.as_mut_ptrs_with_context();
        let ptrs = unsafe { context.ptrs_add_mut(ptrs, len) };
        let slices = context.slice_mut_ptrs_from_raw_parts(ptrs, old_len - len);
        unsafe { context.slices_drop_in_place(slices) }
    }

    #[inline]
    pub fn clear(&mut self) {
        let len = self.len();
        unsafe {
            self.set_len(0);
        }

        let (context, ptrs) = self.as_mut_ptrs_with_context();
        let slices = context.slice_mut_ptrs_from_raw_parts(ptrs, len);
        unsafe { context.slices_drop_in_place(slices) }
    }

    #[inline]
    pub fn raw_iter(&self) -> RawIter<'_, T> {
        let (_, iter) = self.raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn raw_iter_with_context(&self) -> (&T::Context, RawIter<'_, T>) {
        self.slice_ptrs().into_iter_with_context()
    }

    #[inline]
    pub fn raw_iter_mut(&mut self) -> RawIterMut<'_, T> {
        let (_, iter) = self.raw_iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn raw_iter_mut_with_context(&mut self) -> (&T::Context, RawIterMut<'_, T>) {
        self.slice_mut_ptrs().into_iter_with_context()
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_, '_> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&T::Context, T::Slices<'_, '_>) {
        let (context, slices) = self.as_slice_ptrs_with_context();
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> T::SlicesMut<'_, '_> {
        let (_, slices) = self.as_mut_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_mut_slices_with_context(&mut self) -> (&T::Context, T::SlicesMut<'_, '_>) {
        let (context, slices) = self.as_slice_mut_ptrs_with_context();
        let slices = unsafe { T::slice_mut_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn slices(&self) -> SoaSlices<'_, '_, T> {
        unsafe { self.slice_ptrs().deref() }
    }

    #[inline]
    pub fn slices_mut(&mut self) -> SoaSlicesMut<'_, '_, T> {
        unsafe { self.slice_mut_ptrs().deref_mut() }
    }

    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T::Context, T::Refs<'_, '_>) -> bool,
    {
        self.retain_mut(|context, refs| {
            let refs = T::upcast_refs_mut(refs);
            let refs = T::refs_mut_as_refs(context, refs);
            f(context, refs)
        });
    }

    pub fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&T::Context, T::RefsMut<'_, '_>) -> bool,
    {
        let original_len = self.len();
        // Avoid double drop if the drop guard is not executed,
        // since we may make some holes during the process.
        unsafe {
            self.set_len(0);
        }

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
        #[expect(clippy::items_after_statements)]
        struct BackshiftOnDrop<'a, T>
        where
            T: Soa + ?Sized,
        {
            v: &'a mut SoaVec<T>,
            processed_len: usize,
            deleted_cnt: usize,
            original_len: usize,
        }

        #[expect(clippy::items_after_statements)]
        impl<T> Drop for BackshiftOnDrop<'_, T>
        where
            T: Soa + ?Sized,
        {
            fn drop(&mut self) {
                let Self {
                    ref mut v,
                    processed_len,
                    deleted_cnt,
                    original_len,
                } = *self;

                if deleted_cnt > 0 {
                    let (context, ptrs) = v.as_mut_ptrs_with_context();
                    // SAFETY: Trailing unchecked items must be valid since we never touch them.
                    unsafe {
                        let src = context.ptrs_cast_const(ptrs.clone());
                        let src = context.ptrs_add(src, processed_len);
                        let dst = context.ptrs_add_mut(ptrs, processed_len - deleted_cnt);
                        context.ptrs_copy(src, dst, original_len - processed_len);
                    }
                }
                // SAFETY: After filling holes, all items are in contiguous memory.
                unsafe {
                    v.set_len(original_len - deleted_cnt);
                }
            }
        }

        let mut g = BackshiftOnDrop {
            v: self,
            processed_len: 0,
            deleted_cnt: 0,
            original_len,
        };

        #[expect(clippy::items_after_statements)]
        fn process_loop<F, T, const DELETED: bool>(
            original_len: usize,
            f: &mut F,
            g: &mut BackshiftOnDrop<'_, T>,
        ) where
            T: Soa + ?Sized,
            F: FnMut(&T::Context, T::RefsMut<'_, '_>) -> bool,
        {
            while g.processed_len != original_len {
                let (context, ptrs) = g.v.as_mut_ptrs_with_context();
                // SAFETY: Unchecked element must be valid.
                let cur = unsafe { context.ptrs_add_mut(ptrs.clone(), g.processed_len) };
                let res = unsafe {
                    let cur = T::ptrs_to_refs_mut(context, cur.clone());
                    !f(context, cur)
                };
                if res {
                    // Advance early to avoid double drop if `drop_in_place` panicked.
                    g.processed_len += 1;
                    g.deleted_cnt += 1;
                    // SAFETY: We never touch this element again after dropped.
                    unsafe { context.ptrs_drop_in_place(cur) }

                    // We already advanced the counter.
                    if DELETED {
                        continue;
                    }
                    break;
                }

                if DELETED {
                    // SAFETY: `deleted_cnt` > 0, so the hole slot must not overlap with current element.
                    // We use copy for move, and never touch this element again.
                    unsafe {
                        let src = context.ptrs_cast_const(cur);
                        let dst = context.ptrs_add_mut(ptrs, g.processed_len - g.deleted_cnt);
                        context.ptrs_copy_nonoverlapping(src, dst, 1);
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

    #[inline]
    pub fn iter(&self) -> Iter<'_, '_, T> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&self) -> (&T::Context, Iter<'_, '_, T>) {
        let (context, iter) = self.raw_iter_with_context();
        let iter = unsafe { iter.deref() };
        (context, iter)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, '_, T> {
        let (_, iter) = self.iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn iter_mut_with_context(&mut self) -> (&T::Context, IterMut<'_, '_, T>) {
        let (context, iter) = self.raw_iter_mut_with_context();
        let iter = unsafe { iter.deref_mut() };
        (context, iter)
    }

    #[inline]
    #[track_caller]
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, T>
    where
        R: RangeBounds<usize>,
    {
        Drain::new(self, range)
    }

    pub fn swap_remove_into<F, R>(&mut self, index: usize, f: F) -> R
    where
        F: FnOnce(&T::Context, Ptrs<'_, T>) -> R,
    {
        #[cold]
        #[inline(never)]
        #[track_caller]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("swap_remove index (which is {index}) should be < len (which is {len})")
        }

        let len = self.len();
        if index >= len {
            assert_failed(index, len);
        }

        let (context, ptrs) = self.as_mut_ptrs_with_context();
        let dst = unsafe { context.ptrs_add_mut(ptrs.clone(), index) };

        let ptrs_into = context.ptrs_cast_const(dst.clone());
        let result = f(context, ptrs_into);

        unsafe {
            let src = context.ptrs_cast_const(ptrs);
            let src = context.ptrs_add(src, len - 1);
            context.ptrs_copy(src, dst, 1);
        }

        unsafe {
            self.set_len(len - 1);
        }

        result
    }

    pub fn remove_into<F, R>(&mut self, index: usize, f: F) -> R
    where
        F: FnOnce(&T::Context, Ptrs<'_, T>) -> R,
    {
        #[cold]
        #[inline(never)]
        #[track_caller]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("removal index (which is {index}) should be < len (which is {len})");
        }

        let len = self.len();
        if index >= len {
            assert_failed(index, len);
        }

        let (context, ptrs) = self.as_mut_ptrs_with_context();
        let dst = unsafe { context.ptrs_add_mut(ptrs, index) };

        let ptrs_into = context.ptrs_cast_const(dst.clone());
        let result = f(context, ptrs_into);

        unsafe {
            let src = context.ptrs_cast_const(dst.clone());
            let src = context.ptrs_add(src, 1);
            context.ptrs_copy(src, dst, len - index - 1);
        }

        unsafe {
            self.set_len(len - 1);
        }

        result
    }

    pub fn pop_into<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&T::Context, Option<Ptrs<'_, T>>) -> R,
    {
        let len = self.len();
        if len == 0 {
            return f(self.context(), None);
        }

        let (context, ptrs) = self.as_ptrs_with_context();
        let ptrs_into = unsafe { context.ptrs_add(ptrs, len - 1) };
        let result = f(context, Some(ptrs_into));

        unsafe {
            self.set_len(len - 1);
        }

        result
    }

    pub fn insert_from<F, R>(&mut self, index: usize, f: F) -> R
    where
        F: FnOnce(&T::Context, MutPtrs<'_, T>) -> R,
    {
        #[cold]
        #[inline(never)]
        #[track_caller]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("insertion index (which is {index}) should be <= len (which is {len})");
        }

        let len = self.len();
        if index > len {
            assert_failed(index, len);
        }

        let capacity = self.capacity();
        if len == capacity {
            self.buffer.grow_one();

            match capacity {
                0 => self.set_len_in_buffer(0),
                _ => self.move_right(capacity),
            }
        }

        if index < len {
            let (context, ptrs) = self.as_mut_ptrs_with_context();
            let ptrs = unsafe { context.ptrs_add_mut(ptrs, index) };

            let src = context.ptrs_cast_const(ptrs.clone());
            let dst = unsafe { context.ptrs_add_mut(ptrs, 1) };
            unsafe { context.ptrs_copy(src, dst, len - index) }
        }

        #[expect(clippy::items_after_statements)]
        struct CopyBackGuard<'a, T>
        where
            T: Soa + ?Sized,
        {
            v: &'a mut SoaVec<T>,
            index: usize,
        }

        #[expect(clippy::items_after_statements)]
        impl<T> Drop for CopyBackGuard<'_, T>
        where
            T: Soa + ?Sized,
        {
            fn drop(&mut self) {
                let Self { ref mut v, index } = *self;
                let len = v.len();

                let (context, ptrs) = v.as_mut_ptrs_with_context();
                let dst = unsafe { context.ptrs_add_mut(ptrs, index) };

                if index < len {
                    let src = context.ptrs_cast_const(dst.clone());
                    let src = unsafe { context.ptrs_add(src, 1) };
                    unsafe { context.ptrs_copy_rev(src, dst, len - index) }
                }
            }
        }

        let guard = CopyBackGuard { v: self, index };
        let (context, ptrs) = guard.v.as_mut_ptrs_with_context();

        let ptrs_from = unsafe { context.ptrs_add_mut(ptrs, index) };
        let result = f(context, ptrs_from);

        unsafe {
            guard.v.set_len(len + 1);
        }
        forget(guard);

        result
    }

    pub fn push_from<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&T::Context, MutPtrs<'_, T>) -> R,
    {
        let len = self.len();
        let capacity = self.capacity();
        if len == capacity {
            self.buffer.grow_one();

            match capacity {
                0 => self.set_len_in_buffer(0),
                _ => self.move_right(capacity),
            }
        }

        let (context, ptrs) = self.as_mut_ptrs_with_context();
        let ptrs_from = unsafe { context.ptrs_add_mut(ptrs, len) };
        let result = f(context, ptrs_from);

        unsafe {
            self.set_len(len + 1);
        }

        result
    }
}

impl<T> SoaVec<T>
where
    T: SoaToOwned + SoaWrite,
{
    #[track_caller]
    pub fn extend_from_within<R>(&mut self, src: R)
    where
        R: RangeBounds<usize>,
    {
        let local_len = self.len();
        let range = range(src, ..local_len);
        self.reserve(range.len());

        let mut set_len_on_drop = SetLenOnDrop {
            local_len,
            vec: self,
        };

        let (context, slices) = set_len_on_drop.vec.as_slice_mut_ptrs_with_context();
        let dst = context.slice_mut_ptrs_as_ptrs(slices.clone());

        let slices = context.slice_ptrs_cast_const(slices);
        let slices = unsafe { SoaSlicePtrsIndex::<T>::get_unchecked(range, context, slices) };
        for src in RawIter::<T>::new(context, slices) {
            unsafe {
                let refs = T::ptrs_to_refs(context, src);
                let dst = context.ptrs_add_mut(dst.clone(), set_len_on_drop.local_len);
                T::write(context, dst, T::to_owned(context, refs));
            }
            set_len_on_drop.local_len += 1;
        }
    }
}

impl<T> SoaVec<T>
where
    T: Soa + SoaRead,
{
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.swap_remove_into(index, |context, src| unsafe { T::read(context, src) })
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        self.remove_into(index, |context, src| unsafe { T::read(context, src) })
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.pop_into(|context, src| unsafe { T::read(context, src?).into() })
    }
}

impl<T> SoaVec<T>
where
    T: Soa + SoaWrite,
{
    #[inline]
    pub fn insert(&mut self, index: usize, value: T) {
        self.insert_from(index, |context, dst| unsafe {
            T::write(context, dst, value);
        });
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        self.push_from(|context, dst| unsafe {
            T::write(context, dst, value);
        });
    }
}

impl<T> SoaVec<T>
where
    T: Soa + SoaTrustedFields + ?Sized,
{
    #[inline]
    pub fn as_slice(&self) -> &SoaSlice<T>
    where
        T: SoaTrustedFields,
    {
        self
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut SoaSlice<T>
    where
        T: SoaTrustedFields,
    {
        self
    }

    #[must_use]
    pub fn into_boxed_slice(mut self) -> Box<SoaSlice<T>> {
        self.shrink_to_fit();
        let me = ManuallyDrop::new(self);

        let buffer = unsafe { ptr::read(addr_of!(me.buffer)) };
        let len = me.len;
        unsafe { buffer.into_box(len) }
    }
}

impl<T> SoaVec<T>
where
    T: SoaTrustedFields + SoaToOwned + SoaWrite,
{
    #[track_caller]
    pub fn extend_from_slice(&mut self, other: &SoaSlice<T>) {
        self.reserve(other.len());

        let mut set_len_on_drop = SetLenOnDrop {
            local_len: self.len(),
            vec: self,
        };

        let (context, slices) = set_len_on_drop.vec.as_slice_mut_ptrs_with_context();
        let dst = context.slice_mut_ptrs_as_ptrs(slices.clone());

        let slices = context.slice_ptrs_cast_const(slices);
        for src in RawIter::<T>::new(context, slices) {
            unsafe {
                let refs = T::ptrs_to_refs(context, src);
                let dst = context.ptrs_add_mut(dst.clone(), set_len_on_drop.local_len);
                T::write(context, dst, T::to_owned(context, refs));
            }
            set_len_on_drop.local_len += 1;
        }
    }
}

impl<T> Debug for SoaVec<T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("SoaVec").field(&slices).finish()
    }
}

impl<T> Default for SoaVec<T>
where
    T: Soa + ?Sized,
    T::Context: Default,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AsRef<Self> for SoaVec<T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T> AsRef<SoaSlice<T>> for SoaVec<T>
where
    T: Soa + SoaTrustedFields + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &SoaSlice<T> {
        self
    }
}

impl<T> AsMut<Self> for SoaVec<T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T> AsMut<SoaSlice<T>> for SoaVec<T>
where
    T: Soa + SoaTrustedFields + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut SoaSlice<T> {
        self
    }
}

impl<T> Borrow<SoaSlice<T>> for SoaVec<T>
where
    T: Soa + SoaTrustedFields + ?Sized,
{
    #[inline]
    fn borrow(&self) -> &SoaSlice<T> {
        self
    }
}

impl<T> BorrowMut<SoaSlice<T>> for SoaVec<T>
where
    T: Soa + SoaTrustedFields + ?Sized,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut SoaSlice<T> {
        self
    }
}

impl<T> Eq for SoaVec<T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Eq,
{
}

impl<T> Ord for SoaVec<T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let this = self.as_slices();
        let other = other.as_slices();
        Ord::cmp(&this, &other)
    }
}

impl<T> Hash for SoaVec<T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let slices = self.slices();
        Hash::hash(&slices, state);
    }
}

impl<T> Clone for SoaVec<T>
where
    T: SoaToOwned + SoaWrite,
    T::Context: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        self.slices().to_vec()
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        let src = &source.slices();
        self.slices_mut().clone_from_slices(src);
    }
}

impl<T> Deref for SoaVec<T>
where
    T: Soa + SoaTrustedFields + ?Sized,
{
    type Target = SoaSlice<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let data = self.as_ptr();
        let len = self.len();
        let capacity = self.capacity();
        unsafe { from_raw_parts(data, len, capacity) }
    }
}

impl<T> DerefMut for SoaVec<T>
where
    T: Soa + SoaTrustedFields + ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let data = self.as_mut_ptr();
        let len = self.len();
        let capacity = self.capacity();
        unsafe { from_raw_parts_mut(data, len, capacity) }
    }
}

impl<T, U, I> Index<I> for SoaVec<T>
where
    T: Soa + ?Sized,
    U: ?Sized,
    for<'c, 'any> I: IndexHelper<'c, 'any, T, Output = U>,
{
    type Output = U;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.slices().into_index(index)
    }
}

impl<T, U, I> IndexMut<I> for SoaVec<T>
where
    T: Soa + ?Sized,
    U: ?Sized,
    for<'c, 'any> I: IndexHelperMut<'c, 'any, T, Output = U>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.slices_mut().into_index_mut(index)
    }
}

impl<T> Extend<T> for SoaVec<T>
where
    T: Soa + SoaWrite,
{
    #[inline]
    #[track_caller]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        // This is the case for a general iterator.
        //
        // This function should be the moral equivalent of:
        //
        //      for item in iterator {
        //          self.push(item);
        //      }
        let mut iter = iter.into_iter();
        while let Some(element) = iter.next() {
            let len = self.len();
            if len == self.capacity() {
                let (lower, _) = iter.size_hint();
                self.reserve(lower.saturating_add(1));
            }

            let (context, ptrs) = self.as_mut_ptrs_with_context();
            unsafe {
                let dst = context.ptrs_add_mut(ptrs, len);
                T::write(context, dst, element);
            }

            unsafe {
                // Since next() executes user code which can panic we have to bump the length after each step.
                // NB can't overflow since we would have had to alloc the address space
                self.set_len(len + 1);
            }
        }
    }
}

impl<T> From<Box<SoaSlice<T>>> for SoaVec<T>
where
    T: Soa + SoaTrustedFields + ?Sized,
{
    #[inline]
    fn from(value: Box<SoaSlice<T>>) -> Self {
        value.into_vec()
    }
}

impl<T> From<&SoaSlice<T>> for SoaVec<T>
where
    T: SoaTrustedFields + SoaToOwned + SoaWrite,
    T::Context: Clone,
{
    #[inline]
    fn from(value: &SoaSlice<T>) -> Self {
        value.to_vec()
    }
}

impl<T> From<&mut SoaSlice<T>> for SoaVec<T>
where
    T: SoaTrustedFields + SoaToOwned + SoaWrite,
    T::Context: Clone,
{
    #[inline]
    fn from(value: &mut SoaSlice<T>) -> Self {
        value.to_vec()
    }
}

impl<'r, T> IntoIterator for &'r SoaVec<T>
where
    T: Soa + ?Sized,
{
    type Item = T::Refs<'r, 'r>;
    type IntoIter = Iter<'r, 'r, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'r, T> IntoIterator for &'r mut SoaVec<T>
where
    T: Soa + ?Sized,
{
    type Item = T::RefsMut<'r, 'r>;
    type IntoIter = IterMut<'r, 'r, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for SoaVec<T>
where
    T: Soa + SoaRead,
{
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl<T> FromIterator<T> for SoaVec<T>
where
    T: Soa + SoaWrite,
    T::Context: Default,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        // Unroll the first iteration, as the vector is going to be
        // expanded on this iteration in every case when the iterable is not
        // empty, but the loop in extend() is not going to see the
        // vector being full in the few subsequent loop iterations.
        // So we get better branch prediction.
        let mut iter = iter.into_iter();
        let Some(first) = iter.next() else {
            return Self::new();
        };

        let (lower, _) = iter.size_hint();
        let context = Default::default();
        let initial_capacity = cmp::max(
            RawSoaVec::<T>::min_non_zero_cap(&context),
            lower.saturating_add(1),
        );

        let mut vector = Self::with_context_and_capacity(context, initial_capacity);
        let (context, dst) = vector.as_mut_ptrs_with_context();
        unsafe {
            // SAFETY: We requested capacity at least 1
            T::write(context, dst, first);
            vector.set_len(1);
        }

        vector.extend(iter);
        vector
    }
}

impl<T> Drop for SoaVec<T>
where
    T: Soa + ?Sized,
{
    fn drop(&mut self) {
        if self.is_empty() {
            return;
        }

        let (context, slices) = self.as_slice_mut_ptrs_with_context();
        unsafe { context.slices_drop_in_place(slices) }
    }
}

#[inline]
fn actual_capacity<T>(context: &T::Context, capacity: usize) -> usize
where
    T: Soa + ?Sized,
{
    let buffer_layout =
        buffer_layout::<T>(context, capacity).expect("layout size should not exceed `isize::MAX`");
    capacity_from::<T>(context, buffer_layout)
}
