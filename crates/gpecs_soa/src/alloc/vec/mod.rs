use core::{
    borrow::{Borrow, BorrowMut},
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    mem::{transmute, ManuallyDrop},
    ops::{Deref, DerefMut, Index, IndexMut, RangeBounds},
    ptr,
};
use core_alloc::boxed::Box;

pub use super::raw_vec::{TryReserveError, TryReserveErrorKind};

use crate::{
    ptr::{
        buffer_layout, capacity_from, ptrs, should_allocate, BufferData, BufferDataPtr,
        BufferDataPtrMut,
    },
    slice::{
        from_raw_parts, from_raw_parts_mut, slice_range, IndexHelper, IndexHelperMut, Iter,
        IterMut, SoaSlice, SoaSlices, SoaSlicesMut,
    },
    traits::{Soa, SoaToOwned, SoaTrustedFields, SoaVecs},
};

use super::{raw_vec::RawSoaVec, set_len_on_drop::SetLenOnDrop};

pub use self::{drain::Drain, into_iter::IntoIter};

mod drain;
mod into_iter;
mod partial_eq;
mod partial_ord;

pub struct SoaVec<T>
where
    T: Soa,
{
    pub(super) buffer: RawSoaVec<T>,
    len: usize,
}

impl<T> SoaVec<T>
where
    T: Soa,
{
    #[inline]
    pub fn new() -> Self
    where
        T::Context: Default,
    {
        Self::with_capacity(0)
    }

    #[inline]
    pub fn with_context(context: T::Context) -> Self {
        Self::with_context_and_capacity(context, 0)
    }

    #[inline]
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
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn from_raw_parts(ptr: *mut BufferData<T>, len: usize, capacity: usize) -> Self {
        Self {
            buffer: unsafe { RawSoaVec::from_raw_parts(ptr, capacity) },
            len,
        }
    }

    #[inline]
    pub fn into_raw_parts(self) -> (*mut BufferData<T>, usize, usize) {
        let mut me = ManuallyDrop::new(self);
        (me.as_mut_ptr(), me.len(), me.capacity())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        self.buffer.context()
    }

    #[inline]
    fn move_right(&mut self, old_capacity: usize) {
        let new_capacity = self.capacity();
        if new_capacity <= old_capacity {
            return;
        }

        let ptr = self.as_mut_ptr();
        let context = self.context();

        let old_ptrs = unsafe {
            let ptrs = ptrs::<T>(context, ptr, old_capacity).unwrap_unchecked();
            T::ptrs_cast_const(context, ptrs)
        };
        let new_ptrs = self.buffer.ptrs();
        let context = self.context();
        unsafe {
            T::ptrs_copy_rev(context, old_ptrs, new_ptrs, self.len());
        }
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
        let new_ptrs = unsafe { ptrs::<T>(context, ptr, new_capacity).unwrap_unchecked() };

        unsafe {
            T::ptrs_copy(context, old_ptrs, new_ptrs, self.len());
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

        let context = self.context();
        let new_capacity = actual_capacity::<T>(context, self.len);
        self.move_left(new_capacity);
        self.buffer.shrink_to_fit(new_capacity);
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        if self.capacity() <= min_capacity {
            return;
        }

        let context = self.context();
        let new_capacity = actual_capacity::<T>(context, cmp::max(self.len, min_capacity));
        self.move_left(new_capacity);
        self.buffer.shrink_to_fit(new_capacity);
    }

    pub fn truncate(&mut self, len: usize) {
        if len > self.len {
            return;
        }

        let remaining_len = self.len - len;
        unsafe {
            self.set_len(len);

            let context = self.context();
            let ptrs = T::ptrs_add_mut(context, self.buffer.ptrs(), len);
            let slices = T::slices_from_raw_parts_mut(context, ptrs, remaining_len);
            let context = self.context();
            T::slices_drop_in_place(context, slices);
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const BufferData<T> {
        self.buffer.ptr().cast_const()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut BufferData<T> {
        self.buffer.ptr()
    }

    #[inline]
    pub fn as_ptrs(&self) -> T::Ptrs<'_> {
        let ptrs = self.buffer.ptrs();
        let context = self.context();
        T::ptrs_cast_const(context, ptrs)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> T::MutPtrs<'_> {
        self.buffer.ptrs()
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_, '_> {
        let ptrs = self.as_ptrs();
        let len = self.len();
        let context = self.context();

        let slices = T::slices_from_raw_parts(context, ptrs, len);
        unsafe { T::slice_ptrs_to_slices(context, slices) }
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> T::SlicesMut<'_, '_> {
        let ptrs = self.buffer.ptrs();
        let len = self.len();
        let context = self.context();

        let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
        unsafe { T::slice_ptrs_to_slices_mut(context, slices) }
    }

    #[inline]
    pub fn slices(&self) -> SoaSlices<T> {
        let context = self.context();
        let slices = self.as_slices();
        SoaSlices::new(context, slices)
    }

    #[inline]
    pub fn slices_mut(&mut self) -> SoaSlicesMut<T> {
        let context = unsafe { &*self.as_ptr().ptr_to_context() };
        let slices = self.as_mut_slices();
        SoaSlicesMut::new(context, slices)
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());

        self.len = new_len;
        self.set_len_in_buffer(new_len);
    }

    #[inline]
    fn set_len_in_buffer(&mut self, new_len: usize) {
        if !should_allocate::<T>(self.capacity()) {
            return;
        }

        unsafe {
            let len = self.as_mut_ptr().ptr_to_len_mut();
            ptr::write(len, new_len);
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
            let ptrs = self.buffer.ptrs();
            let context = self.context();
            let value = {
                let ptrs = T::ptrs_add_mut(context, ptrs.clone(), index);
                T::ptrs_read(context, T::ptrs_cast_const(context, ptrs))
            };

            T::ptrs_copy(
                context,
                T::ptrs_add(context, T::ptrs_cast_const(context, ptrs.clone()), len - 1),
                T::ptrs_add_mut(context, ptrs, index),
                1,
            );

            self.set_len(len - 1);
            value
        }
    }

    pub fn insert(&mut self, index: usize, value: T) {
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
            self.buffer.grow_one();

            match capacity {
                0 => self.set_len_in_buffer(0),
                _ => self.move_right(capacity),
            }
        }

        unsafe {
            let ptrs = self.buffer.ptrs();
            let context = self.context();
            let ptrs = T::ptrs_add_mut(context, ptrs, index);

            if index < len {
                let src = T::ptrs_cast_const(context, ptrs.clone());
                let dst = T::ptrs_add_mut(context, ptrs.clone(), 1);
                T::ptrs_copy(context, src, dst, len - index);
            }
            T::ptrs_write(context, ptrs, value);

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
            let ptrs = self.buffer.ptrs();
            let context = self.context();
            let ptrs = T::ptrs_add_mut(context, ptrs, index);

            let value = T::ptrs_read(context, T::ptrs_cast_const(context, ptrs.clone()));

            T::ptrs_copy(
                context,
                T::ptrs_add(context, T::ptrs_cast_const(context, ptrs.clone()), 1),
                ptrs,
                len - index - 1,
            );
            self.set_len(len - 1);

            value
        }
    }

    #[inline]
    pub fn retain<'me, F>(&'me mut self, mut f: F)
    where
        F: FnMut(T::Refs<'me, '_>) -> bool,
    {
        let context = ptr::from_ref(self.context());
        self.retain_mut(|refs| {
            let refs = T::mut_refs_as_refs(unsafe { &*context }, refs);
            f(refs)
        });
    }

    pub fn retain_mut<'me, F>(&'me mut self, mut f: F)
    where
        F: FnMut(T::RefsMut<'me, '_>) -> bool,
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
                        let ptrs = self.v.buffer.ptrs();
                        let context = self.v.context();

                        T::ptrs_copy(
                            context,
                            T::ptrs_add(
                                context,
                                T::ptrs_cast_const(context, ptrs.clone()),
                                self.processed_len,
                            ),
                            T::ptrs_add_mut(context, ptrs, self.processed_len - self.deleted_cnt),
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

        fn process_loop<'me, F, T, const DELETED: bool>(
            original_len: usize,
            f: &mut F,
            g: &mut BackshiftOnDrop<'_, T>,
        ) where
            T: Soa,
            F: FnMut(T::RefsMut<'me, '_>) -> bool,
        {
            while g.processed_len != original_len {
                let ptrs = g.v.buffer.ptrs();
                let context = g.v.context();
                // SAFETY: Unchecked element must be valid.
                let cur = unsafe { T::ptrs_add_mut(context, ptrs, g.processed_len) };
                let res = unsafe {
                    let cur = T::ptrs_to_refs_mut(context, cur.clone());
                    let cur = transmute(cur);
                    !f(cur)
                };
                if res {
                    // Advance early to avoid double drop if `drop_in_place` panicked.
                    g.processed_len += 1;
                    g.deleted_cnt += 1;
                    // SAFETY: We never touch this element again after dropped.
                    unsafe {
                        let context = g.v.context();
                        T::ptrs_drop_in_place(context, cur);
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
                        let ptrs = g.v.buffer.ptrs();
                        let context = g.v.context();
                        T::ptrs_copy_nonoverlapping(
                            context,
                            T::ptrs_cast_const(context, cur),
                            T::ptrs_add_mut(context, ptrs, g.processed_len - g.deleted_cnt),
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

    pub fn push(&mut self, value: T) {
        let len = self.len();
        let capacity = self.capacity();
        if len == capacity {
            self.buffer.grow_one();

            match capacity {
                0 => self.set_len_in_buffer(0),
                _ => self.move_right(capacity),
            }
        }

        unsafe {
            let ptrs = self.buffer.ptrs();
            let context = self.context();
            let ptrs = T::ptrs_add_mut(context, ptrs, len);

            T::ptrs_write(context, ptrs, value);
            self.set_len(len + 1);
        }
    }

    #[track_caller]
    pub fn extend_from_within<R>(&mut self, src: R)
    where
        R: RangeBounds<usize>,
        for<'c, 'any> T::Refs<'c, 'any>: SoaToOwned<'c, 'any, Owned = T>,
    {
        let range = slice_range(src, ..self.len());
        self.reserve(range.len());

        let mut set_len_on_drop = SetLenOnDrop {
            local_len: self.len(),
            vec: self,
        };

        let context = set_len_on_drop.vec.context();
        for index in range {
            unsafe {
                let slices = set_len_on_drop.vec.slices();
                let refs = T::ptrs_to_refs(context, slices.get_unchecked(index));
                let dst = T::ptrs_add_mut(
                    context,
                    set_len_on_drop.vec.buffer.ptrs(),
                    set_len_on_drop.local_len,
                );
                refs.clone_into_ptrs(context, dst);
            }
            set_len_on_drop.local_len += 1;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let len = self.len();
        if len == 0 {
            return None;
        }

        unsafe {
            let ptrs = self.as_ptrs();
            let context = self.context();
            let ptrs = T::ptrs_add(context, ptrs, len - 1);

            let value = T::ptrs_read(context, ptrs);
            self.set_len(len - 1);

            Some(value)
        }
    }

    #[inline]
    #[track_caller]
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, T>
    where
        R: RangeBounds<usize>,
    {
        Drain::new(self, range)
    }

    #[inline]
    pub fn clear(&mut self) {
        let len = self.len();
        unsafe { self.set_len(0) }

        let context = self.context();
        let ptrs = self.buffer.ptrs();
        let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
        unsafe { T::slices_drop_in_place(&*context, slices) }
    }
}

impl<T> SoaVec<T>
where
    T: SoaVecs,
{
    pub fn into_vecs(mut self) -> (T::Context, T::Vecs) {
        let len = self.len();
        let context = self.context();
        let mut vecs = T::vecs_with_capacity(context, len);

        unsafe {
            self.set_len(0);
        }

        let context = self.context();
        let src = self.as_ptrs();
        let dst = T::mut_vecs_as_ptrs(context, &mut vecs);
        unsafe {
            T::ptrs_copy_nonoverlapping(context, src, dst, len);
            T::vecs_set_len(context, &mut vecs, len);
        }

        let me = ManuallyDrop::new(self);
        let buffer = unsafe { ptr::read(&me.buffer) };
        let context = buffer.drop_buffer();
        (context, vecs)
    }

    pub fn from_vecs(context: T::Context, mut vecs: T::Vecs) -> Self {
        let len = T::vecs_len(&context, &vecs);
        let mut vec = Self::with_context_and_capacity(context, len);

        let context = vec.context();
        unsafe {
            T::vecs_set_len(context, &mut vecs, 0);
        }

        let src = T::vecs_as_ptrs(context, &vecs);
        let dst = vec.buffer.ptrs();
        let context = vec.context();
        unsafe {
            T::ptrs_copy_nonoverlapping(context, src, dst, len);
            vec.set_len(len);
        }
        vec
    }
}

impl<T> SoaVec<T>
where
    T: SoaTrustedFields,
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

    pub fn into_boxed_slice(mut self) -> Box<SoaSlice<T>> {
        self.shrink_to_fit();
        let me = ManuallyDrop::new(self);

        unsafe {
            let buffer = ptr::read(&me.buffer);
            let len = me.len;
            buffer.into_box(len)
        }
    }

    #[track_caller]
    pub fn extend_from_slice<'other>(&mut self, other: &'other SoaSlice<T>)
    where
        T::Refs<'other, 'other>: SoaToOwned<'other, 'other, Owned = T>,
    {
        self.reserve(other.len());

        let mut set_len_on_drop = SetLenOnDrop {
            local_len: self.len(),
            vec: self,
        };

        let context = set_len_on_drop.vec.context();
        for refs in other.iter() {
            unsafe {
                let dst = T::ptrs_add_mut(
                    context,
                    set_len_on_drop.vec.buffer.ptrs(),
                    set_len_on_drop.local_len,
                );
                refs.clone_into_ptrs(context, dst);
            }
            set_len_on_drop.local_len += 1;
        }
    }
}

impl<T> Debug for SoaVec<T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("SoaVec").field(&slices).finish()
    }
}

impl<T> Default for SoaVec<T>
where
    T: Soa,
    T::Context: Default,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AsRef<SoaVec<T>> for SoaVec<T>
where
    T: Soa,
{
    #[inline]
    fn as_ref(&self) -> &SoaVec<T> {
        self
    }
}

impl<T> AsRef<SoaSlice<T>> for SoaVec<T>
where
    T: SoaTrustedFields,
{
    #[inline]
    fn as_ref(&self) -> &SoaSlice<T> {
        self
    }
}

impl<T> AsMut<SoaVec<T>> for SoaVec<T>
where
    T: Soa,
{
    #[inline]
    fn as_mut(&mut self) -> &mut SoaVec<T> {
        self
    }
}

impl<T> AsMut<SoaSlice<T>> for SoaVec<T>
where
    T: SoaTrustedFields,
{
    #[inline]
    fn as_mut(&mut self) -> &mut SoaSlice<T> {
        self
    }
}

impl<T> Borrow<SoaSlice<T>> for SoaVec<T>
where
    T: SoaTrustedFields,
{
    #[inline]
    fn borrow(&self) -> &SoaSlice<T> {
        self
    }
}

impl<T> BorrowMut<SoaSlice<T>> for SoaVec<T>
where
    T: SoaTrustedFields,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut SoaSlice<T> {
        self
    }
}

impl<T> Eq for SoaVec<T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: Eq,
{
}

impl<T> Ord for SoaVec<T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        Ord::cmp(&self.as_slices(), &other.as_slices())
    }
}

impl<T> Hash for SoaVec<T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let slices = self.slices();
        Hash::hash(&slices, state)
    }
}

impl<T> Clone for SoaVec<T>
where
    T: Soa,
    T::Context: Clone,
    for<'c, 'any> T::Refs<'c, 'any>: SoaToOwned<'c, 'any, Owned = T> + 'any,
{
    #[inline]
    fn clone(&self) -> Self {
        self.slices().to_vec()
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.slices_mut().clone_from_slices(source.slices());
    }
}

impl<T> Deref for SoaVec<T>
where
    T: SoaTrustedFields,
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
    T: SoaTrustedFields,
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
    T: Soa,
    U: ?Sized,
    for<'c, 'any> I: IndexHelper<'c, 'any, T, Output = U>,
{
    type Output = U;

    fn index(&self, index: I) -> &Self::Output {
        self.slices().into_index(index)
    }
}

impl<T, U, I> IndexMut<I> for SoaVec<T>
where
    T: Soa,
    U: ?Sized,
    for<'c, 'any> I: IndexHelperMut<'c, 'any, T, Output = U>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.slices_mut().into_index_mut(index)
    }
}

impl<T> Extend<T> for SoaVec<T>
where
    T: Soa,
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

            let ptrs = self.buffer.ptrs();
            let context = self.context();
            unsafe {
                let dst = T::ptrs_add_mut(context, ptrs, len);
                T::ptrs_write(context, dst, element);
                // Since next() executes user code which can panic we have to bump the length
                // after each step.
                // NB can't overflow since we would have had to alloc the address space
                self.set_len(len + 1);
            }
        }
    }
}

impl<T> From<Box<SoaSlice<T>>> for SoaVec<T>
where
    T: SoaTrustedFields,
{
    #[inline]
    fn from(value: Box<SoaSlice<T>>) -> Self {
        value.into_vec()
    }
}

impl<'me, T> From<&'me SoaSlice<T>> for SoaVec<T>
where
    T: SoaTrustedFields,
    T::Context: Clone,
    T::Refs<'me, 'me>: SoaToOwned<'me, 'me, Owned = T>,
{
    #[inline]
    fn from(value: &'me SoaSlice<T>) -> Self {
        value.to_vec()
    }
}

impl<'me, T> From<&'me mut SoaSlice<T>> for SoaVec<T>
where
    T: SoaTrustedFields,
    T::Context: Clone,
    T::Refs<'me, 'me>: SoaToOwned<'me, 'me, Owned = T>,
{
    #[inline]
    fn from(value: &'me mut SoaSlice<T>) -> Self {
        value.to_vec()
    }
}

impl<'r, T> IntoIterator for &'r SoaVec<T>
where
    T: Soa,
{
    type Item = T::Refs<'r, 'r>;
    type IntoIter = Iter<'r, 'r, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.slices().into_iter()
    }
}

impl<'r, T> IntoIterator for &'r mut SoaVec<T>
where
    T: Soa,
{
    type Item = T::RefsMut<'r, 'r>;
    type IntoIter = IterMut<'r, 'r, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.slices_mut().into_iter()
    }
}

impl<T> IntoIterator for SoaVec<T>
where
    T: Soa,
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
    T: Soa,
    T::Context: Default,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        // Unroll the first iteration, as the vector is going to be
        // expanded on this iteration in every case when the iterable is not
        // empty, but the loop in extend() is not going to see the
        // vector being full in the few subsequent loop iterations.
        // So we get better branch prediction.
        let mut iter = iter.into_iter();
        let mut vector = match iter.next() {
            None => return SoaVec::new(),
            Some(element) => {
                let (lower, _) = iter.size_hint();
                let context = Default::default();
                let initial_capacity = cmp::max(
                    RawSoaVec::<T>::min_non_zero_cap(&context),
                    lower.saturating_add(1),
                );
                let mut vector = SoaVec::with_context_and_capacity(context, initial_capacity);
                unsafe {
                    // SAFETY: We requested capacity at least 1
                    let dst = vector.buffer.ptrs();
                    let context = vector.context();
                    T::ptrs_write(context, dst, element);
                    vector.set_len(1);
                }
                vector
            }
        };
        vector.extend(iter);
        vector
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

        let ptrs = self.buffer.ptrs();
        let len = self.len();
        let context = self.context();

        let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
        unsafe { T::slices_drop_in_place(context, slices) }
    }
}

#[inline]
fn actual_capacity<T>(context: &T::Context, capacity: usize) -> usize
where
    T: Soa,
{
    let buffer_layout =
        buffer_layout::<T>(context, capacity).expect("layout size should not exceed `isize::MAX`");
    capacity_from::<T>(context, buffer_layout)
}
