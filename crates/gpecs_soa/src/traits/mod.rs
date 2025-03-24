use core::{
    alloc::{Layout, LayoutError},
    any::type_name,
    borrow::Borrow,
    ptr,
};

pub use desc::{DropFn, FieldDescriptor};

#[doc(hidden)]
pub mod impls;

mod desc;

#[allow(clippy::missing_safety_doc)]
pub unsafe trait Soa: Sized {
    /// Type of context used to perform all operations of this trait.
    ///
    /// Most of the time, this should be [unit](prim@unit) type.
    /// This is true for all the types with fields' size and alignment known at compile-time.
    type Context;

    /// Special type used to properly allocate a buffer in memory.
    /// This should contain all the fields which are stored inside of a buffer
    /// ([`Copy`], [`Send`], [`Sync`] and some other bounds are defined with account to this type, not `Self`).
    ///
    /// Most of the time, this should be the same as `Self`.
    /// This is true for such implementations which store all the fields of self.
    type Fields;

    /// Collection of [descriptors][`FieldDescriptor`] for each field.
    ///
    /// Safety requirements:
    /// - order of descriptors should resemble their order inside of a buffer in memory
    /// - sum of layouts' sizes of descriptors should be less or equal to the size of [`Fields`](`Soa::Fields`)
    /// - alignment of each layout of descriptors should be less or equal to the alignment of [`Fields`](`Soa::Fields`)
    type FieldDescriptors<'a>: IntoIterator<Item: AsRef<FieldDescriptor>>;

    fn field_descriptors(context: &Self::Context) -> Self::FieldDescriptors<'_>;

    type FieldOffsets<'a>: IntoIterator<Item = usize>;

    /// Calculates layout needed to store `capacity` number of fields inside of a buffer.
    /// Also returns offsets of each field in the buffer (in bytes).
    ///
    /// This layout should not include [`Context`](`Soa::Context`),
    /// as it is handled by the crate itself.
    fn buffer_layout(
        context: &Self::Context,
        capacity: usize,
    ) -> Result<(Layout, Self::FieldOffsets<'_>), LayoutError>;

    /// Retrieves maximum number of fields that can be stored inside of a buffer with given layout.
    fn capacity_from(context: &Self::Context, buffer_layout: Layout) -> usize {
        let packed_size = Self::field_descriptors(context)
            .into_iter()
            .map(|descriptor| descriptor.as_ref().layout().size())
            .sum();
        let max_capacity = buffer_layout
            .size()
            .checked_div(packed_size)
            .unwrap_or_default();

        let mut capacity = max_capacity;
        while {
            let (layout, _) = Self::buffer_layout(context, capacity)
                .expect("new buffer layout should be smaller than the input one");
            layout.size() > buffer_layout.size()
        } {
            capacity -= 1;
        }
        capacity
    }

    type Ptrs: Clone;
    type MutPtrs: Clone;

    type ErasedPtrs: IntoIterator<Item = *const u8>;
    type ErasedMutPtrs: IntoIterator<Item = *mut u8>;

    /// Order of erased pointers should resemble the order of fields inside of a buffer in memory.
    fn ptrs_erase(context: &Self::Context, ptrs: Self::Ptrs) -> Self::ErasedPtrs;

    /// Order of erased pointers should resemble the order of fields inside of a buffer in memory.
    fn ptrs_erase_mut(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::ErasedMutPtrs;

    /// Order of input pointers should resemble the order of fields inside of a buffer in memory.
    fn ptrs_restore(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *const u8>,
    ) -> Self::Ptrs;

    /// Order of input pointers should resemble the order of fields inside of a buffer in memory.
    fn ptrs_restore_mut(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *mut u8>,
    ) -> Self::MutPtrs;

    fn ptrs_dangling(context: &Self::Context) -> Self::MutPtrs;

    fn ptrs_cast_const(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::Ptrs;
    fn ptrs_cast_mut(context: &Self::Context, ptrs: Self::Ptrs) -> Self::MutPtrs;

    unsafe fn ptrs_add(context: &Self::Context, ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs;

    unsafe fn ptrs_add_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        offset: usize,
    ) -> Self::MutPtrs;

    unsafe fn ptrs_offset_from(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        origin: Self::Ptrs,
    ) -> isize;

    unsafe fn ptrs_offset_from_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        origin: Self::Ptrs,
    ) -> isize;

    unsafe fn ptrs_swap(context: &Self::Context, a: Self::MutPtrs, b: Self::MutPtrs);

    unsafe fn ptrs_copy(context: &Self::Context, src: Self::Ptrs, dst: Self::MutPtrs, len: usize);

    unsafe fn ptrs_copy_rev(
        context: &Self::Context,
        src: Self::Ptrs,
        dst: Self::MutPtrs,
        len: usize,
    );

    unsafe fn ptrs_copy_nonoverlapping(
        context: &Self::Context,
        src: Self::Ptrs,
        dst: Self::MutPtrs,
        len: usize,
    );

    unsafe fn ptrs_read(context: &Self::Context, src: Self::Ptrs) -> Self;

    unsafe fn ptrs_write(context: &Self::Context, dst: Self::MutPtrs, value: Self);

    unsafe fn ptrs_drop_in_place(context: &Self::Context, ptrs: Self::MutPtrs) {
        let ptrs = Self::ptrs_cast_const(context, ptrs);
        let value = unsafe { Self::ptrs_read(context, ptrs) };
        drop(value);
    }

    type NonNullPtrs: Clone;

    unsafe fn ptrs_to_nonnull(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::NonNullPtrs;
    fn nonnull_to_ptrs(context: &Self::Context, ptrs: Self::NonNullPtrs) -> Self::MutPtrs;

    type Vecs;

    fn vecs_with_capacity(context: &Self::Context, capacity: usize) -> Self::Vecs;
    fn vecs_as_ptrs(context: &Self::Context, vecs: &Self::Vecs) -> Self::Ptrs;
    fn mut_vecs_as_ptrs(context: &Self::Context, vecs: &mut Self::Vecs) -> Self::MutPtrs;
    fn vecs_len(context: &Self::Context, vecs: &Self::Vecs) -> usize;
    unsafe fn vecs_set_len(context: &Self::Context, vecs: &mut Self::Vecs, len: usize);

    type Refs<'a>
    where
        Self: 'a;

    type RefsMut<'a>
    where
        Self: 'a;

    unsafe fn ptrs_to_refs<'a>(context: &Self::Context, ptrs: Self::Ptrs) -> Self::Refs<'a>;

    unsafe fn ptrs_to_refs_mut<'a>(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
    ) -> Self::RefsMut<'a>;

    fn refs_as_ptrs(context: &Self::Context, refs: Self::Refs<'_>) -> Self::Ptrs;
    fn mut_refs_as_ptrs(context: &Self::Context, refs: Self::RefsMut<'_>) -> Self::MutPtrs;
    fn mut_refs_as_refs<'a>(context: &Self::Context, refs: Self::RefsMut<'a>) -> Self::Refs<'a>;

    type SlicePtrs: Clone;
    type SliceMutPtrs: Clone;

    fn slices_from_raw_parts(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        len: usize,
    ) -> Self::SlicePtrs;

    fn slices_from_raw_parts_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        len: usize,
    ) -> Self::SliceMutPtrs;

    fn slice_ptrs_cast_const(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicePtrs;

    fn slice_ptrs_cast_mut(context: &Self::Context, slices: Self::SlicePtrs) -> Self::SliceMutPtrs;

    fn slice_ptrs_len(context: &Self::Context, slices: Self::SlicePtrs) -> usize;

    fn slice_ptrs_len_mut(context: &Self::Context, slices: Self::SliceMutPtrs) -> usize;

    fn slice_ptrs_as_ptrs(context: &Self::Context, slices: Self::SlicePtrs) -> Self::Ptrs;

    fn mut_slice_ptrs_as_ptrs(context: &Self::Context, slices: Self::SliceMutPtrs)
        -> Self::MutPtrs;

    type Slices<'a>
    where
        Self: 'a;

    type SlicesMut<'a>
    where
        Self: 'a;

    unsafe fn slice_ptrs_to_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicePtrs,
    ) -> Self::Slices<'a>;

    unsafe fn slice_ptrs_to_slices_mut<'a>(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicesMut<'a>;

    fn slices_len(context: &Self::Context, slices: &Self::Slices<'_>) -> usize;

    fn slices_len_mut(context: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize;

    fn slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::Slices<'_>,
    ) -> Self::SlicePtrs;

    fn mut_slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::SliceMutPtrs;

    fn mut_slices_as_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicesMut<'a>,
    ) -> Self::Slices<'a>;

    fn slice_refs_as_ptrs(context: &Self::Context, slices: Self::Slices<'_>) -> Self::Ptrs;

    fn mut_slice_refs_as_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::MutPtrs;

    unsafe fn slices_drop_in_place(context: &Self::Context, slices: Self::SliceMutPtrs) {
        let len = Self::slice_ptrs_len_mut(context, slices.clone());
        let ptrs = Self::mut_slice_ptrs_as_ptrs(context, slices);
        for index in 0..len {
            unsafe {
                let ptrs = Self::ptrs_add_mut(context, ptrs.clone(), index);
                Self::ptrs_drop_in_place(context, ptrs);
            }
        }
    }
}

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

#[inline]
#[track_caller]
pub(super) fn debug_assert_ptr_is_aligned<T>(ptr: *const T) {
    debug_assert!(
        ptr.is_aligned(),
        "pointer of {} should be aligned to {}\nits current align offset (in bytes) is {}",
        type_name::<T>(),
        align_of::<T>(),
        ptr.cast::<u8>().align_offset(align_of::<T>()),
    )
}

#[inline]
#[track_caller]
pub(super) unsafe fn drop_unaligned(to_drop: *mut u8, desc: &FieldDescriptor, temp: &mut [u8]) {
    let Some(drop_fn) = desc.drop_fn() else {
        return;
    };
    assert!(
        temp.len() >= desc.layout().size() * 2,
        "temp buffer should be at least twice as big as the field layout size of {} to hold for any buffer alignment",
        desc.layout().size(),
    );

    let offset = temp.as_mut_ptr().align_offset(desc.layout().align());
    let temp = &mut temp[offset..];

    let dst = temp.as_mut_ptr();
    unsafe {
        ptr::copy_nonoverlapping(to_drop, dst, desc.layout().size());
        drop_fn(dst);
    }
}
