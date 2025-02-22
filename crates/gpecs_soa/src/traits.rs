use alloc::vec::Vec;
use core::{
    alloc::{Layout, LayoutError},
    borrow::{Borrow, BorrowMut},
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::{self, NonNull},
    slice,
};

use crate::ptr::BufferData;

pub trait SoaIter {
    type Output<'a>: Iterator
    where
        Self: 'a;

    fn iter(&self) -> Self::Output<'_>;
}

impl<T> SoaIter for T
where
    for<'a> &'a T: IntoIterator,
{
    type Output<'a>
        = <&'a T as IntoIterator>::IntoIter
    where
        Self: 'a;

    fn iter(&self) -> Self::Output<'_> {
        self.into_iter()
    }
}

pub trait SoaIterMut {
    type Output<'a>: Iterator
    where
        Self: 'a;

    fn iter_mut(&mut self) -> Self::Output<'_>;
}

impl<T> SoaIterMut for T
where
    for<'a> &'a mut T: IntoIterator,
{
    type Output<'a>
        = <&'a mut T as IntoIterator>::IntoIter
    where
        Self: 'a;

    fn iter_mut(&mut self) -> Self::Output<'_> {
        self.into_iter()
    }
}

pub trait SoaIndex<Idx>
where
    Idx: ?Sized,
{
    type Ref<'a>
    where
        Self: 'a;

    #[track_caller]
    fn index(&self, index: Idx) -> Self::Ref<'_>;
}

impl<T, Idx> SoaIndex<Idx> for T
where
    T: Index<Idx>,
    T::Output: 'static,
{
    type Ref<'a>
        = &'a T::Output
    where
        Self: 'a;

    fn index(&self, index: Idx) -> Self::Ref<'_> {
        Index::index(self, index)
    }
}

pub trait SoaIndexMut<Idx>
where
    Idx: ?Sized,
{
    type RefMut<'a>
    where
        Self: 'a;

    #[track_caller]
    fn index_mut(&mut self, index: Idx) -> Self::RefMut<'_>;
}

impl<T, Idx> SoaIndexMut<Idx> for T
where
    T: IndexMut<Idx>,
    T::Output: 'static,
{
    type RefMut<'a>
        = &'a mut T::Output
    where
        Self: 'a;

    fn index_mut(&mut self, index: Idx) -> Self::RefMut<'_> {
        IndexMut::index_mut(self, index)
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe trait Soa: Sized {
    /// Array of layouts for each field.
    ///
    /// Safety requirements:
    /// - sum of layouts' sizes should be less or equal to the size of self
    /// - alignment of each layout should be less or equal to the alignment of self
    type FieldLayouts: for<'a> SoaIndex<usize, Ref<'a>: Borrow<Layout>>
        + for<'a> SoaIter<Output<'a>: ExactSizeIterator<Item: Borrow<Layout>>>;

    type FieldPermutation: for<'a> SoaIter<Output<'a>: ExactSizeIterator<Item: Borrow<usize>>>;

    fn field_layouts() -> Self::FieldLayouts;
    fn field_permutation() -> Self::FieldPermutation;

    type BufferOffsets: Default
        + for<'a> SoaIndex<usize, Ref<'a>: Borrow<usize>>
        + for<'a> SoaIndexMut<usize, RefMut<'a>: BorrowMut<usize>>
        + for<'a> SoaIter<Output<'a>: ExactSizeIterator<Item: Borrow<usize>>>
        + for<'a> SoaIterMut<Output<'a>: ExactSizeIterator<Item: BorrowMut<usize>>>;

    fn buffer_layout(capacity: usize) -> Result<(Layout, Self::BufferOffsets), LayoutError> {
        let layouts = Self::field_layouts();
        let permutation = Self::field_permutation();
        assert_eq!(permutation.iter().len(), layouts.iter().len());

        let mut offsets = Self::BufferOffsets::default();
        assert_eq!(offsets.iter().len(), permutation.iter().len());

        let mut layout = Layout::new::<()>();
        for item in permutation.iter() {
            let &index: &usize = item.borrow();
            let repeated = repeat_layout(layouts.index(index).borrow(), capacity)?;
            (layout, *offsets.index_mut(index).borrow_mut()) = layout.extend(repeated)?;
        }

        Ok((layout, offsets))
    }

    fn capacity_from(buffer_layout: Layout) -> usize {
        let packed_size = Self::field_layouts()
            .iter()
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
            let (layout, _) = Self::buffer_layout(capacity)
                .expect("new buffer layout should be smaller than the input one");
            layout.size() > buffer_layout.size()
        } {
            capacity -= 1;
        }
        capacity
    }

    type Ptrs: Copy;
    type MutPtrs: Copy;

    fn ptrs_dangling() -> Self::MutPtrs;
    unsafe fn ptrs(ptr: *mut BufferData<Self>, offsets: &Self::BufferOffsets) -> Self::MutPtrs;

    fn ptrs_cast_const(ptrs: Self::MutPtrs) -> Self::Ptrs;
    fn ptrs_cast_mut(ptrs: Self::Ptrs) -> Self::MutPtrs;

    unsafe fn ptrs_add(ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs;
    unsafe fn ptrs_add_mut(ptrs: Self::MutPtrs, offset: usize) -> Self::MutPtrs;
    unsafe fn ptrs_offset_from(ptrs: Self::Ptrs, origin: Self::Ptrs) -> isize;
    unsafe fn ptrs_offset_from_mut(ptrs: Self::MutPtrs, origin: Self::Ptrs) -> isize;
    unsafe fn ptrs_swap(a: Self::MutPtrs, b: Self::MutPtrs);
    unsafe fn ptrs_copy(src: Self::Ptrs, dst: Self::MutPtrs, len: usize);
    unsafe fn ptrs_copy_rev(src: Self::Ptrs, dst: Self::MutPtrs, len: usize);
    unsafe fn ptrs_copy_nonoverlapping(src: Self::Ptrs, dst: Self::MutPtrs, len: usize);
    unsafe fn ptrs_read(src: Self::Ptrs) -> Self;
    unsafe fn ptrs_write(dst: Self::MutPtrs, value: Self);
    unsafe fn ptrs_drop_in_place(ptrs: Self::MutPtrs);

    type NonNullPtrs: Copy;

    unsafe fn ptrs_to_nonnull(ptrs: Self::MutPtrs) -> Self::NonNullPtrs;
    fn nonnull_to_ptrs(ptrs: Self::NonNullPtrs) -> Self::MutPtrs;

    type Vecs;

    fn vecs_with_capacity(capacity: usize) -> Self::Vecs;
    fn vecs_as_ptrs(vecs: &Self::Vecs) -> Self::Ptrs;
    fn mut_vecs_as_ptrs(vecs: &mut Self::Vecs) -> Self::MutPtrs;
    fn vecs_len(vecs: &Self::Vecs) -> usize;
    unsafe fn vecs_set_len(vecs: &mut Self::Vecs, len: usize);

    type Refs<'a>
    where
        Self: 'a;

    type RefsMut<'a>
    where
        Self: 'a;

    unsafe fn as_refs<'a>(ptrs: Self::Ptrs) -> Self::Refs<'a>;
    unsafe fn as_mut_refs<'a>(ptrs: Self::MutPtrs) -> Self::RefsMut<'a>;

    fn refs_as_ptrs(refs: Self::Refs<'_>) -> Self::Ptrs;
    fn mut_refs_as_ptrs(refs: Self::RefsMut<'_>) -> Self::MutPtrs;
    fn mut_refs_as_refs(refs: Self::RefsMut<'_>) -> Self::Refs<'_>;

    type SlicePtrs: Copy;
    type SliceMutPtrs: Copy;

    fn slices_from_raw_parts(ptrs: Self::Ptrs, len: usize) -> Self::SlicePtrs;
    fn slices_from_raw_parts_mut(ptrs: Self::MutPtrs, len: usize) -> Self::SliceMutPtrs;

    type Slices<'a>
    where
        Self: 'a;

    type SlicesMut<'a>
    where
        Self: 'a;

    unsafe fn slices_as_refs<'a>(slices: Self::SlicePtrs) -> Self::Slices<'a>;
    unsafe fn mut_slices_as_refs<'a>(slices: Self::SliceMutPtrs) -> Self::SlicesMut<'a>;

    fn slice_refs_as_slice_ptrs(slices: Self::Slices<'_>) -> Self::SlicePtrs;
    fn mut_slice_refs_as_slice_ptrs(slices: Self::SlicesMut<'_>) -> Self::SliceMutPtrs;

    fn slice_refs_as_ptrs(slices: Self::Slices<'_>) -> Self::Ptrs;
    fn mut_slice_refs_as_ptrs(slices: Self::SlicesMut<'_>) -> Self::MutPtrs;

    unsafe fn slices_drop_in_place(slices: Self::SliceMutPtrs);
}

pub trait SoaToOwned<'a> {
    type Owned: Soa<Refs<'a> = Self>
    where
        Self: 'a;

    fn to_owned(&self) -> Self::Owned;

    fn clone_into(&self, target: &mut Self::Owned) {
        *target = self.to_owned();
    }

    unsafe fn clone_into_ptrs(&self, target: <Self::Owned as Soa>::MutPtrs) {
        let owned = self.to_owned();
        unsafe {
            <Self::Owned as Soa>::ptrs_write(target, owned);
        }
    }

    fn clone_into_refs(&self, target: <Self::Owned as Soa>::RefsMut<'_>) {
        let target = <Self::Owned as Soa>::mut_refs_as_ptrs(target);
        unsafe {
            self.clone_into_ptrs(target);
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
    type FieldLayouts = [Layout; 1];
    type FieldPermutation = [usize; 1];

    #[inline(always)]
    fn field_layouts() -> Self::FieldLayouts {
        [Layout::new::<Self>()]
    }
    #[inline(always)]
    fn field_permutation() -> Self::FieldPermutation {
        [0]
    }

    type BufferOffsets = [usize; 1];

    #[inline(always)]
    fn buffer_layout(_: usize) -> Result<(Layout, Self::BufferOffsets), LayoutError> {
        Ok((Layout::new::<Self>(), [0]))
    }

    #[inline(always)]
    fn capacity_from(_: Layout) -> usize {
        usize::MAX
    }

    type Ptrs = *const Self;
    type MutPtrs = *mut Self;

    #[inline(always)]
    fn ptrs_dangling() -> Self::MutPtrs {
        ptr::dangling_mut()
    }

    #[inline(always)]
    unsafe fn ptrs(ptr: *mut BufferData<Self>, offsets: &Self::BufferOffsets) -> Self::MutPtrs {
        let ptr = ptr.cast::<u8>();
        unsafe { ptr.add(offsets[0]).cast() }
    }

    #[inline(always)]
    fn ptrs_cast_const(ptrs: Self::MutPtrs) -> Self::Ptrs {
        ptrs.cast_const()
    }

    #[inline(always)]
    fn ptrs_cast_mut(ptrs: Self::Ptrs) -> Self::MutPtrs {
        ptrs.cast_mut()
    }

    #[inline(always)]
    unsafe fn ptrs_add(ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs {
        unsafe { ptrs.add(offset) }
    }

    #[inline(always)]
    unsafe fn ptrs_add_mut(ptrs: Self::MutPtrs, offset: usize) -> Self::MutPtrs {
        unsafe { ptrs.add(offset) }
    }

    #[inline(always)]
    unsafe fn ptrs_offset_from(ptrs: Self::Ptrs, origin: Self::Ptrs) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    #[inline(always)]
    unsafe fn ptrs_offset_from_mut(ptrs: Self::MutPtrs, origin: Self::Ptrs) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    #[inline(always)]
    unsafe fn ptrs_swap(a: Self::MutPtrs, b: Self::MutPtrs) {
        unsafe { ptr::swap(a, b) }
    }

    #[inline(always)]
    unsafe fn ptrs_copy(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline(always)]
    unsafe fn ptrs_copy_rev(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline(always)]
    unsafe fn ptrs_copy_nonoverlapping(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        unsafe { ptr::copy_nonoverlapping(src, dst, len) }
    }

    #[inline(always)]
    unsafe fn ptrs_read(ptrs: Self::Ptrs) -> Self {
        unsafe { ptr::read(ptrs) }
    }

    #[inline(always)]
    unsafe fn ptrs_write(ptrs: Self::MutPtrs, value: Self) {
        unsafe { ptr::write(ptrs, value) }
    }

    #[inline(always)]
    unsafe fn ptrs_drop_in_place(ptrs: Self::MutPtrs) {
        unsafe { ptr::drop_in_place(ptrs) }
    }

    type NonNullPtrs = NonNull<Self>;

    #[inline(always)]
    unsafe fn ptrs_to_nonnull(ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
        unsafe { NonNull::new_unchecked(ptrs) }
    }

    #[inline(always)]
    fn nonnull_to_ptrs(ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
        ptrs.as_ptr()
    }

    type Vecs = Vec<Self>;

    #[inline(always)]
    fn vecs_with_capacity(capacity: usize) -> Self::Vecs {
        Vec::with_capacity(capacity)
    }

    #[inline(always)]
    fn vecs_as_ptrs(vecs: &Self::Vecs) -> Self::Ptrs {
        vecs.as_ptr()
    }

    #[inline(always)]
    fn mut_vecs_as_ptrs(vecs: &mut Self::Vecs) -> Self::MutPtrs {
        vecs.as_mut_ptr()
    }

    #[inline(always)]
    fn vecs_len(vecs: &Self::Vecs) -> usize {
        vecs.len()
    }

    #[inline(always)]
    unsafe fn vecs_set_len(vecs: &mut Self::Vecs, len: usize) {
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
    unsafe fn as_refs<'a>(ptrs: Self::Ptrs) -> Self::Refs<'a> {
        unsafe { &*ptrs }
    }

    #[inline(always)]
    unsafe fn as_mut_refs<'a>(ptrs: Self::MutPtrs) -> Self::RefsMut<'a> {
        unsafe { &mut *ptrs }
    }

    #[inline(always)]
    fn refs_as_ptrs(refs: Self::Refs<'_>) -> Self::Ptrs {
        ptr::from_ref(refs)
    }

    #[inline(always)]
    fn mut_refs_as_ptrs(refs: Self::RefsMut<'_>) -> Self::MutPtrs {
        ptr::from_mut(refs)
    }

    #[inline(always)]
    fn mut_refs_as_refs(refs: Self::RefsMut<'_>) -> Self::Refs<'_> {
        &*refs
    }

    type SlicePtrs = *const [Self];
    type SliceMutPtrs = *mut [Self];

    #[inline(always)]
    fn slices_from_raw_parts(ptrs: Self::Ptrs, len: usize) -> Self::SlicePtrs {
        ptr::slice_from_raw_parts(ptrs, len)
    }

    #[inline(always)]
    fn slices_from_raw_parts_mut(ptrs: Self::MutPtrs, len: usize) -> Self::SliceMutPtrs {
        ptr::slice_from_raw_parts_mut(ptrs, len)
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
    unsafe fn slices_as_refs<'a>(slices: Self::SlicePtrs) -> Self::Slices<'a> {
        unsafe { slice::from_raw_parts(slices.cast(), slices.len()) }
    }

    #[inline(always)]
    unsafe fn mut_slices_as_refs<'a>(slices: Self::SliceMutPtrs) -> Self::SlicesMut<'a> {
        unsafe { slice::from_raw_parts_mut(slices.cast(), slices.len()) }
    }

    #[inline(always)]
    fn slice_refs_as_slice_ptrs(slices: Self::Slices<'_>) -> Self::SlicePtrs {
        ptr::from_ref(slices)
    }

    #[inline(always)]
    fn mut_slice_refs_as_slice_ptrs(slices: Self::SlicesMut<'_>) -> Self::SliceMutPtrs {
        ptr::from_mut(slices)
    }

    #[inline(always)]
    fn slice_refs_as_ptrs(slices: Self::Slices<'_>) -> Self::Ptrs {
        slices.as_ptr()
    }

    #[inline(always)]
    fn mut_slice_refs_as_ptrs(slices: Self::SlicesMut<'_>) -> Self::MutPtrs {
        slices.as_mut_ptr()
    }

    #[inline(always)]
    unsafe fn slices_drop_in_place(slices: Self::SliceMutPtrs) {
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
    unsafe fn clone_into_ptrs(&self, _: <Self::Owned as Soa>::MutPtrs) {}

    #[inline(always)]
    fn clone_into_refs(&self, _: <Self::Owned as Soa>::RefsMut<'_>) {}
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

struct SoaTupleConst<T> {
    _ph: PhantomData<T>,
}

macro_rules! soa_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        impl<$($types,)*> SoaTupleConst<($($types,)*)> {
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
            type FieldLayouts = [Layout; count_idents!($($types,)*)];
            type FieldPermutation = [usize; count_idents!($($types,)*)];

            #[inline(always)]
            fn field_layouts() -> Self::FieldLayouts {
                SoaTupleConst::<($($types,)*)>::LAYOUTS
            }

            #[inline(always)]
            fn field_permutation() -> Self::FieldPermutation {
                SoaTupleConst::<($($types,)*)>::PERMUTATION
            }

            type BufferOffsets = [usize; count_idents!($($types,)*)];

            #[inline(always)]
            fn buffer_layout(capacity: usize) -> Result<(Layout, Self::BufferOffsets), LayoutError> {
                let layouts = [$(Layout::array::<$types>(capacity)?,)*];
                let permutation = Self::field_permutation();
                let mut offsets = Self::BufferOffsets::default();

                let layout = Layout::new::<()>();
                $(
                    let (layout, offset) = layout.extend(layouts[permutation[$indices]])?;
                    offsets[permutation[$indices]] = offset;
                )*

                Ok((layout, offsets))
            }

            type Ptrs = ($(*const $types,)*);
            type MutPtrs = ($(*mut $types,)*);

            #[inline(always)]
            fn ptrs_dangling() -> Self::MutPtrs {
                ($(ptr::dangling_mut::<$types>(),)*)
            }

            #[inline(always)]
            unsafe fn ptrs(ptr: *mut BufferData<Self>, offsets: &Self::BufferOffsets) -> Self::MutPtrs {
                let ptr = ptr.cast::<u8>();
                unsafe { ($(ptr.add(offsets[$indices]).cast(),)*) }
            }

            #[inline(always)]
            fn ptrs_cast_const(ptrs: Self::MutPtrs) -> Self::Ptrs {
                ($(ptrs.$indices.cast_const(),)*)
            }

            #[inline(always)]
            fn ptrs_cast_mut(ptrs: Self::Ptrs) -> Self::MutPtrs {
                ($(ptrs.$indices.cast_mut(),)*)
            }

            #[inline(always)]
            unsafe fn ptrs_add(ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs {
                unsafe { ($(ptrs.$indices.add(offset),)*) }
            }

            #[inline(always)]
            unsafe fn ptrs_add_mut(ptrs: Self::MutPtrs, offset: usize) -> Self::MutPtrs {
                unsafe { ($(ptrs.$indices.add(offset),)*) }
            }

            #[inline(always)]
            unsafe fn ptrs_offset_from(ptrs: Self::Ptrs, origin: Self::Ptrs) -> isize {
                let offsets = unsafe { [$(ptrs.$indices.offset_from(origin.$indices),)*] };
                assert!(offsets.iter().all(|offset| offsets[0].eq(offset)));
                offsets[0]
            }

            #[inline(always)]
            unsafe fn ptrs_offset_from_mut(ptrs: Self::MutPtrs, origin: Self::Ptrs) -> isize {
                let offsets = unsafe { [$(ptrs.$indices.offset_from(origin.$indices),)*] };
                assert!(offsets.iter().all(|offset| offsets[0].eq(offset)));
                offsets[0]
            }

            #[inline(always)]
            unsafe fn ptrs_swap(a: Self::MutPtrs, b: Self::MutPtrs) {
                let permutation = Self::field_permutation();

                let closures = ($(|| unsafe { ptr::swap(a.$indices, b.$indices); },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline(always)]
            unsafe fn ptrs_copy(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
                let permutation = Self::field_permutation();

                let closures = ($(|| unsafe { ptr::copy(src.$indices, dst.$indices, len); },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline(always)]
            unsafe fn ptrs_copy_rev(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
                let permutation = Self::field_permutation();

                let closures = ($(|| unsafe { ptr::copy(src.$indices, dst.$indices, len); },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in (0..count_idents!($($types,)*)).rev() {
                    closures[permutation[index]]();
                }
            }

            #[inline(always)]
            unsafe fn ptrs_copy_nonoverlapping(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
                // because source and destination are non-overlapping, we can copy them in any order
                unsafe { $(ptr::copy_nonoverlapping(src.$indices, dst.$indices, len);)* }
            }

            #[inline(always)]
            unsafe fn ptrs_read(ptrs: Self::Ptrs) -> Self {
                unsafe { ($(ptr::read(ptrs.$indices),)*) }
            }

            #[inline(always)]
            unsafe fn ptrs_write(dst: Self::MutPtrs, value: Self) {
                unsafe { $(ptr::write(dst.$indices, value.$indices);)* }
            }

            #[inline(always)]
            unsafe fn ptrs_drop_in_place(ptrs: Self::MutPtrs) {
                unsafe { $(ptr::drop_in_place(ptrs.$indices);)* }
            }

            type NonNullPtrs = ($(NonNull<$types>,)*);

            #[inline(always)]
            unsafe fn ptrs_to_nonnull(ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
                unsafe { ($(NonNull::new_unchecked(ptrs.$indices),)*) }
            }

            #[inline(always)]
            fn nonnull_to_ptrs(ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
                ($(ptrs.$indices.as_ptr(),)*)
            }

            type Vecs = ($(Vec<$types>,)*);

            #[inline(always)]
            fn vecs_with_capacity(capacity: usize) -> Self::Vecs {
                ($(Vec::<$types>::with_capacity(capacity),)*)
            }

            #[inline(always)]
            fn vecs_as_ptrs(vecs: &Self::Vecs) -> Self::Ptrs {
                ($(vecs.$indices.as_ptr(),)*)
            }

            #[inline(always)]
            fn mut_vecs_as_ptrs(vecs: &mut Self::Vecs) -> Self::MutPtrs {
                ($(vecs.$indices.as_mut_ptr(),)*)
            }

            #[inline(always)]
            fn vecs_len(vecs: &Self::Vecs) -> usize {
                let lens = [$(vecs.$indices.len(),)*];
                assert!(lens.iter().all(|len| lens[0].eq(len)));
                lens[0]
            }

            #[inline(always)]
            unsafe fn vecs_set_len(vecs: &mut Self::Vecs, len: usize) {
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
            unsafe fn as_refs<'a>(ptrs: Self::Ptrs) -> Self::Refs<'a> {
                unsafe { ($(&*ptrs.$indices,)*) }
            }

            #[inline(always)]
            unsafe fn as_mut_refs<'a>(ptrs: Self::MutPtrs) -> Self::RefsMut<'a> {
                unsafe { ($(&mut *ptrs.$indices,)*) }
            }

            #[inline(always)]
            fn refs_as_ptrs(refs: Self::Refs<'_>) -> Self::Ptrs {
                ($(ptr::from_ref(refs.$indices),)*)
            }

            #[inline(always)]
            fn mut_refs_as_ptrs(refs: Self::RefsMut<'_>) -> Self::MutPtrs {
                ($(ptr::from_mut(refs.$indices),)*)
            }

            #[inline(always)]
            fn mut_refs_as_refs(refs: Self::RefsMut<'_>) -> Self::Refs<'_> {
                ($(&*refs.$indices,)*)
            }

            #[inline(always)]
            fn slices_from_raw_parts(ptrs: Self::Ptrs, len: usize) -> Self::SlicePtrs {
                ($(ptr::slice_from_raw_parts(ptrs.$indices, len),)*)
            }

            type SlicePtrs = ($(*const [$types],)*);
            type SliceMutPtrs = ($(*mut [$types],)*);

            #[inline(always)]
            fn slices_from_raw_parts_mut(ptrs: Self::MutPtrs, len: usize) -> Self::SliceMutPtrs {
                ($(ptr::slice_from_raw_parts_mut(ptrs.$indices, len),)*)
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
            unsafe fn slices_as_refs<'a>(slices: Self::SlicePtrs) -> Self::Slices<'a> {
                let len = {
                    let lens = [$(slices.$indices.len(),)*];
                    assert!(lens.iter().all(|len| lens[0].eq(len)));
                    lens[0]
                };
                unsafe { ($(slice::from_raw_parts(slices.$indices.cast(), len),)*) }
            }

            #[inline(always)]
            unsafe fn mut_slices_as_refs<'a>(slices: Self::SliceMutPtrs) -> Self::SlicesMut<'a> {
                let len = {
                    let lens = [$(slices.$indices.len(),)*];
                    assert!(lens.iter().all(|len| lens[0].eq(len)));
                    lens[0]
                };
                unsafe { ($(slice::from_raw_parts_mut(slices.$indices.cast(), len),)*) }
            }

            #[inline(always)]
            fn slice_refs_as_slice_ptrs(slices: Self::Slices<'_>) -> Self::SlicePtrs {
                ($(ptr::from_ref(slices.$indices),)*)
            }

            #[inline(always)]
            fn mut_slice_refs_as_slice_ptrs(slices: Self::SlicesMut<'_>) -> Self::SliceMutPtrs {
                ($(ptr::from_mut(slices.$indices),)*)
            }

            #[inline(always)]
            fn slice_refs_as_ptrs(slices: Self::Slices<'_>) -> Self::Ptrs {
                ($(slices.$indices.as_ptr(),)*)
            }

            #[inline(always)]
            fn mut_slice_refs_as_ptrs(slices: Self::SlicesMut<'_>) -> Self::MutPtrs {
                ($(slices.$indices.as_mut_ptr(),)*)
            }

            #[inline(always)]
            unsafe fn slices_drop_in_place(slices: Self::SliceMutPtrs) {
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
            fn clone_into_refs(&self, target: <Self::Owned as Soa>::RefsMut<'_>) {
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
