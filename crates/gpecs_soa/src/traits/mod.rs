use core::{
    alloc::{Layout, LayoutError},
    ptr::NonNull,
};

pub use gpecs_soa_core::traits::*;

use crate::{
    field::{self, BufferLayout, FieldLayouts, FieldLayoutsOutput, FieldLayoutsOwned},
    layout::WithLayout,
};

pub use self::tuple::*;

mod identity;
mod tuple;
mod unit;

/// An extension of [SoA context](SoaRawContext) type which allows
/// to declare properties needed for buffer allocation & buffer memory manipulation.
///
/// # Safety
///
/// - [Field layouts](FieldLayouts::Output) **MUST** accurately describe each stored field.
/// - Count of such layouts **MUST** be non-zero & equal to the number of stored fields.
/// - Order of such layouts **MUST** resemble their order inside of a buffer in memory.
///
/// Note that the order of [pointers](SoaRawContext::Ptrs) & their derivatives
/// **may not** resemble their order inside of a buffer in memory.
/// Reordering of such pointers in other methods is up to the implementation of this trait.
pub unsafe trait SoaAllocContext<T>:
    SoaRawContext<T> + FieldLayoutsOwned<T> + Sized
where
    T: ?Sized,
{
    /// Calculates layout needed to store `capacity` number of fields inside of a buffer.
    ///
    /// This layout should not include self, as it is handled by the crate itself.
    fn buffer_layout(&self, capacity: usize) -> Result<Layout, LayoutError> {
        let fields = self.field_layouts();
        field::buffer_layout(fields, capacity).map(BufferLayout::layout)
    }

    /// Calculates an alignment for any possible [buffer layout](Self::buffer_layout).
    fn buffer_align(&self) -> usize {
        self.field_layouts()
            .into_iter()
            .map(|item| item.layout().align())
            .max()
            .unwrap_or(1)
    }

    /// Calculates sum of all the fields' sizes.
    /// Returns [`None`] on integer overflow.
    fn packed_size_of_fields(&self) -> Option<usize> {
        self.field_layouts()
            .into_iter()
            .map(|item| item.layout().size())
            .try_fold(0, usize::checked_add)
    }

    /// Retrieves maximum number of sets of fields which can be stored inside of a buffer with given layout.
    fn capacity_from(&self, buffer_layout: Layout) -> usize {
        let Some(packed_size) = self.packed_size_of_fields() else {
            return 0;
        };

        let buffer_size = buffer_layout.size();
        let Some(max_capacity) = buffer_size.checked_div(packed_size) else {
            return usize::MAX;
        };

        let mut capacity = max_capacity;
        while {
            let layout = self
                .buffer_layout(capacity)
                .expect("new layout should be smaller than the buffer one");
            layout.size() > buffer_size
        } {
            capacity = capacity.strict_sub(1);
        }
        capacity
    }

    /// Creates [pointers](SoaRawContext::Ptrs) to each stored field
    /// from a given buffer with given capacity.
    ///
    /// Implementations of this method should not account for `Self`,
    /// as it is handled by the crate itself.
    ///
    /// # Safety
    ///
    /// Layout from a given pointer to a buffer to the end of the allocation of such buffer
    /// must be the same as the one returned by [`buffer_layout()`](SoaAllocContext::buffer_layout) method.
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_>;

    /// Creates [mutable pointers](SoaRawContext::MutPtrs) to each stored field
    /// from a given buffer with given capacity.
    ///
    /// Implementations of this method should not account for `Self`,
    /// as it is handled by the crate itself.
    ///
    /// # Safety
    ///
    /// Layout from a given pointer to a buffer to the end of the allocation of such buffer
    /// must be the same as the one returned by [`buffer_layout()`](SoaAllocContext::buffer_layout) method.
    unsafe fn mut_ptrs_from_buffer(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_> {
        let ptrs = unsafe { self.ptrs_from_buffer(buffer, capacity) };
        self.ptrs_cast_mut(ptrs)
    }

    /// Creates [non-null pointers](SoaRawContext::NonNullPtrs) to each stored field
    /// from a given buffer with given capacity.
    ///
    /// Implementations of this method should not account for `Self`,
    /// as it is handled by the crate itself.
    ///
    /// # Safety
    ///
    /// Layout from a given pointer to a buffer to the end of the allocation of such buffer
    /// must be the same as the one returned by [`buffer_layout()`](SoaAllocContext::buffer_layout) method.
    unsafe fn nonnull_ptrs_from_buffer(
        &self,
        buffer: NonNull<u8>,
        capacity: usize,
    ) -> Self::NonNullPtrs<'_> {
        let ptrs = unsafe { self.mut_ptrs_from_buffer(buffer.as_ptr(), capacity) };
        unsafe { self.nonnull_ptrs_from_mut_ptrs(ptrs) }
    }

    /// Copies `count * size_of::<fields[0]>() + ...` bytes from [src](SoaRawContext::Ptrs) to [dst](SoaRawContext::MutPtrs)
    /// for each stored field sequentially in the *same* order as they are stored in a buffer.
    ///
    /// The source and destination may overlap, but all the pointers corresponding to the same collection of fields
    /// may not overlap with each other.
    ///
    /// Additionally, all the safety requirements resulting from applying
    /// [`ptr::copy()`](core::ptr::copy) method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// If the source and destination will *never* overlap,
    /// [`ptrs_copy_nonoverlapping()`](SoaRawContext::ptrs_copy_nonoverlapping) can be used instead.
    unsafe fn ptrs_copy_forward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, count: usize);

    /// Copies `count * size_of::<fields[0]>() + ...` bytes from [src](SoaRawContext::NonNullPtrs) to [dst](SoaRawContext::NonNullPtrs)
    /// for each stored field sequentially in the *same* order as they are stored in a buffer.
    ///
    /// The source and destination may overlap, but all the pointers corresponding to the same collection of fields
    /// may not overlap with each other.
    ///
    /// Additionally, all the safety requirements resulting from applying
    /// [`ptr::copy()`](core::ptr::copy) method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// If the source and destination will *never* overlap,
    /// [`nonnull_ptrs_copy_nonoverlapping()`](SoaRawContext::nonnull_ptrs_copy_nonoverlapping) can be used instead.
    unsafe fn nonnull_ptrs_copy_forward(
        &self,
        src: Self::NonNullPtrs<'_>,
        dst: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        let src = Self::nonnull_ptrs_upcast(src);
        let dst = Self::nonnull_ptrs_upcast(dst);

        let src = self.nonnull_ptrs_as_ptrs(src);
        let dst = self.nonnull_ptrs_as_mut_ptrs(dst);
        unsafe { self.ptrs_copy_forward(src, dst, count) }
    }

    /// Copies `count * size_of::<fields[0]>() + ...` bytes from [src](SoaRawContext::Ptrs) to [dst](SoaRawContext::MutPtrs)
    /// for each stored field sequentially in the *reverse* order as they are stored in a buffer.
    ///
    /// The source and destination may overlap, but all the pointers corresponding to the same collection of fields
    /// may not overlap with each other.
    ///
    /// Additionally, all the safety requirements resulting from applying
    /// [`ptr::copy()`](core::ptr::copy) method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// If the source and destination will *never* overlap,
    /// [`ptrs_copy_nonoverlapping()`](SoaRawContext::ptrs_copy_nonoverlapping) can be used instead.
    unsafe fn ptrs_copy_backward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, count: usize);

    /// Copies `count * size_of::<fields[0]>() + ...` bytes from [src](SoaRawContext::NonNullPtrs) to [dst](SoaRawContext::NonNullPtrs)
    /// for each stored field sequentially in the *reverse* order as they are stored in a buffer.
    ///
    /// The source and destination may overlap, but all the pointers corresponding to the same collection of fields
    /// may not overlap with each other.
    ///
    /// Additionally, all the safety requirements resulting from applying
    /// [`ptr::copy()`](core::ptr::copy) method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// If the source and destination will *never* overlap,
    /// [`nonnull_ptrs_copy_nonoverlapping()`](SoaRawContext::nonnull_ptrs_copy_nonoverlapping) can be used instead.
    unsafe fn nonnull_ptrs_copy_backward(
        &self,
        src: Self::NonNullPtrs<'_>,
        dst: Self::NonNullPtrs<'_>,
        count: usize,
    ) {
        let src = Self::nonnull_ptrs_upcast(src);
        let dst = Self::nonnull_ptrs_upcast(dst);

        let src = self.nonnull_ptrs_as_ptrs(src);
        let dst = self.nonnull_ptrs_as_mut_ptrs(dst);
        unsafe { self.ptrs_copy_backward(src, dst, count) }
    }
}

/// An extension of [SoA](SoaRaw) type which allows to
/// declare properties needed for buffer allocation & buffer memory manipulation.
pub unsafe trait SoaAlloc: SoaRaw<Context: SoaAllocContext<Self>, Fields: Sized> {}

unsafe impl<T> SoaAlloc for T
where
    T: SoaRaw + ?Sized,
    T::Context: SoaAllocContext<T>,
    T::Fields: Sized,
{
}

#[inline]
pub fn field_layouts<T>(context: &T::Context) -> FieldLayoutsOutput<'_, T::Context, T>
where
    T: SoaAlloc + ?Sized,
{
    field::field_layouts::<T, T::Context>(context)
}

#[inline]
pub fn buffer_layout<T>(context: &T::Context, capacity: usize) -> Result<Layout, LayoutError>
where
    T: SoaAlloc + ?Sized,
{
    context.buffer_layout(capacity)
}

#[inline]
pub fn buffer_align<T>(context: &T::Context) -> usize
where
    T: SoaAlloc + ?Sized,
{
    context.buffer_align()
}

#[inline]
pub fn packed_size_of_fields<T>(context: &T::Context) -> Option<usize>
where
    T: SoaAlloc + ?Sized,
{
    context.packed_size_of_fields()
}

#[inline]
pub fn capacity_from<T>(context: &T::Context, buffer_layout: Layout) -> usize
where
    T: SoaAlloc + ?Sized,
{
    context.capacity_from(buffer_layout)
}

/// Marker trait which places additional safety requirements
/// on the [`Fields`](SoaRaw::Fields) associated type of [SoA](SoaRaw) type.
///
/// These safety requirements are:
/// - sum of sizes of [field layouts](FieldLayouts::Output)
///   should be less or equal to the size of [`Fields`](SoaRaw::Fields)
/// - each alignment from [field layouts](FieldLayouts::Output)
///   should be less or equal to the alignment of [`Fields`](SoaRaw::Fields)
pub unsafe trait SoaAllocTrusted: SoaAlloc {}
