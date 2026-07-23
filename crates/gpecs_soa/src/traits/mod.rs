use core::alloc::{Layout, LayoutError};

pub use gpecs_soa_core::traits::*;

use crate::{
    buffer::packed_size_of_fields,
    field::{BufferLayout, FieldLayouts, FieldLayoutsOwned, buffer_layout},
};

pub use self::tuple::*;

mod identity;
mod tuple;
mod unit;

/// An extension of [SoA context](RawSoaContext) type which allows
/// to declare properties needed for buffer allocation & buffer memory manipulation.
///
/// # Safety
///
/// - [Field layouts](FieldLayouts::Output) **MUST** accurately describe each stored field.
/// - Count of such layouts **MUST** be non-zero & equal to the number of stored fields.
/// - Order of such layouts **MUST** resemble their order inside of a buffer in memory.
///
/// Note that the order of [pointers](RawSoaContext::Ptrs) & their derivatives
/// **may not** resemble their order inside of a buffer in memory.
/// Reordering of such pointers in other methods is up to the implementation of this trait.
pub unsafe trait AllocSoaContext<T>:
    RawSoaContext<T> + FieldLayoutsOwned<T> + Sized
where
    T: ?Sized,
{
    /// Calculates layout needed to store `capacity` number of fields inside of a buffer.
    ///
    /// This layout should not include self, as it is handled by the crate itself.
    fn buffer_layout(&self, capacity: usize) -> Result<Layout, LayoutError> {
        let fields = self.field_layouts();
        self::buffer_layout(fields, capacity).map(BufferLayout::layout)
    }

    /// Retrieves maximum number of sets of fields which can be stored inside of a buffer with given layout.
    fn capacity_from(&self, buffer_layout: Layout) -> usize {
        let packed_size = packed_size_of_fields(self.field_layouts());
        let buffer_size = buffer_layout.size();
        let Some(max_capacity) = buffer_size.checked_div(packed_size) else {
            return usize::MAX;
        };

        let mut capacity = max_capacity;
        while {
            let layout = self
                .buffer_layout(capacity)
                .expect("new buffer layout should be smaller than the input one");
            layout.size() > buffer_size
        } {
            capacity -= 1;
        }
        capacity
    }

    /// Creates [pointers](RawSoaContext::Ptrs) to each stored field
    /// from a given buffer with given capacity.
    ///
    /// Implementations of this method should not account for `Self`,
    /// as it is handled by the crate itself.
    ///
    /// # Safety
    ///
    /// Layout from a given pointer to a buffer to the end of the allocation of such buffer
    /// must be the same as the one returned by [`buffer_layout()`](AllocSoaContext::buffer_layout) method.
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_>;

    /// Creates [mutable pointers](RawSoaContext::MutPtrs) to each stored field
    /// from a given buffer with given capacity.
    ///
    /// Implementations of this method should not account for `Self`,
    /// as it is handled by the crate itself.
    ///
    /// # Safety
    ///
    /// Layout from a given pointer to a buffer to the end of the allocation of such buffer
    /// must be the same as the one returned by [`buffer_layout()`](AllocSoaContext::buffer_layout) method.
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_>;

    /// Copies `count * size_of::<fields[0]>() + ...` bytes from [src](RawSoaContext::Ptrs) to [dst](RawSoaContext::MutPtrs)
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
    /// [`ptrs_copy_nonoverlapping()`](RawSoaContext::ptrs_copy_nonoverlapping) can be used instead.
    unsafe fn ptrs_copy_forward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, count: usize);

    /// Copies `count * size_of::<fields[0]>() + ...` bytes from [src](RawSoaContext::Ptrs) to [dst](RawSoaContext::MutPtrs)
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
    /// [`ptrs_copy_nonoverlapping()`](RawSoaContext::ptrs_copy_nonoverlapping) can be used instead.
    unsafe fn ptrs_copy_backward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, count: usize);
}

/// An extension of [SoA](RawSoa) type which allows to
/// declare properties needed for buffer allocation & buffer memory manipulation.
pub unsafe trait AllocSoa: RawSoa<Context: AllocSoaContext<Self>, Fields: Sized> {}

unsafe impl<T> AllocSoa for T
where
    T: RawSoa + ?Sized,
    T::Context: AllocSoaContext<T>,
    T::Fields: Sized,
{
}

/// Marker trait which places additional safety requirements
/// on the [`Fields`](RawSoa::Fields) associated type of [SoA](RawSoa) type.
///
/// These safety requirements are:
/// - sum of layouts' sizes of [field layouts](FieldLayouts::Output)
///   should be less or equal to the size of [`Fields`](RawSoa::Fields)
/// - alignment of each layout of [field layouts](FieldLayouts::Output)
///   should be less or equal to the alignment of [`Fields`](RawSoa::Fields)
pub unsafe trait AllocSoaTrusted: AllocSoa {}
