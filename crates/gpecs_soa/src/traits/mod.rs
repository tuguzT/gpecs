use core::{
    alloc::{Layout, LayoutError},
    borrow::Borrow,
};

pub use self::desc::FieldDescriptor;

#[doc(hidden)]
pub mod impls;

mod desc;

pub type DefaultContext = ();

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

    /// Non-empty collection of [descriptors](FieldDescriptor) for each field.
    ///
    /// Order of such descriptors **MUST** resemble their order inside of a buffer in memory.
    type FieldDescriptors<'context>: IntoIterator<Item: AsRef<FieldDescriptor>>;

    /// Returns [field descriptors](Soa::FieldDescriptors) for each field of [`Fields`](Soa::Fields).
    fn field_descriptors(context: &Self::Context) -> Self::FieldDescriptors<'_>;

    /// Non-empty collection of offsets (in bytes) for each field of [`Fields`](Soa::Fields).
    ///
    /// Each of these offsets **MUST** correspond to the offset of the field in the buffer
    /// in the order defined by [`FieldDescriptors`](Soa::FieldDescriptors).
    ///
    /// These offsets should not include offset to the [`Context`](Soa::Context),
    /// as it is handled by the crate itself.
    type FieldOffsets<'context>: IntoIterator<Item = usize>;

    /// Calculates layout needed to store `capacity` number of fields inside of a buffer.
    /// Also returns non-empty collection of offsets for each field of [`Fields`](Soa::Fields) (in bytes).
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

    /// Non-empty collection of properly typed pointers to each field of [`Fields`](Soa::Fields).
    ///
    /// Unlike [`ErasedPtrs`](Soa::ErasedPtrs),
    /// order of such pointers **may not** resemble their order inside of a buffer in memory.
    /// Reordering of such pointers in other methods is up to the implementation of this trait.
    ///
    /// This type should be covariant over given lifetime.
    type Ptrs<'context>: Clone;

    /// Non-empty collection of properly typed mutable pointers to each field of [`Fields`](Soa::Fields).
    ///
    /// Unlike [`ErasedMutPtrs`](Soa::ErasedMutPtrs),
    /// order of such pointers **may not** resemble their order inside of a buffer in memory.
    /// Reordering of such pointers in other methods is up to the implementation of this trait.
    ///
    /// This type should be covariant over given lifetime.
    type MutPtrs<'context>: Clone;

    /// Non-empty collection of pointers to the bytes of each field of [`Fields`](Soa::Fields).
    ///
    /// Unlike [`Ptrs`](Soa::Ptrs),
    /// order of such pointers **MUST** resemble their order inside of a buffer in memory.
    ///
    /// This type should be covariant over given lifetime.
    type ErasedPtrs<'context>: IntoIterator<Item = *const u8>;

    /// Non-empty collection of mutable pointers to the bytes of each field of [`Fields`](Soa::Fields).
    ///
    /// Unlike [`MutPtrs`](Soa::MutPtrs),
    /// order of such pointers **MUST** resemble their order inside of a buffer in memory.
    ///
    /// This type should be covariant over given lifetime.
    type ErasedMutPtrs<'context>: IntoIterator<Item = *mut u8>;

    /// Erases pointers of each field of [`Fields`](Soa::Fields).
    ///
    /// Process of 'erasing' just means converting properly typed pointers to the pointers of bytes.
    fn ptrs_erase<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::ErasedPtrs<'context>;

    /// Erases mutable pointers of each field of [`Fields`](Soa::Fields).
    ///
    /// Process of 'erasing' just means converting properly typed pointers to the pointers of bytes.
    fn ptrs_erase_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::ErasedMutPtrs<'context>;

    /// Restores pointers to bytes of each field of [`Fields`](Soa::Fields).
    ///
    /// Process of 'restoring' just means converting pointers of bytes to properly typed pointers.
    fn ptrs_restore(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *const u8>,
    ) -> Self::Ptrs<'_>;

    /// Restores mutable pointers to bytes of each field of [`Fields`](Soa::Fields).
    ///
    /// Process of 'restoring' just means converting pointers of bytes to properly typed pointers.
    fn ptrs_restore_mut(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *mut u8>,
    ) -> Self::MutPtrs<'_>;

    /// Returns dangling pointers to each field of [`Fields`](Soa::Fields).
    fn ptrs_dangling(context: &Self::Context) -> Self::MutPtrs<'_>;

    /// Converts pointers to the mutable ones of each field of [`Fields`](Soa::Fields).
    fn ptrs_cast_const<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::Ptrs<'context>;

    /// Converts mutable pointers to the const ones of each field of [`Fields`](Soa::Fields).
    fn ptrs_cast_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::MutPtrs<'context>;

    /// Adds an unsigned offset to each properly typed pointer.
    ///
    /// All the safety requirements resulting from applying [`pointer::add()`] method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// [`pointer::add()`]: https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.add
    unsafe fn ptrs_add<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
        offset: usize,
    ) -> Self::Ptrs<'context>;

    /// Adds an unsigned offset to each mutable properly typed pointer.
    ///
    /// All the safety requirements resulting from applying [`pointer::add()`] method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// [`pointer::add()`]: https://doc.rust-lang.org/stable/core/primitive.pointer.html#method.add-1
    unsafe fn ptrs_add_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
        offset: usize,
    ) -> Self::MutPtrs<'context>;

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
        ptrs: Self::Ptrs<'_>,
        origin: Self::Ptrs<'_>,
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
        ptrs: Self::MutPtrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize;

    /// Swaps the values at two mutable locations of the [`Fields`](Soa::Fields)
    /// sequentially in the same order as they are stored in the buffer,
    /// without deinitializing either.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::swap()`](core::ptr::swap) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_swap(context: &Self::Context, a: Self::MutPtrs<'_>, b: Self::MutPtrs<'_>);

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
    unsafe fn ptrs_copy(
        context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    );

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
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
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
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    );

    /// Constructs the value from reading each field to which src points without moving them.
    /// This leaves the memory in src unchanged.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::read()`](core::ptr::read) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_read(context: &Self::Context, src: Self::Ptrs<'_>) -> Self;

    /// Overwrites a memory location of each field of [`Fields`](Soa::Fields)
    /// with the given value without reading or dropping the old value.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::write()`](core::ptr::write) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_write(context: &Self::Context, dst: Self::MutPtrs<'_>, value: Self);

    /// Executes the destructors (if any) for the each field of [`Fields`](Soa::Fields) located at ptrs.
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::drop_in_place()`](core::ptr::drop_in_place) method to each pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// By default, this method just reads the value from ptrs on the stack and then drops it.
    unsafe fn ptrs_drop_in_place<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) {
        let ptrs = Self::ptrs_cast_const(context, ptrs);
        let value = unsafe { Self::ptrs_read(context, ptrs) };
        drop(value);
    }

    /// Non-empty collection of properly typed non-null pointers to each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such pointers **may not** resemble their order inside of a buffer in memory.
    /// Reordering of such pointers in other methods is up to the implementation of this trait.
    ///
    /// This type should be covariant over given lifetime.
    type NonNullPtrs<'context>: Clone;

    /// Creates properly typed non-null pointers to each field of [`Fields`](Soa::Fields).
    ///
    /// All the safety requirements resulting from applying
    /// [`NonNull::new_unchecked()`](core::ptr::NonNull::new_unchecked) method to each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_to_nonnull<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::NonNullPtrs<'context>;

    /// Acquires the underlying pointers from non-null pointers to each field of [`Fields`](Soa::Fields).
    fn nonnull_to_ptrs<'context>(
        context: &'context Self::Context,
        ptrs: Self::NonNullPtrs<'context>,
    ) -> Self::MutPtrs<'context>;

    /// Non-empty collection of properly typed references to each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such references **may not** resemble their order inside of a buffer in memory.
    type Refs<'context, 'a>
    where
        Self: 'a;

    /// Non-empty collection of properly typed mutable references to each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such references **may not** resemble their order inside of a buffer in memory.
    type RefsMut<'context, 'a>
    where
        Self: 'a;

    /// Converts properly typed pointers to each field of [`Fields`](Soa::Fields)
    /// to their references by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_to_refs<'context, 'a>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::Refs<'context, 'a>;

    /// Converts properly typed mutable pointers to each field of [`Fields`](Soa::Fields)
    /// to their mutable references by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn ptrs_to_refs_mut<'context, 'a>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::RefsMut<'context, 'a>;

    /// Converts properly typed references to each field of [`Fields`](Soa::Fields)
    /// to their pointers by taking the pointer of each one of them.
    fn refs_as_ptrs<'context>(
        context: &'context Self::Context,
        refs: Self::Refs<'context, '_>,
    ) -> Self::Ptrs<'context>;

    /// Converts properly typed mutable references to each field of [`Fields`](Soa::Fields)
    /// to their mutable pointers by taking the pointer of each one of them.
    fn mut_refs_as_ptrs<'context>(
        context: &'context Self::Context,
        refs: Self::RefsMut<'context, '_>,
    ) -> Self::MutPtrs<'context>;

    /// Converts properly typed mutable references to each field of [`Fields`](Soa::Fields)
    /// to their references by explicitly converting each one of them via `&*` operator combination.
    fn mut_refs_as_refs<'context, 'a>(
        context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> Self::Refs<'context, 'a>;

    /// Non-empty collection of slice pointers to each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such pointers **may not** resemble their order inside of a buffer in memory.
    ///
    /// This type should be covariant over given lifetime.
    type SlicePtrs<'context>: Clone;

    /// Non-empty collection of mutable slice pointers to each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such pointers **may not** resemble their order inside of a buffer in memory.
    ///
    /// This type should be covariant over given lifetime.
    type SliceMutPtrs<'context>: Clone;

    /// Forms slice pointers to each field of [`Fields`](Soa::Fields)
    /// from pointers to each field and a length.
    ///
    /// The len argument is the number of elements, not the number of bytes.
    fn slices_from_raw_parts<'context>(
        context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
        len: usize,
    ) -> Self::SlicePtrs<'context>;

    /// Forms mutable slice pointers to each field of [`Fields`](Soa::Fields)
    /// from mutable pointers to each field and a length.
    ///
    /// The len argument is the number of elements, not the number of bytes.
    fn slices_from_raw_parts_mut<'context>(
        context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
        len: usize,
    ) -> Self::SliceMutPtrs<'context>;

    /// Converts slice pointers to the mutable ones of each field of [`Fields`](Soa::Fields).
    fn slice_ptrs_cast_const<'context>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::SlicePtrs<'context>;

    /// Converts mutable slice pointers to the const ones of each field of [`Fields`](Soa::Fields).
    fn slice_ptrs_cast_mut<'context>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::SliceMutPtrs<'context>;

    /// Returns the number of elements in slices to each slice pointer of [`Fields`](Soa::Fields),
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the slice pointers,
    /// or else this method could panic.
    fn slice_ptrs_len(context: &Self::Context, slices: &Self::SlicePtrs<'_>) -> usize;

    /// Returns the number of elements in slices to each mutable slice pointer of [`Fields`](Soa::Fields),
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the mutable slice pointers,
    /// or else this method could panic.
    fn slice_ptrs_len_mut(context: &Self::Context, slices: &Self::SliceMutPtrs<'_>) -> usize;

    /// Returns properly typed pointers to the slice's buffer
    /// of each slice pointer of [`Fields`](Soa::Fields).
    fn slice_ptrs_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context>;

    /// Returns properly typed mutable pointers to the slice's buffer
    /// of each mutable slice pointer of [`Fields`](Soa::Fields).
    fn mut_slice_ptrs_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context>;

    /// Non-empty collection of properly typed slices of each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such slices may not resemble their order inside of a buffer in memory.
    type Slices<'context, 'a>
    where
        Self: 'a;

    /// Non-empty collection of properly typed mutable slices of each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such slices may not resemble their order inside of a buffer in memory.
    type SlicesMut<'context, 'a>
    where
        Self: 'a;

    /// Converts properly typed slice pointers to each field of [`Fields`](Soa::Fields)
    /// to their slices by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each slice pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn slice_ptrs_to_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::Slices<'context, 'a>;

    /// Converts properly typed mutable slice pointers to each field of [`Fields`](Soa::Fields)
    /// to their mutable slices by dereferencing each one of them.
    ///
    /// All the safety requirements resulting from dereferencing of each mutable slice pointer
    /// should be satisfied to be safe to call this method.
    unsafe fn slice_ptrs_to_slices_mut<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::SlicesMut<'context, 'a>;

    /// Returns the number of elements in slices to each field of [`Fields`](Soa::Fields),
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the slices,
    /// or else this method could panic.
    fn slices_len(context: &Self::Context, slices: &Self::Slices<'_, '_>) -> usize;

    /// Returns the number of elements in slices to each field of [`Fields`](Soa::Fields),
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the mutable slices,
    /// or else this method could panic.
    fn slices_len_mut(context: &Self::Context, slices: &Self::SlicesMut<'_, '_>) -> usize;

    /// Converts properly typed slices to each field of [`Fields`](Soa::Fields)
    /// to their slice pointers by taking the pointer of each one of them.
    fn slice_refs_as_slice_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::Slices<'context, '_>,
    ) -> Self::SlicePtrs<'context>;

    /// Converts properly typed mutable slices to each field of [`Fields`](Soa::Fields)
    /// to their mutable slice pointers by taking the pointer of each one of them.
    fn mut_slice_refs_as_slice_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, '_>,
    ) -> Self::SliceMutPtrs<'context>;

    /// Converts properly typed mutable slices to each field of [`Fields`](Soa::Fields)
    /// to their slices by explicitly converting each one of them via `&*` operator combination.
    fn mut_slices_as_slices<'context, 'a>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::Slices<'context, 'a>;

    /// Returns properly typed pointers to the slice's buffer
    /// of each slice of [`Fields`](Soa::Fields).
    fn slice_refs_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::Slices<'context, '_>,
    ) -> Self::Ptrs<'context>;

    /// Returns properly typed mutable pointers to the slice's buffer
    /// of each mutable slice of [`Fields`](Soa::Fields).
    fn mut_slice_refs_as_ptrs<'context>(
        context: &'context Self::Context,
        slices: Self::SlicesMut<'context, '_>,
    ) -> Self::MutPtrs<'context>;

    /// Executes the destructors (if any) for the each slice of [`Fields`](Soa::Fields).
    ///
    /// All the safety requirements resulting from applying
    /// [`ptr::drop_in_place()`](core::ptr::drop_in_place) method to each slice pointer
    /// should be satisfied to be safe to call this method.
    ///
    /// By default, this method just iterates by all the fields of slices and drops such fields one by one.
    unsafe fn slices_drop_in_place<'context>(
        context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) {
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
#[inline]
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
#[inline]
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
    /// usually by cloning each field of [`Fields`](Soa::Fields).
    fn to_owned(&self) -> Self::Owned;

    /// Uses borrowed data to replace owned data,
    /// usually by cloning each field of [`Fields`](Soa::Fields).
    ///
    /// This is borrow-generalized version of [`Clone::clone_from()`].
    fn clone_into(&self, target: &mut Self::Owned) {
        *target = self.to_owned();
    }

    /// Uses borrowed data to replace owned data located by `target` pointers,
    /// usually by cloning each field of [`Fields`](Soa::Fields).
    unsafe fn clone_into_ptrs(
        &self,
        context: &<Self::Owned as Soa>::Context,
        target: <Self::Owned as Soa>::MutPtrs<'_>,
    ) {
        let owned = self.to_owned();
        unsafe {
            <Self::Owned as Soa>::ptrs_write(context, target, owned);
        }
    }

    /// Uses borrowed data to replace owned data located by `target` references,
    /// usually by cloning each field of [`Fields`](Soa::Fields).
    fn clone_into_refs<'c>(
        &self,
        context: &'c <Self::Owned as Soa>::Context,
        target: <Self::Owned as Soa>::RefsMut<'c, '_>,
    ) {
        let target = <Self::Owned as Soa>::mut_refs_as_ptrs(context, target);
        unsafe {
            self.clone_into_ptrs(context, target);
        }
    }
}

/// Extension trait which allows to convert SoA vector of [`Fields`](Soa::Fields)
/// into vectors of each field of [`Fields`](Soa::Fields).
pub unsafe trait SoaVecs: Soa {
    /// Non-empty collection of properly typed vectors of each field of [`Fields`](Soa::Fields).
    ///
    /// Order of such vectors **may not** resemble their order inside of a buffer in memory.
    ///
    /// Most of the time type of these vectors is [`Vec`].
    ///
    /// [`Vec`]: https://doc.rust-lang.org/stable/alloc/vec/struct.Vec.html
    type Vecs;

    /// Constructs new, empty vectors for each field of [`Fields`](Soa::Fields)
    /// with at least the specified capacity.
    fn vecs_with_capacity(context: &Self::Context, capacity: usize) -> Self::Vecs;

    /// Returns properly typed pointers to the vectors' buffers of each field of [`Fields`](Soa::Fields),
    /// or a dangling pointers valid for zero sized reads if these vectors didn’t allocate.
    fn vecs_as_ptrs<'context>(
        context: &'context Self::Context,
        vecs: &Self::Vecs,
    ) -> Self::Ptrs<'context>;

    /// Returns properly typed mutable pointers to the vectors' buffers of each field of [`Fields`](Soa::Fields),
    /// or a dangling pointers valid for zero sized reads if these vectors didn’t allocate.
    fn mut_vecs_as_ptrs<'context>(
        context: &'context Self::Context,
        vecs: &mut Self::Vecs,
    ) -> Self::MutPtrs<'context>;

    /// Returns the number of elements in each vector of [`Fields`](Soa::Fields),
    /// also referred to as their 'length'.
    ///
    /// Note that resulting lengths should be the same for all the vectors,
    /// or else this method could panic.
    fn vecs_len(context: &Self::Context, vecs: &Self::Vecs) -> usize;

    /// Forces the length of the vectors of [`Fields`](Soa::Fields) to `new_len`.
    ///
    /// All the safety requirements resulting from applying
    /// [`Vec::set_len()`] method to each vector
    /// should be satisfied to be safe to call this method.
    ///
    /// [`Vec::set_len()`]: https://doc.rust-lang.org/stable/alloc/vec/struct.Vec.html#method.set_len
    unsafe fn vecs_set_len(context: &Self::Context, vecs: &mut Self::Vecs, len: usize);
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
