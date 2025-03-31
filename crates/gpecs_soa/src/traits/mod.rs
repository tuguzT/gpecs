use core::{
    alloc::{Layout, LayoutError},
    borrow::Borrow,
};

pub use desc::FieldDescriptor;

#[doc(hidden)]
pub mod impls;

mod desc;

/// The main trait of the [crate] which defines behavior of this type
/// in the context of Structure of Arrays (or SoA) pattern.
#[allow(clippy::missing_safety_doc)]
pub unsafe trait Soa: Sized {
    /// Type of context used to perform all operations of this trait.
    ///
    /// Most of the time, this should be [unit](primitive@unit) type.
    /// This is true for all the types with fields' size and alignment known at compile-time.
    type Context;

    /// Special type containing all the fields which are stored inside of a buffer.
    ///
    /// This type is used for checking bounds of stored fields.
    /// [`Copy`], [`Send`], [`Sync`] and some other bounds are defined with account to this type, not `Self`.
    ///
    /// Most of the time, this should be the same as `Self`.
    /// This is true for such implementations which store all the fields of self.
    type Fields;

    /// Collection of [descriptors](FieldDescriptor) for each field.
    ///
    /// Order of such descriptors **MUST** resemble their order inside of a buffer in memory.
    type FieldDescriptors<'a>: IntoIterator<Item: AsRef<FieldDescriptor>>;

    /// Returns [field descriptors](Soa::FieldDescriptors) for each field of [`Fields`](Soa::Fields).
    fn field_descriptors(context: &Self::Context) -> Self::FieldDescriptors<'_>;

    /// Collection of offsets (in bytes) for each field of [`Fields`](Soa::Fields).
    ///
    /// Each of these offsets **MUST** correspond to the offset of the field in the buffer
    /// in the order defined by [`FieldDescriptors`](Soa::FieldDescriptors).
    ///
    /// These offsets should not include offset to the [`Context`](Soa::Context),
    /// as it is handled by the crate itself.
    type FieldOffsets<'a>: IntoIterator<Item = usize>;

    /// Calculates layout needed to store `capacity` number of fields inside of a buffer.
    /// Also returns offsets of each field in the buffer (in bytes).
    ///
    /// This layout should not include [`Context`](Soa::Context),
    /// as it is handled by the crate itself.
    fn buffer_layout(
        context: &Self::Context,
        capacity: usize,
    ) -> Result<(Layout, Self::FieldOffsets<'_>), LayoutError>;

    /// Retrieves maximum number of fields that can be stored inside of a buffer with given layout.
    fn capacity_from(context: &Self::Context, buffer_layout: Layout) -> usize {
        let packed_size = Self::field_descriptors(context)
            .into_iter()
            .map(|desc| desc.as_ref().layout().size())
            .sum();
        let buffer_size = buffer_layout.size();
        let max_capacity = buffer_size.checked_div(packed_size).unwrap_or(0);

        let mut capacity = max_capacity;
        while {
            let (layout, _) = Self::buffer_layout(context, capacity)
                .expect("new buffer layout should be smaller than the input one");
            layout.size() > buffer_size
        } {
            capacity -= 1;
        }
        capacity
    }

    /// Collection of properly typed pointers to each field of [`Fields`](Soa::Fields).
    ///
    /// Unlike [`ErasedPtrs`](Soa::ErasedPtrs),
    /// order of such pointers **may not** resemble their order inside of a buffer in memory.
    /// Reordering of such pointers in other methods is up to the implementation of this trait.
    type Ptrs: Clone;

    /// Collection of properly typed mutable pointers to each field of [`Fields`](Soa::Fields).
    ///
    /// Unlike [`ErasedMutPtrs`](Soa::ErasedMutPtrs),
    /// order of such pointers **may not** resemble their order inside of a buffer in memory.
    /// Reordering of such pointers in other methods is up to the implementation of this trait.
    type MutPtrs: Clone;

    /// Collection of pointers to the bytes of each field of [`Fields`](Soa::Fields).
    ///
    /// Unlike [`Ptrs`](Soa::Ptrs),
    /// order of such pointers **MUST** resemble their order inside of a buffer in memory.
    type ErasedPtrs: IntoIterator<Item = *const u8>;

    /// Collection of mutable pointers to the bytes of each field of [`Fields`](Soa::Fields).
    ///
    /// Unlike [`MutPtrs`](Soa::MutPtrs),
    /// order of such pointers **MUST** resemble their order inside of a buffer in memory.
    type ErasedMutPtrs: IntoIterator<Item = *mut u8>;

    /// Erases pointers of each field of [`Fields`](Soa::Fields).
    ///
    /// Process of 'erasing' just means converting properly typed pointers to the pointers of bytes.
    fn ptrs_erase(context: &Self::Context, ptrs: Self::Ptrs) -> Self::ErasedPtrs;

    /// Erases mutable pointers of each field of [`Fields`](Soa::Fields).
    ///
    /// Process of 'erasing' just means converting properly typed pointers to the pointers of bytes.
    fn ptrs_erase_mut(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::ErasedMutPtrs;

    /// Restores pointers to bytes of each field of [`Fields`](Soa::Fields).
    ///
    /// Process of 'restoring' just means converting pointers of bytes to properly typed pointers.
    fn ptrs_restore(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *const u8>,
    ) -> Self::Ptrs;

    /// Restores mutable pointers to bytes of each field of [`Fields`](Soa::Fields).
    ///
    /// Process of 'restoring' just means converting pointers of bytes to properly typed pointers.
    fn ptrs_restore_mut(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *mut u8>,
    ) -> Self::MutPtrs;

    /// Returns dangling pointers to each field of [`Fields`](Soa::Fields).
    fn ptrs_dangling(context: &Self::Context) -> Self::MutPtrs;

    /// Converts pointers to the mutable ones of each field of [`Fields`](Soa::Fields).
    fn ptrs_cast_const(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::Ptrs;

    /// Converts mutable pointers to the const ones of each field of [`Fields`](Soa::Fields).
    fn ptrs_cast_mut(context: &Self::Context, ptrs: Self::Ptrs) -> Self::MutPtrs;

    /// Adds an unsigned offset to each properly typed pointer.
    ///
    /// All the safety requirements resulting from applying [`pointer::add()`] method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// [`pointer::add()`]: https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.add
    unsafe fn ptrs_add(context: &Self::Context, ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs;

    /// Adds an unsigned offset to each mutable properly typed pointer.
    ///
    /// All the safety requirements resulting from applying [`pointer::add()`] method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// [`pointer::add()`]: https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.add-1
    unsafe fn ptrs_add_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        offset: usize,
    ) -> Self::MutPtrs;

    /// Calculates the distance between two properly typed pointers
    /// to each field of [`Fields`](Soa::Fields) within the same allocation.
    ///
    /// All the safety requirements resulting from applying [`pointer::offset_from()`] method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// Note that resulting offsets should be the same for all the fields,
    /// or else this method could panic.
    ///
    /// [`pointer::offset_from()`]: https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.offset_from
    unsafe fn ptrs_offset_from(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        origin: Self::Ptrs,
    ) -> isize;

    /// Calculates the distance between two properly typed mutable pointers
    /// to each field of [`Fields`](Soa::Fields) within the same allocation.
    ///
    /// All the safety requirements resulting from applying [`pointer::offset_from()`] method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// Note that resulting offsets should be the same for all the fields,
    /// or else this method could panic.
    ///
    /// [`pointer::offset_from()`]: https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.offset_from-1
    unsafe fn ptrs_offset_from_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        origin: Self::Ptrs,
    ) -> isize;

    /// Swaps the values at two mutable locations of the [`Fields`](Soa::Fields)
    /// sequentially in the same order as they are stored in the buffer,
    /// without deinitializing either.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::swap()`](core::ptr::swap) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_swap(context: &Self::Context, a: Self::MutPtrs, b: Self::MutPtrs);

    /// Copies `count * size_of::<Fields[0]>() + ...` bytes from src to dst
    /// for each field of [`Fields`](Soa::Fields)
    /// sequentially in the same order as they are stored in the buffer.
    /// The source and destination may overlap.
    ///
    /// If the source and destination will never overlap,
    /// [`ptrs_copy_nonoverlapping()`](Soa::ptrs_copy_nonoverlapping) can be used instead.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::copy()`](core::ptr::copy) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_copy(context: &Self::Context, src: Self::Ptrs, dst: Self::MutPtrs, len: usize);

    /// Copies `count * size_of::<Fields[0]>() + ...` bytes from src to dst
    /// for each field of [`Fields`](Soa::Fields)
    /// sequentially in the *reverse* order as they are stored in the buffer.
    /// The source and destination may overlap.
    ///
    /// If the source and destination will never overlap,
    /// [`ptrs_copy_nonoverlapping()`](Soa::ptrs_copy_nonoverlapping) can be used instead.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::copy()`](core::ptr::copy) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_copy_rev(
        context: &Self::Context,
        src: Self::Ptrs,
        dst: Self::MutPtrs,
        len: usize,
    );

    /// Copies `count * size_of::<Fields[0]>() + ...` bytes from src to dst
    /// for each field of [`Fields`](Soa::Fields) sequentially in unspecified order.
    /// The source and destination must not overlap.
    ///
    /// For regions of memory which might overlap, use
    /// [`ptrs_copy()`](Soa::ptrs_copy) or [`ptrs_copy_rev()`](Soa::ptrs_copy_rev) instead.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::copy_nonoverlapping()`](core::ptr::copy_nonoverlapping) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_copy_nonoverlapping(
        context: &Self::Context,
        src: Self::Ptrs,
        dst: Self::MutPtrs,
        len: usize,
    );

    /// Constructs the value from reading each field to which src points without moving them.
    /// This leaves the memory in src unchanged.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::read()`](core::ptr::read) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_read(context: &Self::Context, src: Self::Ptrs) -> Self;

    /// Overwrites a memory location of each field of [`Fields`](Soa::Fields)
    /// with the given value without reading or dropping the old value.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::write()`](core::ptr::write) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_write(context: &Self::Context, dst: Self::MutPtrs, value: Self);

    /// Executes the destructors (if any) for the each field of [`Fields`](Soa::Fields) located at ptrs.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::drop_in_place()`](core::ptr::drop_in_place) method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// By default, this method just reads the value from ptrs on the stack and then drops it.
    unsafe fn ptrs_drop_in_place(context: &Self::Context, ptrs: Self::MutPtrs) {
        let ptrs = Self::ptrs_cast_const(context, ptrs);
        let value = unsafe { Self::ptrs_read(context, ptrs) };
        drop(value);
    }

    /// Collection of properly typed non-null pointers to each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such pointers **may not** resemble their order inside of a buffer in memory.
    /// Reordering of such pointers in other methods is up to the implementation of this trait.
    type NonNullPtrs: Clone;

    /// Creates properly typed non-null pointers to each field of [`Fields`](Soa::Fields).
    ///
    /// All the safety requirements resulting from applying
    /// [`NonNull::new_unchecked()`](core::ptr::NonNull::new_unchecked) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_to_nonnull(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::NonNullPtrs;

    /// Acquires the underlying pointers from non-null pointers to each field of [`Fields`](Soa::Fields).
    fn nonnull_to_ptrs(context: &Self::Context, ptrs: Self::NonNullPtrs) -> Self::MutPtrs;

    /// Collection of properly typed [vectors](alloc::vec::Vec) of each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such vectors **may not** resemble their order inside of a buffer in memory.
    type Vecs;

    /// Constructs new, empty vectors for each field of [`Fields`](Soa::Fields)
    /// with at least the specified capacity.
    fn vecs_with_capacity(context: &Self::Context, capacity: usize) -> Self::Vecs;

    /// Returns properly typed pointers to the vectors' buffers of each field of [`Fields`](Soa::Fields),
    /// or a dangling pointers valid for zero sized reads if these vectors didn’t allocate.
    fn vecs_as_ptrs(context: &Self::Context, vecs: &Self::Vecs) -> Self::Ptrs;

    /// Returns properly typed mutable pointers to the vectors' buffers of each field of [`Fields`](Soa::Fields),
    /// or a dangling pointers valid for zero sized reads if these vectors didn’t allocate.
    fn mut_vecs_as_ptrs(context: &Self::Context, vecs: &mut Self::Vecs) -> Self::MutPtrs;

    /// Returns the number of elements in each vector of [`Fields`](Soa::Fields),
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the vectors,
    /// or else this method could panic.
    fn vecs_len(context: &Self::Context, vecs: &Self::Vecs) -> usize;

    /// Forces the length of the vectors of [`Fields`](Soa::Fields) to `new_len`.
    ///
    /// All the safety requirements resulting from applying
    /// [`Vec::set_len()`](alloc::vec::Vec::set_len) method to each vector
    /// should be satisfied to be safe to call this method.
    unsafe fn vecs_set_len(context: &Self::Context, vecs: &mut Self::Vecs, len: usize);

    /// Collection of properly typed references to each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such references **may not** resemble their order inside of a buffer in memory.
    type Refs<'a>
    where
        Self: 'a;

    /// Collection of properly typed mutable references to each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such references **may not** resemble their order inside of a buffer in memory.
    type RefsMut<'a>
    where
        Self: 'a;

    /// Converts properly typed pointers to each field of [`Fields`](Soa::Fields)
    /// to their references by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_to_refs<'a>(context: &Self::Context, ptrs: Self::Ptrs) -> Self::Refs<'a>;

    /// Converts properly typed mutable pointers to each field of [`Fields`](Soa::Fields)
    /// to their mutable references by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_to_refs_mut<'a>(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
    ) -> Self::RefsMut<'a>;

    /// Converts properly typed references to each field of [`Fields`](Soa::Fields)
    /// to their pointers by taking the pointer of each one of them.
    fn refs_as_ptrs(context: &Self::Context, refs: Self::Refs<'_>) -> Self::Ptrs;

    /// Converts properly typed mutable references to each field of [`Fields`](Soa::Fields)
    /// to their mutable pointers by taking the pointer of each one of them.
    fn mut_refs_as_ptrs(context: &Self::Context, refs: Self::RefsMut<'_>) -> Self::MutPtrs;

    /// Converts properly typed mutable references to each field of [`Fields`](Soa::Fields)
    /// to their references by explicitly converting each one of them via `&*` operator combination.
    fn mut_refs_as_refs<'a>(context: &Self::Context, refs: Self::RefsMut<'a>) -> Self::Refs<'a>;

    /// Collection of slice pointers to each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such pointers **may not** resemble their order inside of a buffer in memory.
    type SlicePtrs: Clone;

    /// Collection of mutable slice pointers to each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such pointers **may not** resemble their order inside of a buffer in memory.
    type SliceMutPtrs: Clone;

    /// Forms slice pointers to each field of [`Fields`](Soa::Fields)
    /// from pointers to each field and a length.
    ///
    /// The len argument is the number of elements, not the number of bytes.
    fn slices_from_raw_parts(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        len: usize,
    ) -> Self::SlicePtrs;

    /// Forms mutable slice pointers to each field of [`Fields`](Soa::Fields)
    /// from mutable pointers to each field and a length.
    ///
    /// The len argument is the number of elements, not the number of bytes.
    fn slices_from_raw_parts_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        len: usize,
    ) -> Self::SliceMutPtrs;

    /// Converts slice pointers to the mutable ones of each field of [`Fields`](Soa::Fields).
    fn slice_ptrs_cast_const(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicePtrs;

    /// Converts mutable slice pointers to the const ones of each field of [`Fields`](Soa::Fields).
    fn slice_ptrs_cast_mut(context: &Self::Context, slices: Self::SlicePtrs) -> Self::SliceMutPtrs;

    /// Returns the number of elements in slices to each slice pointer of [`Fields`](Soa::Fields),
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the slice pointers,
    /// or else this method could panic.
    fn slice_ptrs_len(context: &Self::Context, slices: &Self::SlicePtrs) -> usize;

    /// Returns the number of elements in slices to each mutable slice pointer of [`Fields`](Soa::Fields),
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the mutable slice pointers,
    /// or else this method could panic.
    fn slice_ptrs_len_mut(context: &Self::Context, slices: &Self::SliceMutPtrs) -> usize;

    /// Returns properly typed pointers to the slice's buffer
    /// of each slice pointer of [`Fields`](Soa::Fields).
    fn slice_ptrs_as_ptrs(context: &Self::Context, slices: Self::SlicePtrs) -> Self::Ptrs;

    /// Returns properly typed mutable pointers to the slice's buffer
    /// of each mutable slice pointer of [`Fields`](Soa::Fields).
    fn mut_slice_ptrs_as_ptrs(context: &Self::Context, slices: Self::SliceMutPtrs)
        -> Self::MutPtrs;

    /// Collection of properly typed slices of each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such slices may not resemble their order inside of a buffer in memory.
    type Slices<'a>
    where
        Self: 'a;

    /// Collection of properly typed mutable slices of each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such slices may not resemble their order inside of a buffer in memory.
    type SlicesMut<'a>
    where
        Self: 'a;

    /// Converts properly typed slice pointers to each field of [`Fields`](Soa::Fields)
    /// to their slices by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each slice pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn slice_ptrs_to_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicePtrs,
    ) -> Self::Slices<'a>;

    /// Converts properly typed mutable slice pointers to each field of [`Fields`](Soa::Fields)
    /// to their mutable slices by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each mutable slice pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn slice_ptrs_to_slices_mut<'a>(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicesMut<'a>;

    /// Returns the number of elements in slices to each field of [`Fields`](Soa::Fields),
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the slices,
    /// or else this method could panic.
    fn slices_len(context: &Self::Context, slices: &Self::Slices<'_>) -> usize;

    /// Returns the number of elements in slices to each field of [`Fields`](Soa::Fields),
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the mutable slices,
    /// or else this method could panic.
    fn slices_len_mut(context: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize;

    /// Converts properly typed slices to each field of [`Fields`](Soa::Fields)
    /// to their slice pointers by taking the pointer of each one of them.
    fn slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::Slices<'_>,
    ) -> Self::SlicePtrs;

    /// Converts properly typed mutable slices to each field of [`Fields`](Soa::Fields)
    /// to their mutable slice pointers by taking the pointer of each one of them.
    fn mut_slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::SliceMutPtrs;

    /// Converts properly typed mutable slices to each field of [`Fields`](Soa::Fields)
    /// to their slices by explicitly converting each one of them via `&*` operator combination.
    fn mut_slices_as_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicesMut<'a>,
    ) -> Self::Slices<'a>;

    /// Returns properly typed pointers to the slice's buffer
    /// of each slice of [`Fields`](Soa::Fields).
    fn slice_refs_as_ptrs(context: &Self::Context, slices: Self::Slices<'_>) -> Self::Ptrs;

    /// Returns properly typed mutable pointers to the slice's buffer
    /// of each mutable slice of [`Fields`](Soa::Fields).
    fn mut_slice_refs_as_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::MutPtrs;

    /// Executes the destructors (if any) for the each slice of [`Fields`](Soa::Fields).
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::drop_in_place()`](core::ptr::drop_in_place) method to each slice pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// By default, this method just iterates by all the fields of slices and drops such fields one by one.
    unsafe fn slices_drop_in_place(context: &Self::Context, slices: Self::SliceMutPtrs) {
        let len = Self::slice_ptrs_len_mut(context, &slices);
        let ptrs = Self::mut_slice_ptrs_as_ptrs(context, slices);
        for index in 0..len {
            unsafe {
                let ptrs = Self::ptrs_add_mut(context, ptrs.clone(), index);
                Self::ptrs_drop_in_place(context, ptrs);
            }
        }
    }
}

/// Default implementation of [`buffer_layout()`](Soa::buffer_layout) method of [`Soa`] trait
/// for collections implementing [`FromIterator`] trait.
pub fn buffer_layout<B, I>(field_layouts: I, capacity: usize) -> Result<(Layout, B), LayoutError>
where
    I: IntoIterator<Item: Borrow<Layout>>,
    B: FromIterator<usize>,
{
    let mut layout = Layout::new::<()>();
    let offsets = field_layouts
        .into_iter()
        .map(|item| {
            let repeated = repeat_layout(item.borrow(), capacity)?;
            let offset;
            (layout, offset) = layout.extend(repeated)?;
            Ok(offset)
        })
        .collect::<Result<_, _>>()?;

    Ok((layout, offsets))
}

/// Use this until [`Layout::repeat()`] is stabilized
fn repeat_layout(layout: &Layout, n: usize) -> Result<Layout, LayoutError> {
    const ERR: LayoutError = match Layout::from_size_align(usize::MAX, 1) {
        Ok(_) => unreachable!(),
        Err(err) => err,
    };

    let layout = layout.pad_to_align();
    let size = match layout.size().checked_mul(n) {
        Some(v) => v,
        None => return Err(ERR),
    };
    Layout::from_size_align(size, layout.align())
}

pub trait SoaToOwned<'a> {
    type Owned: Soa<Refs<'a> = Self>
    where
        Self: 'a;

    fn to_owned(&self) -> Self::Owned;

    fn clone_into(&self, target: &mut Self::Owned) {
        *target = self.to_owned();
    }

    unsafe fn clone_into_ptrs(
        &self,
        context: &<Self::Owned as Soa>::Context,
        target: <Self::Owned as Soa>::MutPtrs,
    ) {
        let owned = self.to_owned();
        unsafe {
            <Self::Owned as Soa>::ptrs_write(context, target, owned);
        }
    }

    fn clone_into_refs(
        &self,
        context: &<Self::Owned as Soa>::Context,
        target: <Self::Owned as Soa>::RefsMut<'_>,
    ) {
        let target = <Self::Owned as Soa>::mut_refs_as_ptrs(context, target);
        unsafe {
            self.clone_into_ptrs(context, target);
        }
    }
}

/// Marker trait which places additional safety requirements
/// on the [`Fields`](Soa::Fields) associated type of [`Soa`] trait implementations.
///
/// These safety requirements are:
/// - sum of layouts' sizes of [`FieldDescriptors`](Soa::FieldDescriptors)
///   should be less or equal to the size of [`Fields`](Soa::Fields)
/// - alignment of each layout of [`FieldDescriptors`](Soa::FieldDescriptors)
///   should be less or equal to the alignment of [`Fields`](Soa::Fields)
pub unsafe trait SoaTrustedFields: Soa {}
