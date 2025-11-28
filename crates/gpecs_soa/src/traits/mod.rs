#![allow(clippy::doc_markdown)] // suppress 'SoA' item in documentation is missing backticks

use core::alloc::{Layout, LayoutError};

use crate::field::{FieldDescriptor, buffer_layout};

pub use self::impls::TupleContext;

#[doc(hidden)]
pub mod impls;

/// This trait is used to perform all memory operations & pointer arithmetics for [SoA](RawSoa) types.
pub unsafe trait RawSoaContext {
    /// Non-empty collection of [descriptors](FieldDescriptor) for each stored field.
    ///
    /// Order of such descriptors **MUST** resemble their order inside of a buffer in memory.
    type FieldDescriptors<'a>: IntoIterator<Item: AsRef<FieldDescriptor>>;

    /// Restricts [field descriptors](RawSoaContext::FieldDescriptors)
    /// to be covariant over generic lifetime.
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: Self::FieldDescriptors<'long>,
    ) -> Self::FieldDescriptors<'short>;

    /// Returns [field descriptors](RawSoaContext::FieldDescriptors) for each stored field.
    fn field_descriptors(&self) -> Self::FieldDescriptors<'_>;

    /// Calculates layout needed to store `capacity` number of fields inside of a buffer.
    ///
    /// This layout should not include self, as it is handled by the crate itself.
    fn buffer_layout(&self, capacity: usize) -> Result<Layout, LayoutError> {
        let fields = self.field_descriptors();
        self::buffer_layout(fields, capacity)
    }

    /// Retrieves maximum number of sets of fields which can be stored inside of a buffer with given layout.
    fn capacity_from(&self, buffer_layout: Layout) -> usize {
        let packed_size = self
            .field_descriptors()
            .into_iter()
            .map(|desc| desc.as_ref().layout().size())
            .sum();
        let buffer_size = buffer_layout.size();
        let max_capacity = buffer_size.checked_div(packed_size).unwrap_or(0);

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

    /// Non-empty collection of pointers to each stored field.
    ///
    /// Unlike [field descriptors](RawSoaContext::FieldDescriptors),
    /// order of such pointers **may not** resemble their order inside of a buffer in memory.
    /// Reordering of such pointers in other methods is up to the implementation of this trait.
    type Ptrs<'a>: Clone;

    /// Restricts [pointers](RawSoaContext::Ptrs) to each stored field
    /// to be covariant over generic lifetime.
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short>;

    /// Returns dangling [pointers](RawSoaContext::Ptrs) to each stored field.
    fn ptrs_dangling(&self) -> Self::Ptrs<'_>;

    /// Creates [pointers](RawSoaContext::Ptrs) to each stored field
    /// from a given buffer with given capacity.
    ///
    /// Implementations of this method should not account for `Self`,
    /// as it is handled by the crate itself.
    ///
    /// # Safety
    ///
    /// Layout from a given pointer to a buffer to the end of the allocation of such buffer
    /// must be the same as the one returned by [`buffer_layout()`](RawSoaContext::buffer_layout) method.
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, capacity: usize) -> Self::Ptrs<'_>;

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

    /// Non-empty collection of mutable pointers to each stored field.
    ///
    /// Unlike [field descriptors](RawSoaContext::FieldDescriptors),
    /// order of such pointers **may not** resemble their order inside of a buffer in memory.
    /// Reordering of such pointers in other methods is up to the implementation of this trait.
    type MutPtrs<'a>: Clone;

    /// Restricts [mutable pointers](RawSoaContext::MutPtrs) to each stored field
    /// to be covariant over generic lifetime.
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short>;

    /// Returns mutable dangling [pointers](RawSoaContext::MutPtrs) to each stored field.
    fn ptrs_dangling_mut(&self) -> Self::MutPtrs<'_>;

    /// Creates [mutable pointers](RawSoaContext::MutPtrs) to each stored field
    /// from a given buffer with given capacity.
    ///
    /// Implementations of this method should not account for `Self`,
    /// as it is handled by the crate itself.
    ///
    /// # Safety
    ///
    /// Layout from a given pointer to a buffer to the end of the allocation of such buffer
    /// must be the same as the one returned by [`buffer_layout()`](RawSoaContext::buffer_layout) method.
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, capacity: usize) -> Self::MutPtrs<'_>;

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
    /// sequentially in the same order as they are stored in the buffer,
    /// without deinitializing either.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::swap()`](core::ptr::swap) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_swap(&self, a: Self::MutPtrs<'_>, b: Self::MutPtrs<'_>);

    /// Copies `count * size_of::<fields[0]>() + ...` bytes from [src](RawSoaContext::Ptrs) to [dst](RawSoaContext::MutPtrs)
    /// for each stored field sequentially in the same order as they are stored in the buffer.
    /// The source and destination may overlap.
    ///
    /// If the source and destination will never overlap,
    /// [`ptrs_copy_nonoverlapping()`](RawSoaContext::ptrs_copy_nonoverlapping) can be used instead.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::copy()`](core::ptr::copy) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_copy(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize);

    /// Copies `count * size_of::<fields[0]>() + ...` bytes from [src](RawSoaContext::Ptrs) to [dst](RawSoaContext::MutPtrs)
    /// for each stored field sequentially in the *reverse* order as they are stored in the buffer.
    /// The source and destination may overlap.
    ///
    /// If the source and destination will never overlap,
    /// [`ptrs_copy_nonoverlapping()`](RawSoaContext::ptrs_copy_nonoverlapping) can be used instead.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::copy()`](core::ptr::copy) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_copy_rev(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize);

    /// Copies `count * size_of::<fields[0]>() + ...` bytes from [src](RawSoaContext::Ptrs) to [dst](RawSoaContext::MutPtrs)
    /// for each stored field sequentially in unspecified order.
    /// The source and destination must not overlap.
    ///
    /// For regions of memory which might overlap, use
    /// [`ptrs_copy()`](RawSoaContext::ptrs_copy) or [`ptrs_copy_rev()`](RawSoaContext::ptrs_copy_rev) instead.
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

    /// Executes the destructors (if any) for the each stored field located at ptrs.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::drop_in_place()`](core::ptr::drop_in_place) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_drop_in_place(&self, ptrs: Self::MutPtrs<'_>);

    /// Non-empty collection of non-null pointers to each stored field.
    ///
    /// Order of such pointers **may not** resemble their order inside of a buffer in memory.
    /// Reordering of such pointers in other methods is up to the implementation of this trait.
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

    /// Non-empty collection of slice pointers to each stored field.
    ///
    /// Unlike [field descriptors](RawSoaContext::FieldDescriptors),
    /// order of such pointers **may not** resemble their order inside of a buffer in memory.
    /// Reordering of such pointers in other methods is up to the implementation of this trait.
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

    /// Non-empty collection of mutable slice pointers to each stored field.
    ///
    /// Unlike [field descriptors](RawSoaContext::FieldDescriptors),
    /// order of such pointers **may not** resemble their order inside of a buffer in memory.
    /// Reordering of such pointers in other methods is up to the implementation of this trait.
    type SliceMutPtrs<'a>: Clone;

    /// Restricts [mutable slice pointers](RawSoaContext::SliceMutPtrs) to each stored field
    /// to be covariant over generic lifetime.
    fn upcast_slice_mut_ptrs<'short, 'long: 'short>(
        from: Self::SliceMutPtrs<'long>,
    ) -> Self::SliceMutPtrs<'short>;

    /// Forms [mutable slice pointers](RawSoaContext::SliceMutPtrs) to each stored field
    /// from [mutable pointers](RawSoaContext::MutPtrs) to each field and a length.
    ///
    /// The len argument is the number of elements, not the number of bytes.
    fn slice_mut_ptrs_from_raw_parts<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        len: usize,
    ) -> Self::SliceMutPtrs<'a>;

    /// Returns the number of elements in slices to each [mutable slice pointer](RawSoaContext::SliceMutPtrs) of stored fields,
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the mutable slice pointers,
    /// or else this method could panic.
    fn slice_mut_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize;

    /// Returns [mutable pointers](RawSoaContext::MutPtrs) to the slice's buffer
    /// of each [mutable slice pointer](RawSoaContext::SliceMutPtrs) of stored fields.
    fn slice_mut_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::MutPtrs<'a>;

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
        let slices = Self::upcast_slice_mut_ptrs(slices);
        let len = self.slice_mut_ptrs_len(&slices);
        let ptrs = self.slice_mut_ptrs_as_ptrs(slices);
        for index in 0..len {
            let ptrs = unsafe { self.ptrs_add_mut(ptrs.clone(), index) };
            unsafe { self.ptrs_drop_in_place(ptrs) }
        }
    }
}

/// Alias for the [`FieldDescriptors`](RawSoaContext::FieldDescriptors) associated type
/// of the [`Context`](RawSoa::Context) associated type of a given [`SoA`](RawSoa) type.
pub type FieldDescriptors<'a, T> = <<T as RawSoa>::Context as RawSoaContext>::FieldDescriptors<'a>;

/// Alias for the [`Ptrs`](RawSoaContext::Ptrs) associated type
/// of the [`Context`](RawSoa::Context) associated type of a given [`SoA`](RawSoa) type.
pub type Ptrs<'a, T> = <<T as RawSoa>::Context as RawSoaContext>::Ptrs<'a>;

/// Alias for the [`MutPtrs`](RawSoaContext::MutPtrs) associated type
/// of the [`Context`](RawSoa::Context) associated type of a given [`SoA`](RawSoa) type.
pub type MutPtrs<'a, T> = <<T as RawSoa>::Context as RawSoaContext>::MutPtrs<'a>;

/// Alias for the [`NonNullPtrs`](RawSoaContext::NonNullPtrs) associated type
/// of the [`Context`](RawSoa::Context) associated type of a given [`SoA`](RawSoa) type.
pub type NonNullPtrs<'a, T> = <<T as RawSoa>::Context as RawSoaContext>::NonNullPtrs<'a>;

/// Alias for the [`SlicePtrs`](RawSoaContext::SlicePtrs) associated type
/// of the [`Context`](RawSoa::Context) associated type of a given [`SoA`](RawSoa) type.
pub type SlicePtrs<'a, T> = <<T as RawSoa>::Context as RawSoaContext>::SlicePtrs<'a>;

/// Alias for the [`SliceMutPtrs`](RawSoaContext::SliceMutPtrs) associated type
/// of the [`Context`](RawSoa::Context) associated type of a given [`SoA`](RawSoa) type.
pub type SliceMutPtrs<'a, T> = <<T as RawSoa>::Context as RawSoaContext>::SliceMutPtrs<'a>;

/// The main trait of the [crate] which defines behavior of this type
/// in the context of Structure of Arrays pattern, or SoA.
pub unsafe trait RawSoa {
    /// Type of SoA [context](RawSoaContext).
    ///
    /// Most of the time, this should be zero-sized type.
    /// This is true for all the SoA types with stored fields' size and alignment known at compile-time.
    type Context: RawSoaContext;

    /// Special type containing all the fields which are stored inside of a buffer.
    ///
    /// This type is used to define implementations of [`Copy`], [`Send`], [`Sync`]
    /// and other traits for SoA containers.
    ///
    /// Most of the time, this should be just `Self`.
    /// This is true for such implementations which store all the fields of self.
    type Fields;
}

pub unsafe trait SoaRead: RawSoa + Sized {
    /// Constructs the value from reading each field to which [src](RawSoaContext::Ptrs) points without moving them.
    /// This leaves the memory in src unchanged.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::read()`](core::ptr::read) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn read(context: &Self::Context, src: Ptrs<'_, Self>) -> Self;
}

pub unsafe trait SoaWrite: RawSoa + Sized {
    /// Overwrites a memory [location](RawSoaContext::MutPtrs) of each stored field
    /// with the given value without reading or dropping the old value.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::write()`](core::ptr::write) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn write(context: &Self::Context, dst: MutPtrs<'_, Self>, value: Self);
}

pub unsafe trait Soa: RawSoa {
    /// Non-empty collection of references to each stored field.
    ///
    /// Order of such references **may not** resemble their order inside of a buffer in memory.
    type Refs<'context, 'a>
    where
        Self: 'a;

    /// Restricts [references](Soa::Refs) to each stored field
    /// to be covariant over generic lifetimes.
    fn upcast_refs<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::Refs<'long, 'a_long>,
    ) -> Self::Refs<'short, 'a_short>
    where
        Self: 'a_long;

    /// Non-empty collection of mutable references to each stored field.
    ///
    /// Order of such references **may not** resemble their order inside of a buffer in memory.
    type RefsMut<'context, 'a>
    where
        Self: 'a;

    /// Restricts [mutable references](Soa::RefsMut) to each stored field
    /// to be covariant over generic lifetimes.
    fn upcast_refs_mut<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::RefsMut<'long, 'a_long>,
    ) -> Self::RefsMut<'short, 'a_short>
    where
        Self: 'a_long;

    /// Converts [pointers](RawSoaContext::Ptrs) to each stored field
    /// to their [references](Soa::Refs) by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_to_refs<'context, 'a>(
        context: &'context Self::Context,
        ptrs: Ptrs<'context, Self>,
    ) -> Self::Refs<'context, 'a>
    where
        Self: 'a;

    /// Converts [mutable pointers](RawSoaContext::MutPtrs) to each stored field
    /// to their [mutable references](Soa::RefsMut) by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_to_refs_mut<'context, 'a>(
        context: &'context Self::Context,
        ptrs: MutPtrs<'context, Self>,
    ) -> Self::RefsMut<'context, 'a>
    where
        Self: 'a;

    /// Converts [references](Soa::Refs) to each stored field
    /// to their [pointers](RawSoaContext::Ptrs) by taking the pointer of each one of them.
    fn refs_as_ptrs<'context, 'a>(
        context: &'context Self::Context,
        refs: Self::Refs<'context, 'a>,
    ) -> Ptrs<'context, Self>
    where
        Self: 'a;

    /// Converts [mutable references](Soa::RefsMut) to each stored field
    /// to their [mutable pointers](RawSoaContext::MutPtrs) by taking the pointer of each one of them.
    fn refs_mut_as_ptrs<'context, 'a>(
        context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> MutPtrs<'context, Self>
    where
        Self: 'a;

    /// Converts [mutable references](Soa::RefsMut) to each stored field
    /// to their [references](Soa::Refs) by explicitly converting each one of them via `&*` operator combination.
    fn refs_mut_as_refs<'context, 'a>(
        context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> Self::Refs<'context, 'a>
    where
        Self: 'a;

    /// Retrieves [references](Soa::Refs) to each stored field
    /// from a given value reference by taking the reference of each one of them.
    fn value_as_refs<'a>(context: &'a Self::Context, value: &'a Self) -> Self::Refs<'a, 'a>
    where
        Self: 'a;

    /// Retrieves [mutable references](Soa::RefsMut) to each stored field
    /// from a given mutable value reference by taking the mutable reference of each one of them.
    fn mut_value_as_refs<'a>(
        context: &'a Self::Context,
        value: &'a mut Self,
    ) -> Self::RefsMut<'a, 'a>
    where
        Self: 'a;

    /// Non-empty collection of slices of each stored field.
    ///
    /// Order of such slices may not resemble their order inside of a buffer in memory.
    type Slices<'context, 'a>
    where
        Self: 'a;

    /// Restricts [slices](Soa::Slices) to each stored field
    /// to be covariant over generic lifetimes.
    fn upcast_slices<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::Slices<'long, 'a_long>,
    ) -> Self::Slices<'short, 'a_short>
    where
        Self: 'a_long;

    /// Non-empty collection of mutable slices of each stored field.
    ///
    /// Order of such slices may not resemble their order inside of a buffer in memory.
    type SlicesMut<'context, 'a>
    where
        Self: 'a;

    /// Restricts [mutable slices](Soa::SlicesMut) to each stored field
    /// to be covariant over generic lifetimes.
    fn upcast_slices_mut<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::SlicesMut<'long, 'a_long>,
    ) -> Self::SlicesMut<'short, 'a_short>
    where
        Self: 'a_long;

    /// Converts [slice pointers](RawSoaContext::SlicePtrs) to each stored field
    /// to their [slices](Soa::Slices) by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each slice pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn slice_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: SlicePtrs<'context, Self>,
    ) -> Self::Slices<'context, 'a>
    where
        Self: 'a;

    /// Converts [mutable slice pointers](RawSoaContext::SliceMutPtrs) to each stored field
    /// to their [mutable slices](Soa::SlicesMut) by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each mutable slice pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn slice_mut_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: SliceMutPtrs<'context, Self>,
    ) -> Self::SlicesMut<'context, 'a>
    where
        Self: 'a;

    /// Returns the number of elements in [slices](Soa::Slices) to each stored field,
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the slices,
    /// or else this method could panic.
    fn slices_len<'a>(context: &Self::Context, slices: &Self::Slices<'_, 'a>) -> usize
    where
        Self: 'a;

    /// Returns the number of elements in [mutable slices](Soa::SlicesMut) to each stored field,
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the mutable slices,
    /// or else this method could panic.
    fn slices_mut_len<'a>(context: &Self::Context, slices: &Self::SlicesMut<'_, 'a>) -> usize
    where
        Self: 'a;

    /// Converts [slices](Soa::Slices) to each stored field
    /// to their [slice pointers](RawSoaContext::SlicePtrs) by taking the pointer of each one of them.
    fn slices_as_slice_ptrs<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::Slices<'context, 'a>,
    ) -> SlicePtrs<'context, Self>
    where
        Self: 'a;

    /// Converts [mutable slices](Soa::SlicesMut) to each stored field
    /// to their [mutable slice pointers](RawSoaContext::SliceMutPtrs) by taking the pointer of each one of them.
    fn slices_mut_as_slice_ptrs<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> SliceMutPtrs<'context, Self>
    where
        Self: 'a;

    /// Converts [mutable slices](Soa::SlicesMut) to each stored field
    /// to their [slices](Soa::Slices) by explicitly converting each one of them via `&*` operator combination.
    fn slices_mut_as_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::Slices<'context, 'a>
    where
        Self: 'a;

    /// Returns [pointers](RawSoaContext::Ptrs) to the slice's buffer
    /// of each [slice](Soa::Slices) of each stored field.
    fn slices_as_ptrs<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::Slices<'context, 'a>,
    ) -> Ptrs<'context, Self>
    where
        Self: 'a;

    /// Returns [mutable pointers](RawSoaContext::MutPtrs) to the slice's buffer
    /// of each [mutable slice](Soa::SlicesMut) of each stored field.
    fn slices_mut_as_ptrs<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> MutPtrs<'context, Self>
    where
        Self: 'a;
}

/// A generalization of [`Clone`] to borrowed data
/// specifically for the [`Soa`] trait.
///
/// This trait is analogous to [`ToOwned`] trait from the standard library.
///
/// [`ToOwned`]: https://doc.rust-lang.org/stable/alloc/borrow/trait.ToOwned.html
pub trait SoaToOwned<'context, 'a> {
    /// The resulting type after obtaining ownership.
    type Owned: Soa<Refs<'context, 'a> = Self> + 'a;

    /// Creates owned data from borrowed data,
    /// usually by cloning each stored field.
    fn to_owned(&self, context: &<Self::Owned as RawSoa>::Context) -> Self::Owned;

    /// Uses borrowed data to replace owned data,
    /// usually by cloning each stored field.
    ///
    /// This is borrow-generalized version of [`Clone::clone_from()`].
    fn clone_into(&self, context: &<Self::Owned as RawSoa>::Context, target: &mut Self::Owned) {
        *target = self.to_owned(context);
    }

    /// Uses borrowed data to replace owned data located by `target` [references](Soa::RefsMut),
    /// usually by cloning each stored field.
    fn clone_into_refs<'c>(
        &self,
        context: &'c <Self::Owned as RawSoa>::Context,
        target: <Self::Owned as Soa>::RefsMut<'c, '_>,
    );
}

/// Marker trait which places additional safety requirements
/// on the [`Fields`](RawSoa::Fields) associated type of [`Soa`] trait implementations.
///
/// These safety requirements are:
/// - sum of layouts' sizes of [`FieldDescriptors`](RawSoaContext::FieldDescriptors)
///   should be less or equal to the size of [`Fields`](RawSoa::Fields)
/// - alignment of each layout of [`FieldDescriptors`](RawSoaContext::FieldDescriptors)
///   should be less or equal to the alignment of [`Fields`](RawSoa::Fields)
pub unsafe trait SoaTrustedFields: RawSoa {}
