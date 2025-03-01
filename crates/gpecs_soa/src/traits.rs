use alloc::vec::Vec;
use core::{
    alloc::{Layout, LayoutError},
    array,
    borrow::Borrow,
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};

#[allow(clippy::missing_safety_doc)]
pub unsafe trait Soa: Sized {
    /// Special type used to properly allocate the buffer in memory.
    ///
    /// Most of the time, this should be the same as `Self`.
    /// This is true for such implementations which store all the fields of self.
    type SizeAlign;

    /// Type of context used to perform all operations of this trait.
    ///
    /// Most of the time, this should be [unit](prim@unit) type.
    /// This is true for all the types with fields' size and alignment known at compile-time.
    type Context;

    /// Collection of layouts for each field.
    ///
    /// Safety requirements:
    /// - sum of layouts' sizes should be less or equal to the size of [`SizeAlign`](`Soa::SizeAlign`)
    /// - alignment of each layout should be less or equal to the alignment of [`SizeAlign`](`Soa::SizeAlign`)
    type FieldLayouts<'a>: IntoIterator<Item: Borrow<Layout>>
    where
        Self::Context: 'a;

    fn field_layouts(context: &Self::Context) -> Self::FieldLayouts<'_>;

    /// Calculates layout needed to store `capacity` number of fields inside of a buffer.
    ///
    /// This layout should not include [`Context`](`Soa::Context`),
    /// as it is handled by the crate itself.
    fn buffer_layout(
        context: &Self::Context,
        capacity: usize,
    ) -> Result<(Layout, impl '_ + IntoIterator<Item = usize>), LayoutError> {
        let mut layout = Layout::new::<()>();
        let offsets = Self::field_layouts(context)
            .into_iter()
            .map(|item| {
                let repeated = repeat_layout(item.borrow(), capacity)?;
                let offset;
                (layout, offset) = layout.extend(repeated)?;
                Ok(offset)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok((layout, offsets))
    }

    /// Retrieves maximum number of fields that can be stored inside of a buffer with given layout.
    fn capacity_from(context: &Self::Context, buffer_layout: Layout) -> usize {
        let packed_size = Self::field_layouts(context)
            .into_iter()
            .map(|item| {
                let layout: &Layout = item.borrow();
                layout.size()
            })
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

    type Ptrs: Copy;
    type MutPtrs: Copy;

    unsafe fn ptrs(
        context: &Self::Context,
        ptr: *mut u8,
        offsets: impl IntoIterator<Item = usize>,
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

    unsafe fn ptrs_drop_in_place(context: &Self::Context, ptrs: Self::MutPtrs);

    type NonNullPtrs: Copy;

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

    type SlicePtrs: Copy;
    type SliceMutPtrs: Copy;

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

    unsafe fn slices_drop_in_place(context: &Self::Context, slices: Self::SliceMutPtrs);
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
    type SizeAlign = Self;
    type Context = ();
    type FieldLayouts<'a> = [Layout; 1];

    #[inline(always)]
    fn field_layouts(_: &Self::Context) -> Self::FieldLayouts<'_> {
        [Layout::new::<Self>()]
    }

    #[inline(always)]
    fn buffer_layout(
        _: &Self::Context,
        _: usize,
    ) -> Result<(Layout, impl IntoIterator<Item = usize>), LayoutError> {
        Ok((Layout::new::<Self>(), [0]))
    }

    #[inline(always)]
    fn capacity_from(_: &Self::Context, _: Layout) -> usize {
        usize::MAX
    }

    type Ptrs = *const Self;
    type MutPtrs = *mut Self;

    #[track_caller]
    #[inline(always)]
    unsafe fn ptrs(
        _: &Self::Context,
        ptr: *mut u8,
        offsets: impl IntoIterator<Item = usize>,
    ) -> Self::MutPtrs {
        let offsets: [usize; 1] = collect_array(offsets);
        unsafe { ptr.add(offsets[0]).cast() }
    }

    #[inline(always)]
    fn ptrs_dangling(_: &Self::Context) -> Self::MutPtrs {
        ptr::dangling_mut()
    }

    #[inline(always)]
    fn ptrs_cast_const(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::Ptrs {
        ptrs.cast_const()
    }

    #[inline(always)]
    fn ptrs_cast_mut(_: &Self::Context, ptrs: Self::Ptrs) -> Self::MutPtrs {
        ptrs.cast_mut()
    }

    #[inline(always)]
    unsafe fn ptrs_add(_: &Self::Context, ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs {
        unsafe { ptrs.add(offset) }
    }

    #[inline(always)]
    unsafe fn ptrs_add_mut(_: &Self::Context, ptrs: Self::MutPtrs, offset: usize) -> Self::MutPtrs {
        unsafe { ptrs.add(offset) }
    }

    #[inline(always)]
    unsafe fn ptrs_offset_from(_: &Self::Context, ptrs: Self::Ptrs, origin: Self::Ptrs) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    #[inline(always)]
    unsafe fn ptrs_offset_from_mut(
        _: &Self::Context,
        ptrs: Self::MutPtrs,
        origin: Self::Ptrs,
    ) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    #[inline(always)]
    unsafe fn ptrs_swap(_: &Self::Context, a: Self::MutPtrs, b: Self::MutPtrs) {
        unsafe { ptr::swap(a, b) }
    }

    #[inline(always)]
    unsafe fn ptrs_copy(_: &Self::Context, src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline(always)]
    unsafe fn ptrs_copy_rev(_: &Self::Context, src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline(always)]
    unsafe fn ptrs_copy_nonoverlapping(
        _: &Self::Context,
        src: Self::Ptrs,
        dst: Self::MutPtrs,
        len: usize,
    ) {
        unsafe { ptr::copy_nonoverlapping(src, dst, len) }
    }

    #[inline(always)]
    unsafe fn ptrs_read(_: &Self::Context, ptrs: Self::Ptrs) -> Self {
        unsafe { ptr::read(ptrs) }
    }

    #[inline(always)]
    unsafe fn ptrs_write(_: &Self::Context, ptrs: Self::MutPtrs, value: Self) {
        unsafe { ptr::write(ptrs, value) }
    }

    #[inline(always)]
    unsafe fn ptrs_drop_in_place(_: &Self::Context, ptrs: Self::MutPtrs) {
        unsafe { ptr::drop_in_place(ptrs) }
    }

    type NonNullPtrs = NonNull<Self>;

    #[inline(always)]
    unsafe fn ptrs_to_nonnull(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
        unsafe { NonNull::new_unchecked(ptrs) }
    }

    #[inline(always)]
    fn nonnull_to_ptrs(_: &Self::Context, ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
        ptrs.as_ptr()
    }

    type Vecs = Vec<Self>;

    #[inline(always)]
    fn vecs_with_capacity(_: &Self::Context, capacity: usize) -> Self::Vecs {
        Vec::with_capacity(capacity)
    }

    #[inline(always)]
    fn vecs_as_ptrs(_: &Self::Context, vecs: &Self::Vecs) -> Self::Ptrs {
        vecs.as_ptr()
    }

    #[inline(always)]
    fn mut_vecs_as_ptrs(_: &Self::Context, vecs: &mut Self::Vecs) -> Self::MutPtrs {
        vecs.as_mut_ptr()
    }

    #[inline(always)]
    fn vecs_len(_: &Self::Context, vecs: &Self::Vecs) -> usize {
        vecs.len()
    }

    #[inline(always)]
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

    #[inline(always)]
    unsafe fn ptrs_to_refs<'a>(_: &Self::Context, ptrs: Self::Ptrs) -> Self::Refs<'a> {
        unsafe { &*ptrs }
    }

    #[inline(always)]
    unsafe fn ptrs_to_refs_mut<'a>(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::RefsMut<'a> {
        unsafe { &mut *ptrs }
    }

    #[inline(always)]
    fn refs_as_ptrs(_: &Self::Context, refs: Self::Refs<'_>) -> Self::Ptrs {
        ptr::from_ref(refs)
    }

    #[inline(always)]
    fn mut_refs_as_ptrs(_: &Self::Context, refs: Self::RefsMut<'_>) -> Self::MutPtrs {
        ptr::from_mut(refs)
    }

    #[inline(always)]
    fn mut_refs_as_refs<'a>(_: &Self::Context, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        &*refs
    }

    type SlicePtrs = *const [Self];
    type SliceMutPtrs = *mut [Self];

    #[inline(always)]
    fn slices_from_raw_parts(_: &Self::Context, ptrs: Self::Ptrs, len: usize) -> Self::SlicePtrs {
        ptr::slice_from_raw_parts(ptrs, len)
    }

    #[inline(always)]
    fn slices_from_raw_parts_mut(
        _: &Self::Context,
        ptrs: Self::MutPtrs,
        len: usize,
    ) -> Self::SliceMutPtrs {
        ptr::slice_from_raw_parts_mut(ptrs, len)
    }

    #[inline(always)]
    fn slice_ptrs_cast_const(_: &Self::Context, slices: Self::SliceMutPtrs) -> Self::SlicePtrs {
        slices.cast_const()
    }

    #[inline(always)]
    fn slice_ptrs_cast_mut(_: &Self::Context, slices: Self::SlicePtrs) -> Self::SliceMutPtrs {
        slices.cast_mut()
    }

    #[inline(always)]
    fn slice_ptrs_len(_: &Self::Context, slices: Self::SlicePtrs) -> usize {
        slices.len()
    }

    #[inline(always)]
    fn slice_ptrs_len_mut(_: &Self::Context, slices: Self::SliceMutPtrs) -> usize {
        slices.len()
    }

    #[inline(always)]
    fn slice_ptrs_as_ptrs(_: &Self::Context, slices: Self::SlicePtrs) -> Self::Ptrs {
        slices.cast() // should be `slices.as_ptr()` but it's unstable
    }

    #[inline(always)]
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

    #[inline(always)]
    unsafe fn slice_ptrs_to_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicePtrs,
    ) -> Self::Slices<'a> {
        let data = Self::slice_ptrs_as_ptrs(context, slices);
        let len = Self::slice_ptrs_len(context, slices);
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline(always)]
    unsafe fn slice_ptrs_to_slices_mut<'a>(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicesMut<'a> {
        let data = Self::mut_slice_ptrs_as_ptrs(context, slices);
        let len = Self::slice_ptrs_len_mut(context, slices);
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    #[inline(always)]
    fn slices_len(_: &Self::Context, slices: &Self::Slices<'_>) -> usize {
        slices.len()
    }

    #[inline(always)]
    fn slices_len_mut(_: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize {
        slices.len()
    }

    #[inline(always)]
    fn slice_refs_as_slice_ptrs(_: &Self::Context, slices: Self::Slices<'_>) -> Self::SlicePtrs {
        ptr::from_ref(slices)
    }

    #[inline(always)]
    fn mut_slice_refs_as_slice_ptrs(
        _: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::SliceMutPtrs {
        ptr::from_mut(slices)
    }

    #[inline(always)]
    fn mut_slices_as_slices<'a>(
        _: &Self::Context,
        slices: Self::SlicesMut<'a>,
    ) -> Self::Slices<'a> {
        &*slices
    }

    #[inline(always)]
    fn slice_refs_as_ptrs(_: &Self::Context, slices: Self::Slices<'_>) -> Self::Ptrs {
        slices.as_ptr()
    }

    #[inline(always)]
    fn mut_slice_refs_as_ptrs(_: &Self::Context, slices: Self::SlicesMut<'_>) -> Self::MutPtrs {
        slices.as_mut_ptr()
    }

    #[inline(always)]
    unsafe fn slices_drop_in_place(_: &Self::Context, slices: Self::SliceMutPtrs) {
        unsafe { ptr::drop_in_place(slices) }
    }
}

impl<'a> SoaToOwned<'a> for &'a () {
    type Owned = ();

    #[inline(always)]
    fn to_owned(&self) -> Self::Owned {}

    #[inline(always)]
    fn clone_into(&self, _: &mut Self::Owned) {}

    #[inline(always)]
    unsafe fn clone_into_ptrs(
        &self,
        _: &<Self::Owned as Soa>::Context,
        _: <Self::Owned as Soa>::MutPtrs,
    ) {
    }

    #[inline(always)]
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

struct SoaTupleHelper<T>(PhantomData<T>);

macro_rules! soa_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        impl<$($types,)*> SoaTupleHelper<($($types,)*)> {
            const LAYOUTS: [Layout; count_idents!($($types,)*)] = [
                $(Layout::new::<$types>(),)*
            ];
            const PERMUTATION: [usize; count_idents!($($types,)*)] = {
                let mut permutation = [$($indices,)*];
                let mut i = 1;
                while i < count_idents!($($types,)*) {
                    let mut j = i;
                    while j > 0 && Self::LAYOUTS[j - 1].align() > Self::LAYOUTS[j].align() {
                        let tmp = permutation[j - 1];
                        permutation[j - 1] = permutation[j];
                        permutation[j] = tmp;

                        j -= 1;
                    }
                    i += 1;
                }
                permutation
            };
        }

        unsafe impl<$($types,)*> Soa for ($($types,)*) {
            type SizeAlign = Self;
            type Context = ();
            type FieldLayouts<'a> = [Layout; count_idents!($($types,)*)];

            #[inline(always)]
            fn field_layouts(_: &Self::Context) -> Self::FieldLayouts<'_> {
                SoaTupleHelper::<($($types,)*)>::LAYOUTS
            }

            #[inline(always)]
            fn buffer_layout(
                _: &Self::Context,
                capacity: usize,
            ) -> Result<(Layout, impl '_ + IntoIterator<Item = usize>), LayoutError> {
                let layouts = [$(Layout::array::<$types>(capacity)?,)*];
                let permutation = SoaTupleHelper::<($($types,)*)>::PERMUTATION;
                let mut offsets: [usize; count_idents!($($types,)*)] = Default::default();

                let layout = Layout::new::<()>();
                $(
                    let (layout, offset) = layout.extend(layouts[permutation[$indices]])?;
                    offsets[permutation[$indices]] = offset;
                )*

                Ok((layout, offsets))
            }

            type Ptrs = ($(*const $types,)*);
            type MutPtrs = ($(*mut $types,)*);

            #[track_caller]
            #[inline(always)]
            unsafe fn ptrs(
                _: &Self::Context,
                ptr: *mut u8,
                offsets: impl IntoIterator<Item = usize>,
            ) -> Self::MutPtrs {
                let offsets: [usize; count_idents!($($types,)*)] = collect_array(offsets);
                unsafe { ($(ptr.add(offsets[$indices]).cast(),)*) }
            }

            #[inline(always)]
            fn ptrs_dangling(_: &Self::Context) -> Self::MutPtrs {
                ($(ptr::dangling_mut::<$types>(),)*)
            }

            #[inline(always)]
            fn ptrs_cast_const(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::Ptrs {
                ($(ptrs.$indices.cast_const(),)*)
            }

            #[inline(always)]
            fn ptrs_cast_mut(_: &Self::Context, ptrs: Self::Ptrs) -> Self::MutPtrs {
                ($(ptrs.$indices.cast_mut(),)*)
            }

            #[inline(always)]
            unsafe fn ptrs_add(
                _: &Self::Context,
                ptrs: Self::Ptrs,
                offset: usize,
            ) -> Self::Ptrs {
                unsafe { ($(ptrs.$indices.add(offset),)*) }
            }

            #[inline(always)]
            unsafe fn ptrs_add_mut(
                _: &Self::Context,
                ptrs: Self::MutPtrs,
                offset: usize,
            ) -> Self::MutPtrs {
                unsafe { ($(ptrs.$indices.add(offset),)*) }
            }

            #[inline(always)]
            unsafe fn ptrs_offset_from(
                _: &Self::Context,
                ptrs: Self::Ptrs,
                origin: Self::Ptrs,
            ) -> isize {
                let offsets = unsafe { [$(ptrs.$indices.offset_from(origin.$indices),)*] };
                assert!(offsets.iter().all(|offset| offsets[0].eq(offset)));
                offsets[0]
            }

            #[inline(always)]
            unsafe fn ptrs_offset_from_mut(
                _: &Self::Context,
                ptrs: Self::MutPtrs,
                origin: Self::Ptrs,
            ) -> isize {
                let offsets = unsafe { [$(ptrs.$indices.offset_from(origin.$indices),)*] };
                assert!(offsets.iter().all(|offset| offsets[0].eq(offset)));
                offsets[0]
            }

            #[inline(always)]
            unsafe fn ptrs_swap(
                _: &Self::Context,
                a: Self::MutPtrs,
                b: Self::MutPtrs,
            ) {
                let permutation = SoaTupleHelper::<($($types,)*)>::PERMUTATION;

                let closures = ($(|| unsafe { ptr::swap(a.$indices, b.$indices); },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline(always)]
            unsafe fn ptrs_copy(
                _: &Self::Context,
                src: Self::Ptrs,
                dst: Self::MutPtrs,
                len: usize,
            ) {
                let permutation = SoaTupleHelper::<($($types,)*)>::PERMUTATION;

                let closures = ($(|| unsafe { ptr::copy(src.$indices, dst.$indices, len); },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline(always)]
            unsafe fn ptrs_copy_rev(
                _: &Self::Context,
                src: Self::Ptrs,
                dst: Self::MutPtrs,
                len: usize,
            ) {
                let permutation = SoaTupleHelper::<($($types,)*)>::PERMUTATION;

                let closures = ($(|| unsafe { ptr::copy(src.$indices, dst.$indices, len); },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in (0..count_idents!($($types,)*)).rev() {
                    closures[permutation[index]]();
                }
            }

            #[inline(always)]
            unsafe fn ptrs_copy_nonoverlapping(
                _: &Self::Context,
                src: Self::Ptrs,
                dst: Self::MutPtrs,
                len: usize,
            ) {
                // because source and destination are non-overlapping, we can copy them in any order
                unsafe { $(ptr::copy_nonoverlapping(src.$indices, dst.$indices, len);)* }
            }

            #[inline(always)]
            unsafe fn ptrs_read(_: &Self::Context, ptrs: Self::Ptrs) -> Self {
                unsafe { ($(ptr::read(ptrs.$indices),)*) }
            }

            #[inline(always)]
            unsafe fn ptrs_write(_: &Self::Context, dst: Self::MutPtrs, value: Self) {
                unsafe { $(ptr::write(dst.$indices, value.$indices);)* }
            }

            #[inline(always)]
            unsafe fn ptrs_drop_in_place(_: &Self::Context, ptrs: Self::MutPtrs) {
                unsafe { $(ptr::drop_in_place(ptrs.$indices);)* }
            }

            type NonNullPtrs = ($(NonNull<$types>,)*);

            #[inline(always)]
            unsafe fn ptrs_to_nonnull(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
                unsafe { ($(NonNull::new_unchecked(ptrs.$indices),)*) }
            }

            #[inline(always)]
            fn nonnull_to_ptrs(_: &Self::Context, ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
                ($(ptrs.$indices.as_ptr(),)*)
            }

            type Vecs = ($(Vec<$types>,)*);

            #[inline(always)]
            fn vecs_with_capacity(_: &Self::Context, capacity: usize) -> Self::Vecs {
                ($(Vec::<$types>::with_capacity(capacity),)*)
            }

            #[inline(always)]
            fn vecs_as_ptrs(_: &Self::Context, vecs: &Self::Vecs) -> Self::Ptrs {
                ($(vecs.$indices.as_ptr(),)*)
            }

            #[inline(always)]
            fn mut_vecs_as_ptrs(_: &Self::Context, vecs: &mut Self::Vecs) -> Self::MutPtrs {
                ($(vecs.$indices.as_mut_ptr(),)*)
            }

            #[inline(always)]
            fn vecs_len(_: &Self::Context, vecs: &Self::Vecs) -> usize {
                let lens = [$(vecs.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline(always)]
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

            #[inline(always)]
            unsafe fn ptrs_to_refs<'a>(_: &Self::Context, ptrs: Self::Ptrs) -> Self::Refs<'a> {
                unsafe { ($(&*ptrs.$indices,)*) }
            }

            #[inline(always)]
            unsafe fn ptrs_to_refs_mut<'a>(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::RefsMut<'a> {
                unsafe { ($(&mut *ptrs.$indices,)*) }
            }

            #[inline(always)]
            fn refs_as_ptrs(_: &Self::Context, refs: Self::Refs<'_>) -> Self::Ptrs {
                ($(ptr::from_ref(refs.$indices),)*)
            }

            #[inline(always)]
            fn mut_refs_as_ptrs(_: &Self::Context, refs: Self::RefsMut<'_>) -> Self::MutPtrs {
                ($(ptr::from_mut(refs.$indices),)*)
            }

            #[inline(always)]
            fn mut_refs_as_refs<'a>(_: &Self::Context, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
                ($(&*refs.$indices,)*)
            }

            #[inline(always)]
            fn slices_from_raw_parts(
                _: &Self::Context,
                ptrs: Self::Ptrs,
                len: usize,
            ) -> Self::SlicePtrs {
                ($(ptr::slice_from_raw_parts(ptrs.$indices, len),)*)
            }

            type SlicePtrs = ($(*const [$types],)*);
            type SliceMutPtrs = ($(*mut [$types],)*);

            #[inline(always)]
            fn slices_from_raw_parts_mut(
                _: &Self::Context,
                ptrs: Self::MutPtrs,
                len: usize,
            ) -> Self::SliceMutPtrs {
                ($(ptr::slice_from_raw_parts_mut(ptrs.$indices, len),)*)
            }

            #[inline(always)]
            fn slice_ptrs_cast_const(
                _: &Self::Context,
                slices: Self::SliceMutPtrs,
            ) -> Self::SlicePtrs {
                ($(slices.$indices.cast_const(),)*)
            }

            #[inline(always)]
            fn slice_ptrs_cast_mut(
                _: &Self::Context,
                slices: Self::SlicePtrs,
            ) -> Self::SliceMutPtrs {
                ($(slices.$indices.cast_mut(),)*)
            }

            #[inline(always)]
            fn slice_ptrs_len(_: &Self::Context, slices: Self::SlicePtrs) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline(always)]
            fn slice_ptrs_len_mut(_: &Self::Context, slices: Self::SliceMutPtrs) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline(always)]
            fn slice_ptrs_as_ptrs(_: &Self::Context, slices: Self::SlicePtrs) -> Self::Ptrs {
                ($(slices.$indices.cast(),)*) // should be `slices.$indices.as_ptr()` but it's unstable
            }

            #[inline(always)]
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

            #[inline(always)]
            unsafe fn slice_ptrs_to_slices<'a>(
                context: &Self::Context,
                slices: Self::SlicePtrs,
            ) -> Self::Slices<'a> {
                let data = Self::slice_ptrs_as_ptrs(context, slices);
                let len = Self::slice_ptrs_len(context, slices);
                unsafe { ($(slice::from_raw_parts(data.$indices, len),)*) }
            }

            #[inline(always)]
            unsafe fn slice_ptrs_to_slices_mut<'a>(
                context: &Self::Context,
                slices: Self::SliceMutPtrs,
            ) -> Self::SlicesMut<'a> {
                let data = Self::mut_slice_ptrs_as_ptrs(context, slices);
                let len = Self::slice_ptrs_len_mut(context, slices);
                unsafe { ($(slice::from_raw_parts_mut(data.$indices, len),)*) }
            }

            #[inline(always)]
            fn slices_len(_: &Self::Context, slices: &Self::Slices<'_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline(always)]
            fn slices_len_mut(_: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize {
                let lens = [$(slices.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline(always)]
            fn slice_refs_as_slice_ptrs(_: &Self::Context, slices: Self::Slices<'_>) -> Self::SlicePtrs {
                ($(ptr::from_ref(slices.$indices),)*)
            }

            #[inline(always)]
            fn mut_slice_refs_as_slice_ptrs(_: &Self::Context, slices: Self::SlicesMut<'_>) -> Self::SliceMutPtrs {
                ($(ptr::from_mut(slices.$indices),)*)
            }

            #[inline(always)]
            fn mut_slices_as_slices<'a>(_: &Self::Context, slices: Self::SlicesMut<'a>) -> Self::Slices<'a> {
                ($(&*slices.$indices,)*)
            }

            #[inline(always)]
            fn slice_refs_as_ptrs(_: &Self::Context, slices: Self::Slices<'_>) -> Self::Ptrs {
                ($(slices.$indices.as_ptr(),)*)
            }

            #[inline(always)]
            fn mut_slice_refs_as_ptrs(_: &Self::Context, slices: Self::SlicesMut<'_>) -> Self::MutPtrs {
                ($(slices.$indices.as_mut_ptr(),)*)
            }

            #[inline(always)]
            unsafe fn slices_drop_in_place(_: &Self::Context, slices: Self::SliceMutPtrs) {
                unsafe { $(ptr::drop_in_place(slices.$indices);)* }
            }
        }

        impl<'a, $($types,)*> SoaToOwned<'a> for ($(&'a $types,)*)
        where
            $($types: Clone,)*
        {
            type Owned = ($($types,)*);

            #[inline(always)]
            fn to_owned(&self) -> Self::Owned {
                ($(self.$indices.clone(),)*)
            }

            #[inline(always)]
            fn clone_into(&self, target: &mut Self::Owned) {
                $(target.$indices.clone_from(self.$indices);)*
            }

            #[inline(always)]
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

soa_impl!(
    A index 0,
);

soa_impl!(
    A index 0,
    B index 1,
);

soa_impl!(
    A index 0,
    B index 1,
    C index 2,
);

soa_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
);

soa_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
);

soa_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
);

soa_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
);

soa_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
);

soa_impl!(
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

soa_impl!(
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

soa_impl!(
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

soa_impl!(
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
