use core::{
    alloc::{Layout, LayoutError},
    marker::PhantomData,
};

use crate::ptr::BufferData;

#[allow(clippy::missing_safety_doc)]
pub unsafe trait Soa: Sized {
    type FieldLayouts: AsRef<[Layout]>;
    type FieldPermutation: AsRef<[usize]>;

    fn field_layouts() -> Self::FieldLayouts;
    fn field_permutation() -> Self::FieldPermutation;

    fn packed_size_of() -> usize {
        let layouts = Self::field_layouts();
        layouts.as_ref().iter().map(Layout::size).sum()
    }

    type BufferOffsets: Default + AsRef<[usize]> + AsMut<[usize]>;

    fn buffer_layout(capacity: usize) -> Result<(Layout, Self::BufferOffsets), LayoutError> {
        let layouts = Self::field_layouts();
        let permutation = Self::field_permutation();

        let layouts = layouts.as_ref();
        let permutation = permutation.as_ref();
        assert_eq!(permutation.len(), layouts.len());

        let mut offsets = Self::BufferOffsets::default();
        let offsets_mut = offsets.as_mut();
        assert_eq!(offsets_mut.len(), permutation.len());

        let mut layout = Layout::new::<()>();
        for &index in permutation {
            let repeated = repeat_layout(layouts[index], capacity)?;
            (layout, offsets_mut[index]) = layout.extend(repeated)?;
        }

        Ok((layout, offsets))
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

    fn clone_into_refs(&self, target: <Self::Owned as Soa>::RefsMut<'_>) {
        let owned = self.to_owned();
        unsafe {
            let dst = <Self::Owned as Soa>::mut_refs_as_ptrs(target);
            <Self::Owned as Soa>::ptrs_write(dst, owned);
        }
    }
}

/// Use this until [`Layout::repeat()`] is stabilized
fn repeat_layout(layout: Layout, n: usize) -> Result<Layout, LayoutError> {
    const ERR: LayoutError = match Layout::from_size_align(usize::MAX, 1) {
        Ok(_) => unreachable!(),
        Err(err) => err,
    };

    let layout = layout.pad_to_align();
    let size = layout.size().checked_mul(n).ok_or(ERR)?;
    Layout::from_size_align(size, layout.align())
}

unsafe impl Soa for () {
    type FieldLayouts = [Layout; 0];
    type FieldPermutation = [usize; 0];

    #[inline(always)]
    fn field_layouts() -> Self::FieldLayouts {
        []
    }
    #[inline(always)]
    fn field_permutation() -> Self::FieldPermutation {
        []
    }

    #[inline(always)]
    fn packed_size_of() -> usize {
        size_of::<Self>()
    }

    type BufferOffsets = [usize; 0];

    #[inline(always)]
    fn buffer_layout(_: usize) -> Result<(Layout, Self::BufferOffsets), LayoutError> {
        Ok((Layout::new::<Self>(), []))
    }

    type Ptrs = ();
    type MutPtrs = ();

    #[inline(always)]
    fn ptrs_dangling() -> Self::MutPtrs {}
    #[inline(always)]
    unsafe fn ptrs(_: *mut BufferData<Self>, _: &Self::BufferOffsets) -> Self::MutPtrs {}

    #[inline(always)]
    fn ptrs_cast_const(_: Self::MutPtrs) -> Self::Ptrs {}
    #[inline(always)]
    fn ptrs_cast_mut(_: Self::Ptrs) -> Self::MutPtrs {}

    #[inline(always)]
    unsafe fn ptrs_add(_: Self::Ptrs, _: usize) -> Self::Ptrs {}
    #[inline(always)]
    unsafe fn ptrs_add_mut(_: Self::MutPtrs, _: usize) -> Self::MutPtrs {}
    #[inline(always)]
    unsafe fn ptrs_offset_from(_: Self::Ptrs, _: Self::Ptrs) -> isize {
        0
    }
    #[inline(always)]
    unsafe fn ptrs_offset_from_mut(_: Self::MutPtrs, _: Self::Ptrs) -> isize {
        0
    }
    #[inline(always)]
    unsafe fn ptrs_swap(_: Self::MutPtrs, _: Self::MutPtrs) {}
    #[inline(always)]
    unsafe fn ptrs_copy(_: Self::Ptrs, _: Self::MutPtrs, _: usize) {}
    #[inline(always)]
    unsafe fn ptrs_copy_rev(_: Self::Ptrs, _: Self::MutPtrs, _: usize) {}
    #[inline(always)]
    unsafe fn ptrs_copy_nonoverlapping(_: Self::Ptrs, _: Self::MutPtrs, _: usize) {}
    #[inline(always)]
    unsafe fn ptrs_read(_: Self::Ptrs) -> Self {}
    #[inline(always)]
    unsafe fn ptrs_write(_: Self::MutPtrs, _: Self) {}
    #[inline(always)]
    unsafe fn ptrs_drop_in_place(_: Self::MutPtrs) {}

    type NonNullPtrs = ();

    #[inline(always)]
    unsafe fn ptrs_to_nonnull(_: Self::MutPtrs) -> Self::NonNullPtrs {}
    #[inline(always)]
    fn nonnull_to_ptrs(_: Self::NonNullPtrs) -> Self::MutPtrs {}

    type Vecs = ();

    #[inline(always)]
    fn vecs_with_capacity(_: usize) -> Self::Vecs {}
    #[inline(always)]
    fn vecs_as_ptrs(_: &Self::Vecs) -> Self::Ptrs {}
    #[inline(always)]
    fn mut_vecs_as_ptrs(_: &mut Self::Vecs) -> Self::MutPtrs {}
    #[inline(always)]
    fn vecs_len(_: &Self::Vecs) -> usize {
        0
    }
    #[inline(always)]
    unsafe fn vecs_set_len(_: &mut Self::Vecs, _: usize) {}

    type Refs<'a>
        = ()
    where
        Self: 'a;

    type RefsMut<'a>
        = ()
    where
        Self: 'a;

    #[inline(always)]
    unsafe fn as_refs<'a>(_: Self::Ptrs) -> Self::Refs<'a> {}
    #[inline(always)]
    unsafe fn as_mut_refs<'a>(_: Self::MutPtrs) -> Self::RefsMut<'a> {}

    #[inline(always)]
    fn refs_as_ptrs(_: Self::Refs<'_>) -> Self::Ptrs {}
    #[inline(always)]
    fn mut_refs_as_ptrs(_: Self::RefsMut<'_>) -> Self::MutPtrs {}
    #[inline(always)]
    fn mut_refs_as_refs(_: Self::RefsMut<'_>) -> Self::Refs<'_> {}

    type SlicePtrs = ();
    type SliceMutPtrs = ();

    #[inline(always)]
    fn slices_from_raw_parts(_: Self::Ptrs, _: usize) -> Self::SlicePtrs {}
    #[inline(always)]
    fn slices_from_raw_parts_mut(_: Self::MutPtrs, _: usize) -> Self::SliceMutPtrs {}

    type Slices<'a>
        = ()
    where
        Self: 'a;

    type SlicesMut<'a>
        = ()
    where
        Self: 'a;

    #[inline(always)]
    unsafe fn slices_as_refs<'a>(_: Self::SlicePtrs) -> Self::Slices<'a> {}
    #[inline(always)]
    unsafe fn mut_slices_as_refs<'a>(_: Self::SliceMutPtrs) -> Self::SlicesMut<'a> {}

    #[inline(always)]
    fn slice_refs_as_slice_ptrs(_: Self::Slices<'_>) -> Self::SlicePtrs {}
    #[inline(always)]
    fn mut_slice_refs_as_slice_ptrs(_: Self::SlicesMut<'_>) -> Self::SliceMutPtrs {}

    #[inline(always)]
    fn slice_refs_as_ptrs(_: Self::Slices<'_>) -> Self::Ptrs {}
    #[inline(always)]
    fn mut_slice_refs_as_ptrs(_: Self::SlicesMut<'_>) -> Self::MutPtrs {}

    #[inline(always)]
    unsafe fn slices_drop_in_place(_: Self::SliceMutPtrs) {}
}

impl SoaToOwned<'_> for () {
    type Owned = ();

    #[inline(always)]
    fn to_owned(&self) -> Self::Owned {}

    #[inline(always)]
    fn clone_into(&self, _: &mut Self::Owned) {}

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

            #[inline(always)]
            fn packed_size_of() -> usize {
                #[repr(packed)]
                struct PackedSelf<$($types,)*>($($types,)*);

                size_of::<PackedSelf<$($types,)*>>()
            }

            type BufferOffsets = [usize; count_idents!($($types,)*)];

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
                ($(::core::ptr::dangling_mut::<$types>(),)*)
            }

            #[inline(always)]
            unsafe fn ptrs(ptr: *mut BufferData<Self>, offsets: &Self::BufferOffsets) -> Self::MutPtrs {
                let ptr = ptr.cast::<u8>();
                unsafe { ($(ptr.add(offsets[$indices]).cast(),)*) }
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            fn ptrs_cast_const(ptrs: Self::MutPtrs) -> Self::Ptrs {
                let ($($types,)*) = ptrs;
                ($($types.cast_const(),)*)
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            fn ptrs_cast_mut(ptrs: Self::Ptrs) -> Self::MutPtrs {
                let ($($types,)*) = ptrs;
                ($($types.cast_mut(),)*)
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            unsafe fn ptrs_add(ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs {
                let ($($types,)*) = ptrs;
                unsafe { ($($types.add(offset),)*) }
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            unsafe fn ptrs_add_mut(ptrs: Self::MutPtrs, offset: usize) -> Self::MutPtrs {
                let ($($types,)*) = ptrs;
                unsafe { ($($types.add(offset),)*) }
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

                let closures = ($(|| unsafe { ::core::ptr::swap(a.$indices, b.$indices); },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline(always)]
            unsafe fn ptrs_copy(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
                let permutation = Self::field_permutation();

                let closures = ($(|| unsafe { ::core::ptr::copy(src.$indices, dst.$indices, len); },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in 0..count_idents!($($types,)*) {
                    closures[permutation[index]]();
                }
            }

            #[inline(always)]
            unsafe fn ptrs_copy_rev(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
                let permutation = Self::field_permutation();

                let closures = ($(|| unsafe { ::core::ptr::copy(src.$indices, dst.$indices, len); },)*);
                let closures: [&dyn Fn(); count_idents!($($types,)*)] = [$(&closures.$indices,)*];

                for index in (0..count_idents!($($types,)*)).rev() {
                    closures[permutation[index]]();
                }
            }

            #[inline(always)]
            unsafe fn ptrs_copy_nonoverlapping(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
                // because source and destination are non-overlapping, we can copy them in any order
                unsafe { $(::core::ptr::copy_nonoverlapping(src.$indices, dst.$indices, len);)* }
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            unsafe fn ptrs_read(ptrs: Self::Ptrs) -> Self {
                let ($($types,)*) = ptrs;
                unsafe { ($(::core::ptr::read($types),)*) }
            }

            #[inline(always)]
            unsafe fn ptrs_write(dst: Self::MutPtrs, value: Self) {
                unsafe { $(::core::ptr::write(dst.$indices, value.$indices);)* }
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            unsafe fn ptrs_drop_in_place(ptrs: Self::MutPtrs) {
                let ($($types,)*) = ptrs;
                unsafe { $(::core::ptr::drop_in_place($types);)* }
            }

            type NonNullPtrs = ($(::core::ptr::NonNull<$types>,)*);

            #[inline(always)]
            #[allow(non_snake_case)]
            unsafe fn ptrs_to_nonnull(ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
                let ($($types,)*) = ptrs;
                unsafe { ($(::core::ptr::NonNull::new_unchecked($types),)*) }
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            fn nonnull_to_ptrs(ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
                let ($($types,)*) = ptrs;
                ($($types.as_ptr(),)*)
            }

            type Vecs = ($(::alloc::vec::Vec<$types>,)*);

            #[inline(always)]
            fn vecs_with_capacity(capacity: usize) -> Self::Vecs {
                ($(::alloc::vec::Vec::<$types>::with_capacity(capacity),)*)
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
            #[allow(non_snake_case)]
            unsafe fn as_refs<'a>(ptrs: Self::Ptrs) -> Self::Refs<'a> {
                let ($($types,)*) = ptrs;
                unsafe { ($(&*$types,)*) }
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            unsafe fn as_mut_refs<'a>(ptrs: Self::MutPtrs) -> Self::RefsMut<'a> {
                let ($($types,)*) = ptrs;
                unsafe { ($(&mut *$types,)*) }
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            fn refs_as_ptrs(refs: Self::Refs<'_>) -> Self::Ptrs {
                let ($($types,)*) = refs;
                ($($types,)*)
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            fn mut_refs_as_ptrs(refs: Self::RefsMut<'_>) -> Self::MutPtrs {
                let ($($types,)*) = refs;
                ($($types,)*)
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            fn mut_refs_as_refs(refs: Self::RefsMut<'_>) -> Self::Refs<'_> {
                let ($($types,)*) = refs;
                ($($types,)*)
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            fn slices_from_raw_parts(ptrs: Self::Ptrs, len: usize) -> Self::SlicePtrs {
                let ($($types,)*) = ptrs;
                ($(::core::ptr::slice_from_raw_parts($types, len),)*)
            }

            type SlicePtrs = ($(*const [$types],)*);
            type SliceMutPtrs = ($(*mut [$types],)*);

            #[inline(always)]
            #[allow(non_snake_case)]
            fn slices_from_raw_parts_mut(ptrs: Self::MutPtrs, len: usize) -> Self::SliceMutPtrs {
                let ($($types,)*) = ptrs;
                ($(::core::ptr::slice_from_raw_parts_mut($types, len),)*)
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
            #[allow(non_snake_case)]
            unsafe fn slices_as_refs<'a>(slices: Self::SlicePtrs) -> Self::Slices<'a> {
                let ($($types,)*) = slices;
                unsafe { ($(::core::slice::from_raw_parts($types.cast(), $types.len()),)*) }
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            unsafe fn mut_slices_as_refs<'a>(slices: Self::SliceMutPtrs) -> Self::SlicesMut<'a> {
                let ($($types,)*) = slices;
                unsafe { ($(::core::slice::from_raw_parts_mut($types.cast(), $types.len()),)*) }
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            fn slice_refs_as_slice_ptrs(slices: Self::Slices<'_>) -> Self::SlicePtrs {
                let ($($types,)*) = slices;
                ($($types,)*)
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            fn mut_slice_refs_as_slice_ptrs(slices: Self::SlicesMut<'_>) -> Self::SliceMutPtrs {
                let ($($types,)*) = slices;
                ($($types,)*)
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            fn slice_refs_as_ptrs(slices: Self::Slices<'_>) -> Self::Ptrs {
                let ($($types,)*) = slices;
                ($($types.as_ptr(),)*)
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            fn mut_slice_refs_as_ptrs(slices: Self::SlicesMut<'_>) -> Self::MutPtrs {
                let ($($types,)*) = slices;
                ($($types.as_mut_ptr(),)*)
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            unsafe fn slices_drop_in_place(slices: Self::SliceMutPtrs) {
                let ($($types,)*) = slices;
                unsafe { $(::core::ptr::drop_in_place($types);)* }
            }
        }

        impl<'a, $($types,)*> SoaToOwned<'a> for ($(&'a $types,)*)
        where
            $($types: Clone,)*
        {
            type Owned = ($($types,)*);

            #[inline(always)]
            #[allow(non_snake_case)]
            fn to_owned(&self) -> Self::Owned {
                let ($($types,)*) = *self;
                ($($types.clone(),)*)
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            fn clone_into(&self, target: &mut Self::Owned) {
                let ($($types,)*) = *self;
                $(target.$indices.clone_from($types);)*
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            fn clone_into_refs(&self, target: <Self::Owned as Soa>::RefsMut<'_>) {
                let ($($types,)*) = *self;
                $(target.$indices.clone_from($types);)*
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
