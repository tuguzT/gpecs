use alloc::vec::Vec;
use core::{
    alloc::{Layout, LayoutError},
    array,
    borrow::Borrow,
    iter,
    marker::PhantomData,
    mem,
    ptr::{self, NonNull},
    slice,
};

pub type DropFn = unsafe fn(to_drop: *mut u8);

#[derive(Debug, Clone, Copy)]
pub struct FieldDescriptor {
    layout: Layout,
    drop_fn: Option<DropFn>,
}

impl FieldDescriptor {
    #[inline]
    pub fn new<D>(layout: Layout, drop_fn: D) -> Self
    where
        D: Into<Option<DropFn>>,
    {
        Self {
            layout,
            drop_fn: drop_fn.into(),
        }
    }

    #[inline]
    pub const fn of<T>() -> Self {
        let layout = Layout::new::<T>();
        let drop_fn = if mem::needs_drop::<T>() {
            let drop_fn: DropFn = |to_drop| unsafe {
                let to_drop = to_drop.cast();
                ptr::drop_in_place::<T>(to_drop);
            };
            Some(drop_fn)
        } else {
            None
        };

        Self { layout, drop_fn }
    }

    #[inline]
    pub const fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub const fn drop_fn(&self) -> Option<DropFn> {
        let Self { drop_fn, .. } = *self;
        drop_fn
    }

    #[inline]
    pub const fn into_inner(self) -> (Layout, Option<DropFn>) {
        let Self { layout, drop_fn } = self;
        (layout, drop_fn)
    }
}

#[allow(clippy::missing_safety_doc)]
#[inline]
#[track_caller]
pub unsafe fn drop_unaligned(to_drop: *mut u8, descriptor: &FieldDescriptor, temp: &mut [u8]) {
    let Some(drop_fn) = descriptor.drop_fn() else {
        return;
    };

    let offset = to_drop.align_offset(descriptor.layout().align());
    assert!(offset + descriptor.layout().size() <= temp.len());

    let temp = &raw mut temp[offset];
    unsafe {
        ptr::copy_nonoverlapping(to_drop, temp, descriptor.layout().size());
        drop_fn(temp);
    }
}

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
    type FieldDescriptors<'a>: IntoIterator<Item: Borrow<FieldDescriptor>>;

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
            .map(|descriptor| descriptor.borrow().layout().size())
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

/// Use this until [`Layout::repeat()`] is stabilized
const fn repeat_layout(layout: &Layout, n: usize) -> Result<Layout, LayoutError> {
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

unsafe impl Soa for () {
    type Context = Self;
    type Fields = Self;
    type FieldDescriptors<'a> = [FieldDescriptor; 1];

    #[inline]
    fn field_descriptors(_: &Self::Context) -> Self::FieldDescriptors<'_> {
        [FieldDescriptor::of::<Self>()]
    }

    type FieldOffsets<'a> = [usize; 1];

    #[inline]
    fn buffer_layout(
        _: &Self::Context,
        _: usize,
    ) -> Result<(Layout, Self::FieldOffsets<'_>), LayoutError> {
        Ok((Layout::new::<Self>(), [0]))
    }

    #[inline]
    fn capacity_from(_: &Self::Context, _: Layout) -> usize {
        usize::MAX
    }

    type Ptrs = *const Self;
    type MutPtrs = *mut Self;

    type ErasedPtrs = iter::Once<*const u8>;
    type ErasedMutPtrs = iter::Once<*mut u8>;

    #[inline]
    fn ptrs_erase(_: &Self::Context, ptrs: Self::Ptrs) -> Self::ErasedPtrs {
        iter::once(ptrs.cast())
    }

    #[inline]
    fn ptrs_erase_mut(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::ErasedMutPtrs {
        iter::once(ptrs.cast())
    }

    #[track_caller]
    #[inline]
    fn ptrs_restore(_: &Self::Context, ptrs: impl IntoIterator<Item = *const u8>) -> Self::Ptrs {
        let ptrs: [*const u8; 1] = collect_array(ptrs);
        ptrs[0].cast()
    }

    #[track_caller]
    #[inline]
    fn ptrs_restore_mut(
        _: &Self::Context,
        ptrs: impl IntoIterator<Item = *mut u8>,
    ) -> Self::MutPtrs {
        let ptrs: [*mut u8; 1] = collect_array(ptrs);
        ptrs[0].cast()
    }

    #[inline]
    fn ptrs_dangling(_: &Self::Context) -> Self::MutPtrs {
        ptr::dangling_mut()
    }

    #[inline]
    fn ptrs_cast_const(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::Ptrs {
        ptrs.cast_const()
    }

    #[inline]
    fn ptrs_cast_mut(_: &Self::Context, ptrs: Self::Ptrs) -> Self::MutPtrs {
        ptrs.cast_mut()
    }

    #[inline]
    unsafe fn ptrs_add(_: &Self::Context, ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs {
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_add_mut(_: &Self::Context, ptrs: Self::MutPtrs, offset: usize) -> Self::MutPtrs {
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(_: &Self::Context, ptrs: Self::Ptrs, origin: Self::Ptrs) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    #[inline]
    unsafe fn ptrs_offset_from_mut(
        _: &Self::Context,
        ptrs: Self::MutPtrs,
        origin: Self::Ptrs,
    ) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    #[inline]
    unsafe fn ptrs_swap(_: &Self::Context, a: Self::MutPtrs, b: Self::MutPtrs) {
        unsafe { ptr::swap(a, b) }
    }

    #[inline]
    unsafe fn ptrs_copy(_: &Self::Context, src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_rev(_: &Self::Context, src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        _: &Self::Context,
        src: Self::Ptrs,
        dst: Self::MutPtrs,
        len: usize,
    ) {
        unsafe { ptr::copy_nonoverlapping(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_read(_: &Self::Context, ptrs: Self::Ptrs) -> Self {
        unsafe { ptr::read(ptrs) }
    }

    #[inline]
    unsafe fn ptrs_write(_: &Self::Context, ptrs: Self::MutPtrs, value: Self) {
        unsafe { ptr::write(ptrs, value) }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(_: &Self::Context, ptrs: Self::MutPtrs) {
        unsafe { ptr::drop_in_place(ptrs) }
    }

    type NonNullPtrs = NonNull<Self>;

    #[inline]
    unsafe fn ptrs_to_nonnull(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
        unsafe { NonNull::new_unchecked(ptrs) }
    }

    #[inline]
    fn nonnull_to_ptrs(_: &Self::Context, ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
        ptrs.as_ptr()
    }

    type Vecs = Vec<Self>;

    #[inline]
    fn vecs_with_capacity(_: &Self::Context, capacity: usize) -> Self::Vecs {
        Vec::with_capacity(capacity)
    }

    #[inline]
    fn vecs_as_ptrs(_: &Self::Context, vecs: &Self::Vecs) -> Self::Ptrs {
        vecs.as_ptr()
    }

    #[inline]
    fn mut_vecs_as_ptrs(_: &Self::Context, vecs: &mut Self::Vecs) -> Self::MutPtrs {
        vecs.as_mut_ptr()
    }

    #[inline]
    fn vecs_len(_: &Self::Context, vecs: &Self::Vecs) -> usize {
        vecs.len()
    }

    #[inline]
    unsafe fn vecs_set_len(_: &Self::Context, vecs: &mut Self::Vecs, len: usize) {
        unsafe {
            vecs.set_len(len);
        }
    }

    type Refs<'a>
        = &'a Self
    where
        Self: 'a;

    type RefsMut<'a>
        = &'a mut Self
    where
        Self: 'a;

    #[inline]
    unsafe fn ptrs_to_refs<'a>(_: &Self::Context, ptrs: Self::Ptrs) -> Self::Refs<'a> {
        unsafe { &*ptrs }
    }

    #[inline]
    unsafe fn ptrs_to_refs_mut<'a>(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::RefsMut<'a> {
        unsafe { &mut *ptrs }
    }

    #[inline]
    fn refs_as_ptrs(_: &Self::Context, refs: Self::Refs<'_>) -> Self::Ptrs {
        ptr::from_ref(refs)
    }

    #[inline]
    fn mut_refs_as_ptrs(_: &Self::Context, refs: Self::RefsMut<'_>) -> Self::MutPtrs {
        ptr::from_mut(refs)
    }

    #[inline]
    fn mut_refs_as_refs<'a>(_: &Self::Context, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        &*refs
    }

    type SlicePtrs = *const [Self];
    type SliceMutPtrs = *mut [Self];

    #[inline]
    fn slices_from_raw_parts(_: &Self::Context, ptrs: Self::Ptrs, len: usize) -> Self::SlicePtrs {
        ptr::slice_from_raw_parts(ptrs, len)
    }

    #[inline]
    fn slices_from_raw_parts_mut(
        _: &Self::Context,
        ptrs: Self::MutPtrs,
        len: usize,
    ) -> Self::SliceMutPtrs {
        ptr::slice_from_raw_parts_mut(ptrs, len)
    }

    #[inline]
    fn slice_ptrs_cast_const(_: &Self::Context, slices: Self::SliceMutPtrs) -> Self::SlicePtrs {
        slices.cast_const()
    }

    #[inline]
    fn slice_ptrs_cast_mut(_: &Self::Context, slices: Self::SlicePtrs) -> Self::SliceMutPtrs {
        slices.cast_mut()
    }

    #[inline]
    fn slice_ptrs_len(_: &Self::Context, slices: Self::SlicePtrs) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_ptrs_len_mut(_: &Self::Context, slices: Self::SliceMutPtrs) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_ptrs_as_ptrs(_: &Self::Context, slices: Self::SlicePtrs) -> Self::Ptrs {
        slices.cast() // should be `slices.as_ptr()` but it's unstable
    }

    #[inline]
    fn mut_slice_ptrs_as_ptrs(_: &Self::Context, slices: Self::SliceMutPtrs) -> Self::MutPtrs {
        slices.cast() // should be `slices.as_mut_ptr()` but it's unstable
    }

    type Slices<'a>
        = &'a [Self]
    where
        Self: 'a;

    type SlicesMut<'a>
        = &'a mut [Self]
    where
        Self: 'a;

    #[inline]
    unsafe fn slice_ptrs_to_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicePtrs,
    ) -> Self::Slices<'a> {
        let data = Self::slice_ptrs_as_ptrs(context, slices);
        let len = Self::slice_ptrs_len(context, slices);
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices_mut<'a>(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicesMut<'a> {
        let data = Self::mut_slice_ptrs_as_ptrs(context, slices);
        let len = Self::slice_ptrs_len_mut(context, slices);
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    #[inline]
    fn slices_len(_: &Self::Context, slices: &Self::Slices<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slices_len_mut(_: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_refs_as_slice_ptrs(_: &Self::Context, slices: Self::Slices<'_>) -> Self::SlicePtrs {
        ptr::from_ref(slices)
    }

    #[inline]
    fn mut_slice_refs_as_slice_ptrs(
        _: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::SliceMutPtrs {
        ptr::from_mut(slices)
    }

    #[inline]
    fn mut_slices_as_slices<'a>(
        _: &Self::Context,
        slices: Self::SlicesMut<'a>,
    ) -> Self::Slices<'a> {
        &*slices
    }

    #[inline]
    fn slice_refs_as_ptrs(_: &Self::Context, slices: Self::Slices<'_>) -> Self::Ptrs {
        slices.as_ptr()
    }

    #[inline]
    fn mut_slice_refs_as_ptrs(_: &Self::Context, slices: Self::SlicesMut<'_>) -> Self::MutPtrs {
        slices.as_mut_ptr()
    }

    #[inline]
    unsafe fn slices_drop_in_place(_: &Self::Context, slices: Self::SliceMutPtrs) {
        unsafe { ptr::drop_in_place(slices) }
    }
}

impl<'a> SoaToOwned<'a> for &'a () {
    type Owned = ();

    #[inline]
    fn to_owned(&self) -> Self::Owned {}

    #[inline]
    fn clone_into(&self, _: &mut Self::Owned) {}

    #[inline]
    unsafe fn clone_into_ptrs(
        &self,
        _: &<Self::Owned as Soa>::Context,
        _: <Self::Owned as Soa>::MutPtrs,
    ) {
    }

    #[inline]
    fn clone_into_refs(
        &self,
        _: &<Self::Owned as Soa>::Context,
        _: <Self::Owned as Soa>::RefsMut<'_>,
    ) {
    }
}

#[inline]
#[track_caller]
fn collect_array<T, const N: usize>(iter: impl IntoIterator<Item = T>) -> [T; N] {
    #[cold]
    #[inline(never)]
    #[track_caller]
    fn collect_fail(actual_len: usize, required_len: usize) -> ! {
        panic!("iterator should have {required_len} items, but got {actual_len}")
    }

    let mut iter = iter.into_iter();
    let array = array::from_fn(|index| {
        let Some(offset) = iter.next() else {
            collect_fail(index, N);
        };
        offset
    });
    match iter.count() {
        0 => array,
        len => collect_fail(len + N, N),
    }
}

// https://veykril.github.io/tlborm/decl-macros/building-blocks/counting.html#enum-counting
#[macro_export]
#[doc(hidden)]
macro_rules! count_idents {
    ($($idents:ident),* $(,)*) => {
        {
            #[allow(dead_code, non_camel_case_types)]
            #[repr(usize)]
            enum Idents { $($idents,)* __CountIdentsLast }

            const COUNT: usize = Idents::__CountIdentsLast as usize;
            COUNT
        }
    };
}

#[doc(hidden)]
pub use count_idents;

#[doc(hidden)]
pub struct SoaTupleImplHelper<T>(PhantomData<T>);

#[inline]
const fn permutation<const N: usize>() -> [usize; N] {
    let mut permutation = [0; N];
    let mut i = 0;
    while i < N {
        permutation[i] = i;
        i += 1;
    }
    permutation
}

#[inline]
const fn layout_permutation<const N: usize>(layouts: [Layout; N]) -> [usize; N] {
    let mut permutation = permutation::<N>();
    let mut i = 1;
    while i < N {
        let mut j = i;
        while j > 0 && layouts[permutation[j - 1]].align() > layouts[permutation[j]].align() {
            let tmp = permutation[j - 1];
            permutation[j - 1] = permutation[j];
            permutation[j] = tmp;
            j -= 1;
        }
        i += 1;
    }
    permutation
}

macro_rules! soa_tuple_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        impl<$($types,)*> SoaTupleImplHelper<($($types,)*)> {
            pub const PERMUTATION: [usize; count_idents!($($types,)*)] = {
                let layouts = [$(Layout::new::<$types>(),)*];
                layout_permutation(layouts)
            };
            pub const FIELD_DESCRIPTORS: [FieldDescriptor; count_idents!($($types,)*)] = {
                let permutation = Self::PERMUTATION;
                let descriptors = [$(FieldDescriptor::of::<$types>(),)*];
                [$(descriptors[permutation[$indices]],)*]
            };
        }

        unsafe impl<$($types,)*> Soa for ($($types,)*) {
            type Context = ();
            type Fields = Self;
            type FieldDescriptors<'a> = [FieldDescriptor; count_idents!($($types,)*)];

            #[inline]
            fn field_descriptors(_: &Self::Context) -> Self::FieldDescriptors<'_> {
                SoaTupleImplHelper::<($($types,)*)>::FIELD_DESCRIPTORS
            }

            type FieldOffsets<'a> = [usize; count_idents!($($types,)*)];

            #[inline]
            fn buffer_layout(
                _: &Self::Context,
                capacity: usize,
            ) -> Result<(Layout, Self::FieldOffsets<'_>), LayoutError> {
                let layouts = [$(Layout::array::<$types>(capacity)?,)*];
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let mut layout = Layout::new::<()>();
                let mut offsets: [usize; count_idents!($($types,)*)] = Default::default();
                $((layout, offsets[$indices]) = layout.extend(layouts[permutation[$indices]])?;)*

                Ok((layout, offsets))
            }

            type Ptrs = ($(*const $types,)*);
            type MutPtrs = ($(*mut $types,)*);

            type ErasedPtrs = [*const u8; count_idents!($($types,)*)];
            type ErasedMutPtrs = [*mut u8; count_idents!($($types,)*)];

            #[inline]
            fn ptrs_erase(_: &Self::Context, ptrs: Self::Ptrs) -> Self::ErasedPtrs {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let erased: [*const u8; count_idents!($($types,)*)] = [$(ptrs.$indices.cast(),)*];
                [$(erased[permutation[$indices]],)*]
            }

            #[inline]
            fn ptrs_erase_mut(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::ErasedMutPtrs {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let erased: [*mut u8; count_idents!($($types,)*)] = [$(ptrs.$indices.cast(),)*];
                [$(erased[permutation[$indices]],)*]
            }

            #[inline]
            fn ptrs_restore(_: &Self::Context, ptrs: impl IntoIterator<Item = *const u8>) -> Self::Ptrs {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let ptrs: [*const u8; count_idents!($($types,)*)] = collect_array(ptrs);
                ($(ptrs[permutation[$indices]].cast(),)*)
            }

            #[inline]
            fn ptrs_restore_mut(_: &Self::Context, ptrs: impl IntoIterator<Item = *mut u8>) -> Self::MutPtrs {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let ptrs: [*mut u8; count_idents!($($types,)*)] = collect_array(ptrs);
                ($(ptrs[permutation[$indices]].cast(),)*)
            }

            #[inline]
            fn ptrs_dangling(_: &Self::Context) -> Self::MutPtrs {
                ($(ptr::dangling_mut::<$types>(),)*)
            }

            #[inline]
            fn ptrs_cast_const(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::Ptrs {
                ($(ptrs.$indices.cast_const(),)*)
            }

            #[inline]
            fn ptrs_cast_mut(_: &Self::Context, ptrs: Self::Ptrs) -> Self::MutPtrs {
                ($(ptrs.$indices.cast_mut(),)*)
            }

            #[inline]
            unsafe fn ptrs_add(
                _: &Self::Context,
                ptrs: Self::Ptrs,
                offset: usize,
            ) -> Self::Ptrs {
                unsafe { ($(ptrs.$indices.add(offset),)*) }
            }

            #[inline]
            unsafe fn ptrs_add_mut(
                _: &Self::Context,
                ptrs: Self::MutPtrs,
                offset: usize,
            ) -> Self::MutPtrs {
                unsafe { ($(ptrs.$indices.add(offset),)*) }
            }

            #[inline]
            unsafe fn ptrs_offset_from(
                _: &Self::Context,
                ptrs: Self::Ptrs,
                origin: Self::Ptrs,
            ) -> isize {
                let offsets = unsafe { [$(ptrs.$indices.offset_from(origin.$indices),)*] };
                assert!(offsets.iter().all(|offset| offsets[0].eq(offset)));
                offsets[0]
            }

            #[inline]
            unsafe fn ptrs_offset_from_mut(
                _: &Self::Context,
                ptrs: Self::MutPtrs,
                origin: Self::Ptrs,
            ) -> isize {
                let offsets = unsafe { [$(ptrs.$indices.offset_from(origin.$indices),)*] };
                assert!(offsets.iter().all(|offset| offsets[0].eq(offset)));
                offsets[0]
            }

            #[inline]
            unsafe fn ptrs_swap(
                _: &Self::Context,
                a: Self::MutPtrs,
                b: Self::MutPtrs,
            ) {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let closures = ($(|| unsafe { ptr::swap(a.$indices, b.$indices) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline]
            unsafe fn ptrs_copy(
                _: &Self::Context,
                src: Self::Ptrs,
                dst: Self::MutPtrs,
                len: usize,
            ) {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let closures = ($(|| unsafe { ptr::copy(src.$indices, dst.$indices, len) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline]
            unsafe fn ptrs_copy_rev(
                _: &Self::Context,
                src: Self::Ptrs,
                dst: Self::MutPtrs,
                len: usize,
            ) {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let closures = ($(|| unsafe { ptr::copy(src.$indices, dst.$indices, len) },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in (0..count_idents!($($types,)*)).rev() {
                    closures[permutation[index]]();
                }
            }

            #[inline]
            unsafe fn ptrs_copy_nonoverlapping(
                _: &Self::Context,
                src: Self::Ptrs,
                dst: Self::MutPtrs,
                len: usize,
            ) {
                // because source and destination are non-overlapping, we can copy them in any order
                unsafe { $(ptr::copy_nonoverlapping(src.$indices, dst.$indices, len);)* }
            }

            #[inline]
            unsafe fn ptrs_read(_: &Self::Context, ptrs: Self::Ptrs) -> Self {
                unsafe { ($(ptr::read(ptrs.$indices),)*) }
            }

            #[inline]
            unsafe fn ptrs_write(_: &Self::Context, dst: Self::MutPtrs, value: Self) {
                unsafe { $(ptr::write(dst.$indices, value.$indices);)* }
            }

            #[inline]
            unsafe fn ptrs_drop_in_place(_: &Self::Context, ptrs: Self::MutPtrs) {
                unsafe { $(ptr::drop_in_place(ptrs.$indices);)* }
            }

            type NonNullPtrs = ($(NonNull<$types>,)*);

            #[inline]
            unsafe fn ptrs_to_nonnull(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
                unsafe { ($(NonNull::new_unchecked(ptrs.$indices),)*) }
            }

            #[inline]
            fn nonnull_to_ptrs(_: &Self::Context, ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
                ($(ptrs.$indices.as_ptr(),)*)
            }

            type Vecs = ($(Vec<$types>,)*);

            #[inline]
            fn vecs_with_capacity(_: &Self::Context, capacity: usize) -> Self::Vecs {
                ($(Vec::<$types>::with_capacity(capacity),)*)
            }

            #[inline]
            fn vecs_as_ptrs(_: &Self::Context, vecs: &Self::Vecs) -> Self::Ptrs {
                ($(vecs.$indices.as_ptr(),)*)
            }

            #[inline]
            fn mut_vecs_as_ptrs(_: &Self::Context, vecs: &mut Self::Vecs) -> Self::MutPtrs {
                ($(vecs.$indices.as_mut_ptr(),)*)
            }

            #[inline]
            fn vecs_len(_: &Self::Context, vecs: &Self::Vecs) -> usize {
                let lens = [$(vecs.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            unsafe fn vecs_set_len(_: &Self::Context, vecs: &mut Self::Vecs, len: usize) {
                unsafe { $(vecs.$indices.set_len(len);)* }
            }

            type Refs<'a>
                = ($(&'a $types,)*)
            where
                Self: 'a;

            type RefsMut<'a>
                = ($(&'a mut $types,)*)
            where
                Self: 'a;

            #[inline]
            unsafe fn ptrs_to_refs<'a>(_: &Self::Context, ptrs: Self::Ptrs) -> Self::Refs<'a> {
                unsafe { ($(&*ptrs.$indices,)*) }
            }

            #[inline]
            unsafe fn ptrs_to_refs_mut<'a>(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::RefsMut<'a> {
                unsafe { ($(&mut *ptrs.$indices,)*) }
            }

            #[inline]
            fn refs_as_ptrs(_: &Self::Context, refs: Self::Refs<'_>) -> Self::Ptrs {
                ($(ptr::from_ref(refs.$indices),)*)
            }

            #[inline]
            fn mut_refs_as_ptrs(_: &Self::Context, refs: Self::RefsMut<'_>) -> Self::MutPtrs {
                ($(ptr::from_mut(refs.$indices),)*)
            }

            #[inline]
            fn mut_refs_as_refs<'a>(_: &Self::Context, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
                ($(&*refs.$indices,)*)
            }

            #[inline]
            fn slices_from_raw_parts(
                _: &Self::Context,
                ptrs: Self::Ptrs,
                len: usize,
            ) -> Self::SlicePtrs {
                ($(ptr::slice_from_raw_parts(ptrs.$indices, len),)*)
            }

            type SlicePtrs = ($(*const [$types],)*);
            type SliceMutPtrs = ($(*mut [$types],)*);

            #[inline]
            fn slices_from_raw_parts_mut(
                _: &Self::Context,
                ptrs: Self::MutPtrs,
                len: usize,
            ) -> Self::SliceMutPtrs {
                ($(ptr::slice_from_raw_parts_mut(ptrs.$indices, len),)*)
            }

            #[inline]
            fn slice_ptrs_cast_const(
                _: &Self::Context,
                slices: Self::SliceMutPtrs,
            ) -> Self::SlicePtrs {
                ($(slices.$indices.cast_const(),)*)
            }

            #[inline]
            fn slice_ptrs_cast_mut(
                _: &Self::Context,
                slices: Self::SlicePtrs,
            ) -> Self::SliceMutPtrs {
                ($(slices.$indices.cast_mut(),)*)
            }

            #[inline]
            fn slice_ptrs_len(_: &Self::Context, slices: Self::SlicePtrs) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn slice_ptrs_len_mut(_: &Self::Context, slices: Self::SliceMutPtrs) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn slice_ptrs_as_ptrs(_: &Self::Context, slices: Self::SlicePtrs) -> Self::Ptrs {
                ($(slices.$indices.cast(),)*) // should be `slices.$indices.as_ptr()` but it's unstable
            }

            #[inline]
            fn mut_slice_ptrs_as_ptrs(_: &Self::Context, slices: Self::SliceMutPtrs) -> Self::MutPtrs {
                ($(slices.$indices.cast(),)*) // should be `slices.$indices.as_mut_ptr()` but it's unstable
            }

            type Slices<'a>
                = ($(&'a [$types],)*)
            where
                Self: 'a;

            type SlicesMut<'a>
                = ($(&'a mut [$types],)*)
            where
                Self: 'a;

            #[inline]
            unsafe fn slice_ptrs_to_slices<'a>(
                context: &Self::Context,
                slices: Self::SlicePtrs,
            ) -> Self::Slices<'a> {
                let data = Self::slice_ptrs_as_ptrs(context, slices);
                let len = Self::slice_ptrs_len(context, slices);
                unsafe { ($(slice::from_raw_parts(data.$indices, len),)*) }
            }

            #[inline]
            unsafe fn slice_ptrs_to_slices_mut<'a>(
                context: &Self::Context,
                slices: Self::SliceMutPtrs,
            ) -> Self::SlicesMut<'a> {
                let data = Self::mut_slice_ptrs_as_ptrs(context, slices);
                let len = Self::slice_ptrs_len_mut(context, slices);
                unsafe { ($(slice::from_raw_parts_mut(data.$indices, len),)*) }
            }

            #[inline]
            fn slices_len(_: &Self::Context, slices: &Self::Slices<'_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn slices_len_mut(_: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline]
            fn slice_refs_as_slice_ptrs(_: &Self::Context, slices: Self::Slices<'_>) -> Self::SlicePtrs {
                ($(ptr::from_ref(slices.$indices),)*)
            }

            #[inline]
            fn mut_slice_refs_as_slice_ptrs(_: &Self::Context, slices: Self::SlicesMut<'_>) -> Self::SliceMutPtrs {
                ($(ptr::from_mut(slices.$indices),)*)
            }

            #[inline]
            fn mut_slices_as_slices<'a>(_: &Self::Context, slices: Self::SlicesMut<'a>) -> Self::Slices<'a> {
                ($(&*slices.$indices,)*)
            }

            #[inline]
            fn slice_refs_as_ptrs(_: &Self::Context, slices: Self::Slices<'_>) -> Self::Ptrs {
                ($(slices.$indices.as_ptr(),)*)
            }

            #[inline]
            fn mut_slice_refs_as_ptrs(_: &Self::Context, slices: Self::SlicesMut<'_>) -> Self::MutPtrs {
                ($(slices.$indices.as_mut_ptr(),)*)
            }

            #[inline]
            unsafe fn slices_drop_in_place(_: &Self::Context, slices: Self::SliceMutPtrs) {
                unsafe { $(ptr::drop_in_place(slices.$indices);)* }
            }
        }

        impl<'a, $($types,)*> SoaToOwned<'a> for ($(&'a $types,)*)
        where
            $($types: Clone,)*
        {
            type Owned = ($($types,)*);

            #[inline]
            fn to_owned(&self) -> Self::Owned {
                ($(self.$indices.clone(),)*)
            }

            #[inline]
            fn clone_into(&self, target: &mut Self::Owned) {
                $(target.$indices.clone_from(self.$indices);)*
            }

            #[inline]
            fn clone_into_refs(
                &self,
                _: &<Self::Owned as Soa>::Context,
                target: <Self::Owned as Soa>::RefsMut<'_>,
            ) {
                $(target.$indices.clone_from(self.$indices);)*
            }
        }
    };
}

soa_tuple_impl!(
    A index 0,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
    J index 9,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
    J index 9,
    K index 10,
);

soa_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
    J index 9,
    K index 10,
    L index 11,
);
