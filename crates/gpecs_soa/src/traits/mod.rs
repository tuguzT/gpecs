use core::alloc::{Layout, LayoutError};

use crate::{
    buffer::packed_size_of_fields,
    field::{FieldLayouts, FieldLayoutsOwned, buffer_layout},
};

pub use self::tuple::*;

mod tuple;
mod unit;

/// This trait is used to perform all raw pointer arithmetics for [SoA](RawSoa) types.
pub unsafe trait RawSoaContext<T>
where
    T: ?Sized,
{
    /// Collection of pointers to each stored field.
    type Ptrs<'a>: Clone;

    /// Restricts [pointers](RawSoaContext::Ptrs) to each stored field
    /// to be covariant over generic lifetime.
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short>;

    /// Returns dangling [pointers](RawSoaContext::Ptrs) to each stored field.
    fn ptrs_dangling(&self) -> Self::Ptrs<'_>;

    /// Adds an unsigned offset to each [pointer](RawSoaContext::Ptrs) of each stored field.
    ///
    /// All the safety requirements resulting from applying [`pointer::add()`] method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// [`pointer::add()`]: https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.add
    unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, offset: usize) -> Self::Ptrs<'a>;

    /// Calculates the distance between two [pointers](RawSoaContext::Ptrs)
    /// to each stored field within the same allocation.
    ///
    /// All the safety requirements resulting from applying [`pointer::offset_from()`] method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// Note that resulting offsets should be the same for all the fields,
    /// or else this method could panic.
    ///
    /// [`pointer::offset_from()`]: https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.offset_from
    unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize;

    /// Collection of mutable pointers to each stored field.
    type MutPtrs<'a>: Clone;

    /// Restricts [mutable pointers](RawSoaContext::MutPtrs) to each stored field
    /// to be covariant over generic lifetime.
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short>;

    /// Returns mutable dangling [pointers](RawSoaContext::MutPtrs) to each stored field.
    fn ptrs_dangling_mut(&self) -> Self::MutPtrs<'_>;

    /// Adds an unsigned offset to each [mutable pointer](RawSoaContext::MutPtrs) of each stored field.
    ///
    /// All the safety requirements resulting from applying [`pointer::add()`] method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// [`pointer::add()`]: https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.add-1
    unsafe fn ptrs_add_mut<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        offset: usize,
    ) -> Self::MutPtrs<'a>;

    /// Calculates the distance between two [mutable pointers](RawSoaContext::MutPtrs)
    /// to each stored field within the same allocation.
    ///
    /// All the safety requirements resulting from applying [`pointer::offset_from()`] method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// Note that resulting offsets should be the same for all the fields,
    /// or else this method could panic.
    ///
    /// [`pointer::offset_from()`]: https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.offset_from-1
    unsafe fn ptrs_offset_from_mut(&self, ptrs: Self::MutPtrs<'_>, origin: Self::Ptrs<'_>)
    -> isize;

    /// Converts [pointers](RawSoaContext::Ptrs) of each stored field
    /// to the [mutable ones](RawSoaContext::MutPtrs).
    fn ptrs_cast_const<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::Ptrs<'a>;

    /// Converts [mutable pointers](RawSoaContext::MutPtrs) of each stored field
    /// to the [const ones](RawSoaContext::Ptrs).
    fn ptrs_cast_mut<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::MutPtrs<'a>;

    /// Swaps the values at two [mutable locations](RawSoaContext::MutPtrs) of the stored fields
    /// sequentially in the *same* order as they are stored in the buffer,
    /// without deinitializing either.
    ///
    /// The source and destination may overlap, but all the pointers corresponding to the same collection of fields
    /// may not overlap with each other.
    ///
    /// Additionally, all the safety requirements resulting from applying
    /// [`ptr::swap()`](core::ptr::swap) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_swap(&self, a: Self::MutPtrs<'_>, b: Self::MutPtrs<'_>);

    /// Copies `count * size_of::<fields[0]>() + ...` bytes from [src](RawSoaContext::Ptrs) to [dst](RawSoaContext::MutPtrs)
    /// for each stored field sequentially in the *same* order as they are stored in the buffer.
    ///
    /// The source and destination may overlap, but all the pointers corresponding to the same collection of fields
    /// may not overlap with each other.
    ///
    /// Additionally, all the safety requirements resulting from applying
    /// [`ptr::copy()`](core::ptr::copy) method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// If the source and destination will never overlap,
    /// [`ptrs_copy_nonoverlapping()`](RawSoaContext::ptrs_copy_nonoverlapping) can be used instead.
    unsafe fn ptrs_copy_forward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize);

    /// Copies `count * size_of::<fields[0]>() + ...` bytes from [src](RawSoaContext::Ptrs) to [dst](RawSoaContext::MutPtrs)
    /// for each stored field sequentially in the *reverse* order as they are stored in the buffer.
    ///
    /// The source and destination may overlap, but all the pointers corresponding to the same collection of fields
    /// may not overlap with each other.
    ///
    /// Additionally, all the safety requirements resulting from applying
    /// [`ptr::copy()`](core::ptr::copy) method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// If the source and destination will never overlap,
    /// [`ptrs_copy_nonoverlapping()`](RawSoaContext::ptrs_copy_nonoverlapping) can be used instead.
    unsafe fn ptrs_copy_backward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize);

    /// Copies `count * size_of::<fields[0]>() + ...` bytes from [src](RawSoaContext::Ptrs) to [dst](RawSoaContext::MutPtrs)
    /// for each stored field sequentially in unspecified order.
    /// The source and destination, as well as all the field pointers, must not overlap.
    ///
    /// For regions of memory which might overlap, use
    /// [`ptrs_copy_forward()`](RawSoaContext::ptrs_copy_forward) or [`ptrs_copy_backward()`](RawSoaContext::ptrs_copy_backward) instead.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::copy_nonoverlapping()`](core::ptr::copy_nonoverlapping) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_copy_nonoverlapping(
        &self,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    );

    /// Executes the destructors (if any) for the each stored field located at input [pointers](RawSoaContext::Ptrs).
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::drop_in_place()`](core::ptr::drop_in_place) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_drop_in_place(&self, ptrs: Self::MutPtrs<'_>);

    /// Collection of non-null pointers to each stored field.
    type NonNullPtrs<'a>: Clone;

    /// Restricts [non-null pointers](RawSoaContext::NonNullPtrs) to each stored field
    /// to be covariant over generic lifetime.
    fn upcast_nonnull_ptrs<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short>;

    /// Creates [non-null pointers](RawSoaContext::NonNullPtrs) to each stored field.
    ///
    /// All the safety requirements resulting from applying
    /// [`NonNull::new_unchecked()`](core::ptr::NonNull::new_unchecked) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_to_nonnull<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::NonNullPtrs<'a>;

    /// Acquires the underlying [pointers](RawSoaContext::MutPtrs) from [non-null pointers](RawSoaContext::NonNullPtrs)
    /// to each stored field.
    fn nonnull_to_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a>;

    /// Collection of slice pointers to each stored field.
    type SlicePtrs<'a>: Clone;

    /// Restricts [slice pointers](RawSoaContext::SlicePtrs) to each stored field
    /// to be covariant over generic lifetime.
    fn upcast_slice_ptrs<'short, 'long: 'short>(
        from: Self::SlicePtrs<'long>,
    ) -> Self::SlicePtrs<'short>;

    /// Forms [slice pointers](RawSoaContext::SlicePtrs) to each stored field
    /// from [pointers](RawSoaContext::Ptrs) to each field and a length.
    ///
    /// The len argument is the number of elements, not the number of bytes.
    fn slice_ptrs_from_raw_parts<'a>(
        &'a self,
        ptrs: Self::Ptrs<'a>,
        len: usize,
    ) -> Self::SlicePtrs<'a>;

    /// Returns the number of elements in slices to each [slice pointer](RawSoaContext::SlicePtrs) of stored fields,
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the slice pointers,
    /// or else this method could panic.
    fn slice_ptrs_len(&self, slices: &Self::SlicePtrs<'_>) -> usize;

    /// Returns [pointers](RawSoaContext::Ptrs) to the slice's buffer
    /// of each [slice pointer](RawSoaContext::SlicePtrs) of stored fields.
    fn slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Ptrs<'a>;

    /// Collection of mutable slice pointers to each stored field.
    type SliceMutPtrs<'a>: Clone;

    /// Restricts [mutable slice pointers](RawSoaContext::SliceMutPtrs) to each stored field
    /// to be covariant over generic lifetime.
    fn upcast_mut_slice_ptrs<'short, 'long: 'short>(
        from: Self::SliceMutPtrs<'long>,
    ) -> Self::SliceMutPtrs<'short>;

    /// Forms [mutable slice pointers](RawSoaContext::SliceMutPtrs) to each stored field
    /// from [mutable pointers](RawSoaContext::MutPtrs) to each field and a length.
    ///
    /// The len argument is the number of elements, not the number of bytes.
    fn mut_slice_ptrs_from_raw_parts<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        len: usize,
    ) -> Self::SliceMutPtrs<'a>;

    /// Returns the number of elements in slices to each [mutable slice pointer](RawSoaContext::SliceMutPtrs) of stored fields,
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the mutable slice pointers,
    /// or else this method could panic.
    fn mut_slice_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize;

    /// Returns [mutable pointers](RawSoaContext::MutPtrs) to the slice's buffer
    /// of each [mutable slice pointer](RawSoaContext::SliceMutPtrs) of stored fields.
    fn mut_slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::MutPtrs<'a>;

    /// Converts [slice pointers](RawSoaContext::SlicePtrs) of each field of stored fields
    /// to the [mutable ones](RawSoaContext::SliceMutPtrs).
    fn slice_ptrs_cast_const<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::SlicePtrs<'a>;

    /// Converts [mutable slice pointers](RawSoaContext::SliceMutPtrs) of each field of stored fields
    /// to the [const ones](RawSoaContext::SlicePtrs).
    fn slice_ptrs_cast_mut<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::SliceMutPtrs<'a>;

    /// Executes the destructors (if any) for the each [slice](RawSoaContext::SliceMutPtrs) of stored fields.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::drop_in_place()`](core::ptr::drop_in_place) method to each slice pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// By default, this method just iterates by all the fields of slices and drops such fields one by one.
    unsafe fn slices_drop_in_place(&self, slices: Self::SliceMutPtrs<'_>) {
        let slices = Self::upcast_mut_slice_ptrs(slices);
        let len = self.mut_slice_ptrs_len(&slices);
        let ptrs = self.mut_slice_ptrs_as_ptrs(slices);
        for index in 0..len {
            let ptrs = unsafe { self.ptrs_add_mut(ptrs.clone(), index) };
            unsafe { self.ptrs_drop_in_place(ptrs) }
        }
    }
}

/// Alias for the [`Ptrs`](RawSoaContext::Ptrs) associated type
/// of the [`Context`](RawSoa::Context) associated type of a given [SoA](RawSoa) type.
pub type Ptrs<'a, T> = <<T as RawSoa>::Context as RawSoaContext<T>>::Ptrs<'a>;

/// Alias for the [`MutPtrs`](RawSoaContext::MutPtrs) associated type
/// of the [`Context`](RawSoa::Context) associated type of a given [SoA](RawSoa) type.
pub type MutPtrs<'a, T> = <<T as RawSoa>::Context as RawSoaContext<T>>::MutPtrs<'a>;

/// Alias for the [`NonNullPtrs`](RawSoaContext::NonNullPtrs) associated type
/// of the [`Context`](RawSoa::Context) associated type of a given [SoA](RawSoa) type.
pub type NonNullPtrs<'a, T> = <<T as RawSoa>::Context as RawSoaContext<T>>::NonNullPtrs<'a>;

/// Alias for the [`SlicePtrs`](RawSoaContext::SlicePtrs) associated type
/// of the [`Context`](RawSoa::Context) associated type of a given [SoA](RawSoa) type.
pub type SlicePtrs<'a, T> = <<T as RawSoa>::Context as RawSoaContext<T>>::SlicePtrs<'a>;

/// Alias for the [`SliceMutPtrs`](RawSoaContext::SliceMutPtrs) associated type
/// of the [`Context`](RawSoa::Context) associated type of a given [SoA](RawSoa) type.
pub type SliceMutPtrs<'a, T> = <<T as RawSoa>::Context as RawSoaContext<T>>::SliceMutPtrs<'a>;

/// The main trait of the [crate] which defines behavior of this type
/// in the context of Structure of Arrays pattern, or SoA.
pub unsafe trait RawSoa {
    /// Type of SoA [context](RawSoaContext).
    ///
    /// Most of the time, this should be zero-sized type.
    /// This is true for all the SoA types with stored fields' size and alignment known at compile-time.
    type Context: RawSoaContext<Self> + ?Sized;

    /// Special type containing all the fields which are stored inside of a buffer.
    ///
    /// This type is used to define implementations of [`Copy`], [`Send`], [`Sync`]
    /// and other traits for SoA containers.
    ///
    /// Most of the time, this should be just `Self`.
    /// This is true for such implementations which store all the fields of self.
    type Fields: ?Sized;
}

/// An extension of [SoA context](RawSoaContext) type which allows to perform copy-assignment of each stored field.
///
/// This trait is analogous to the unstable [`CloneToUninit`](core::clone::CloneToUninit) trait.
pub unsafe trait CloneToUninitSoaContext<T>: RawSoaContext<T>
where
    T: ?Sized,
{
    /// Performs copy-assignment of each stored field from [src](RawSoaContext::Ptrs) to [dst](RawSoaContext::MutPtrs).
    /// Before this function is called, src must point to initialized memory and dst may point to uninitialized memory.
    unsafe fn clone_to_uninit(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>);
}

/// A generalization of [`Clone`] specifically for [SoA](RawSoa) type.
///
/// This trait is analogous to the unstable [`CloneToUninit`](core::clone::CloneToUninit) trait.
pub unsafe trait SoaCloneToUninit: RawSoa<Context: CloneToUninitSoaContext<Self>> {}

unsafe impl<T> SoaCloneToUninit for T
where
    T: RawSoa + ?Sized,
    T::Context: CloneToUninitSoaContext<T>,
{
}

/// An extension of [SoA context](RawSoaContext) type which allows to read a value borrowed from self
/// from [pointers](RawSoaContext::Ptrs) to each stored field.
pub unsafe trait ReadSoaContext<'a, R, T>: RawSoaContext<T>
where
    T: ?Sized,
{
    /// Constructs the value from reading each field to which [src](RawSoaContext::Ptrs) points without moving them.
    /// This leaves the memory in src unchanged.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::read()`](core::ptr::read) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn read(&'a self, src: Self::Ptrs<'a>) -> R;
}

/// An extension of [SoA](RawSoa) type which allows to read a value borrowed from the context
/// from [pointers](RawSoaContext::Ptrs) to each stored field.
pub unsafe trait SoaRead<'a, R>: RawSoa<Context: ReadSoaContext<'a, R, Self>> {}

unsafe impl<'a, T, R> SoaRead<'a, R> for T
where
    T: RawSoa + ?Sized,
    T::Context: ReadSoaContext<'a, R, T>,
{
}

/// An extension of [SoA](Soa) type which allows to read a value of *any* lifetime
/// from [pointers](RawSoaContext::Ptrs) to each stored field.
pub unsafe trait SoaReadOwned<R>: for<'a> SoaRead<'a, R> {}

unsafe impl<T, R> SoaReadOwned<R> for T where T: for<'a> SoaRead<'a, R> + ?Sized {}

/// An extension of [SoA context](RawSoaContext) type which allows to write a value
/// into [mutale pointers](RawSoaContext::MutPtrs) to each stored field.
pub unsafe trait WriteSoaContext<W, T>: RawSoaContext<T>
where
    T: ?Sized,
{
    /// Overwrites a memory [location](RawSoaContext::MutPtrs) of each stored field
    /// with the given value without reading or dropping the old value.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::write()`](core::ptr::write) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn write(&self, dst: Self::MutPtrs<'_>, value: W);
}

/// An extension of [SoA](RawSoa) type which allows to write given value
/// into [mutable pointers](RawSoaContext::Ptrs) to each stored field.
pub unsafe trait SoaWrite<W>: RawSoa<Context: WriteSoaContext<W, Self>> {}

unsafe impl<T, W> SoaWrite<W> for T
where
    T: RawSoa + ?Sized,
    T::Context: WriteSoaContext<W, T>,
{
}

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
        self::buffer_layout(fields, capacity)
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

/// An extension of [SoA context](RawSoaContext) type which provides
/// reference and slice types of specific lifetime to each stored field.
pub unsafe trait SoaContext<'data, T>: RawSoaContext<T>
where
    T: ?Sized,
{
    /// Collection of references to each stored field.
    ///
    /// Order of such references **may not** resemble their order inside of a buffer in memory.
    type Refs<'a>;

    /// Restricts [references](SoaContext::Refs) to each stored field
    /// to be covariant over generic lifetime.
    fn upcast_refs<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short>;

    /// Converts [pointers](RawSoaContext::Ptrs) to each stored field
    /// to their [references](SoaContext::Refs) by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_to_refs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::Refs<'a>;

    /// Converts [references](SoaContext::Refs) to each stored field
    /// to their [pointers](RawSoaContext::Ptrs) by taking the pointer of each one of them.
    fn refs_as_ptrs<'a>(&'a self, refs: Self::Refs<'a>) -> Self::Ptrs<'a>;

    /// Collection of mutable references to each stored field.
    ///
    /// Order of such references **may not** resemble their order inside of a buffer in memory.
    type RefsMut<'a>;

    /// Restricts [mutable references](SoaContext::RefsMut) to each stored field
    /// to be covariant over generic lifetime.
    fn upcast_mut_refs<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short>;

    /// Converts [mutable pointers](RawSoaContext::MutPtrs) to each stored field
    /// to their [mutable references](SoaContext::RefsMut) by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn mut_ptrs_to_mut_refs<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::RefsMut<'a>;

    /// Converts [mutable references](SoaContext::RefsMut) to each stored field
    /// to their [mutable pointers](RawSoaContext::MutPtrs) by taking the pointer of each one of them.
    fn mut_refs_as_mut_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::MutPtrs<'a>;

    /// Converts [mutable references](SoaContext::RefsMut) to each stored field
    /// to their [references](SoaContext::Refs) by explicitly converting each one of them via `&*` operator combination.
    fn mut_refs_as_refs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Refs<'a>;

    /// Collection of slices of each stored field.
    ///
    /// Order of such slices may not resemble their order inside of a buffer in memory.
    type Slices<'a>;

    /// Restricts [slices](SoaContext::Slices) to each stored field
    /// to be covariant over generic lifetime.
    fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short>;

    /// Converts [slice pointers](RawSoaContext::SlicePtrs) to each stored field
    /// to their [slices](SoaContext::Slices) by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each slice pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn slice_ptrs_to_slices<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Slices<'a>;

    /// Converts [slices](SoaContext::Slices) to each stored field
    /// to their [slice pointers](RawSoaContext::SlicePtrs) by taking the pointer of each one of them.
    fn slices_as_slice_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::SlicePtrs<'a>;

    /// Returns the number of elements in [slices](SoaContext::Slices) to each stored field,
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the slices,
    /// or else this method could panic.
    fn slices_len(&self, slices: &Self::Slices<'_>) -> usize;

    /// Collection of mutable slices of each stored field.
    ///
    /// Order of such slices may not resemble their order inside of a buffer in memory.
    type SlicesMut<'a>;

    /// Restricts [mutable slices](SoaContext::SlicesMut) to each stored field
    /// to be covariant over generic lifetime.
    fn upcast_mut_slices<'short, 'long: 'short>(
        from: Self::SlicesMut<'long>,
    ) -> Self::SlicesMut<'short>;

    /// Converts [mutable slice pointers](RawSoaContext::SliceMutPtrs) to each stored field
    /// to their [mutable slices](SoaContext::SlicesMut) by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each mutable slice pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn mut_slice_ptrs_to_mut_slices<'a>(
        &'a self,
        slices: Self::SliceMutPtrs<'a>,
    ) -> Self::SlicesMut<'a>;

    /// Converts [mutable slices](SoaContext::SlicesMut) to each stored field
    /// to their [mutable slice pointers](RawSoaContext::SliceMutPtrs) by taking the pointer of each one of them.
    fn mut_slices_as_mut_slice_ptrs<'a>(
        &'a self,
        slices: Self::SlicesMut<'a>,
    ) -> Self::SliceMutPtrs<'a>;

    /// Returns the number of elements in [mutable slices](SoaContext::SlicesMut) to each stored field,
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the mutable slices,
    /// or else this method could panic.
    fn mut_slices_len(&self, slices: &Self::SlicesMut<'_>) -> usize;

    /// Converts [mutable slices](SoaContext::SlicesMut) to each stored field
    /// to their [slices](SoaContext::Slices) by explicitly converting each one of them via `&*` operator combination.
    fn mut_slices_as_slices<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Slices<'a>;
}

/// Alias for the [`Refs`](SoaContext::Refs) associated type
/// of the [`Context`](RawSoa::Context) associated type of a given [SoA](Soa) type.
pub type Refs<'a, 'data, T> = <<T as RawSoa>::Context as SoaContext<'data, T>>::Refs<'a>;

/// Alias for the [`RefsMut`](SoaContext::RefsMut) associated type
/// of the [`Context`](RawSoa::Context) associated type of a given [SoA](Soa) type.
pub type RefsMut<'a, 'data, T> = <<T as RawSoa>::Context as SoaContext<'data, T>>::RefsMut<'a>;

/// Alias for the [`Slices`](SoaContext::Slices) associated type
/// of the [`Context`](RawSoa::Context) associated type of a given [SoA](Soa) type.
pub type Slices<'a, 'data, T> = <<T as RawSoa>::Context as SoaContext<'data, T>>::Slices<'a>;

/// Alias for the [`SlicesMut`](SoaContext::SlicesMut) associated type
/// of the [`Context`](RawSoa::Context) associated type of a given [SoA](Soa) type.
pub type SlicesMut<'a, 'data, T> = <<T as RawSoa>::Context as SoaContext<'data, T>>::SlicesMut<'a>;

/// An extension of [SoA](RawSoa) type which allows to access
/// each stored field by their reference types of specific lifetime.
pub unsafe trait Soa<'a>: RawSoa<Context: SoaContext<'a, Self>> {}

unsafe impl<'a, T> Soa<'a> for T
where
    T: RawSoa + ?Sized,
    T::Context: SoaContext<'a, T>,
{
}

/// An extension of [SoA](RawSoa) type which allows to access
/// each stored field by their reference types of **any** lifetime.
pub trait SoaOwned: for<'a> Soa<'a> {}

impl<T> SoaOwned for T where T: for<'a> Soa<'a> + ?Sized {}
