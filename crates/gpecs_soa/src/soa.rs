use core::{
    alloc::{Layout, LayoutError},
    marker::PhantomData,
};

#[allow(clippy::missing_safety_doc)]
pub unsafe trait Soa: Sized {
    type Ptrs: Copy;
    type MutPtrs: Copy;
    type NonNullPtrs: Copy;
    type Offsets: AsRef<[usize]>;

    type Refs<'a>
    where
        Self: 'a;

    type RefsMut<'a>
    where
        Self: 'a;

    type SlicePtrs: Copy;
    type SliceMutPtrs: Copy;

    type Slices<'a>
    where
        Self: 'a;

    type SlicesMut<'a>
    where
        Self: 'a;

    fn packed_size_of() -> usize;
    fn buffer_layout_unaligned(
        initial: Layout,
        capacity: usize,
    ) -> Result<(Layout, Self::Offsets), LayoutError>;

    fn ptrs_dangling() -> Self::MutPtrs;
    unsafe fn ptrs(ptr: *mut u8, initial: Layout, capacity: usize) -> Self::MutPtrs;
    unsafe fn ptrs_to_nonnull(ptrs: Self::MutPtrs) -> Self::NonNullPtrs;

    fn ptrs_cast_const(ptrs: Self::MutPtrs) -> Self::Ptrs;
    fn ptrs_cast_mut(ptrs: Self::Ptrs) -> Self::MutPtrs;

    unsafe fn ptrs_add(ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs;
    unsafe fn ptrs_add_mut(ptrs: Self::MutPtrs, offset: usize) -> Self::MutPtrs;
    unsafe fn ptrs_swap(a: Self::MutPtrs, b: Self::MutPtrs);
    unsafe fn ptrs_copy(src: Self::Ptrs, dst: Self::MutPtrs, len: usize);
    unsafe fn ptrs_copy_rev(src: Self::Ptrs, dst: Self::MutPtrs, len: usize);
    unsafe fn ptrs_copy_nonoverlapping(src: Self::Ptrs, dst: Self::MutPtrs, len: usize);
    unsafe fn ptrs_read(src: Self::Ptrs) -> Self;
    unsafe fn ptrs_write(dst: Self::MutPtrs, value: Self);
    unsafe fn ptrs_drop_in_place(ptrs: Self::MutPtrs);

    unsafe fn as_refs<'a>(ptrs: Self::Ptrs) -> Self::Refs<'a>;
    unsafe fn as_mut_refs<'a>(ptrs: Self::MutPtrs) -> Self::RefsMut<'a>;

    fn refs_as_ptrs(refs: Self::Refs<'_>) -> Self::Ptrs;
    fn mut_refs_as_ptrs(refs: Self::RefsMut<'_>) -> Self::MutPtrs;
    fn mut_refs_as_refs(refs: Self::RefsMut<'_>) -> Self::Refs<'_>;

    fn slices_from_raw_parts(ptrs: Self::Ptrs, len: usize) -> Self::SlicePtrs;
    fn slices_from_raw_parts_mut(ptrs: Self::MutPtrs, len: usize) -> Self::SliceMutPtrs;

    unsafe fn slices_as_refs<'a>(slices: Self::SlicePtrs) -> Self::Slices<'a>;
    unsafe fn mut_slices_as_refs<'a>(slices: Self::SliceMutPtrs) -> Self::SlicesMut<'a>;

    fn slice_refs_as_ptrs(slices: Self::Slices<'_>) -> Self::SlicePtrs;
    fn mut_slice_refs_as_ptrs(slices: Self::SlicesMut<'_>) -> Self::SliceMutPtrs;

    unsafe fn slices_drop_in_place(slices: Self::SliceMutPtrs);
}

unsafe impl Soa for () {
    type Ptrs = ();
    type MutPtrs = ();
    type NonNullPtrs = ();
    type Offsets = [usize; 0];

    type Refs<'a>
        = ()
    where
        Self: 'a;

    type RefsMut<'a>
        = ()
    where
        Self: 'a;

    type SlicePtrs = ();
    type SliceMutPtrs = ();

    type Slices<'a>
        = ()
    where
        Self: 'a;

    type SlicesMut<'a>
        = ()
    where
        Self: 'a;

    #[inline(always)]
    fn packed_size_of() -> usize {
        size_of::<Self>()
    }

    #[inline(always)]
    fn buffer_layout_unaligned(
        initial: Layout,
        _: usize,
    ) -> Result<(Layout, Self::Offsets), LayoutError> {
        Ok((initial, []))
    }

    #[inline(always)]
    fn ptrs_dangling() -> Self::MutPtrs {}
    #[inline(always)]
    unsafe fn ptrs(_: *mut u8, _: Layout, _: usize) -> Self::MutPtrs {}
    #[inline(always)]
    unsafe fn ptrs_to_nonnull(_: Self::MutPtrs) -> Self::NonNullPtrs {}

    #[inline(always)]
    fn ptrs_cast_const(_: Self::MutPtrs) -> Self::Ptrs {}
    #[inline(always)]
    fn ptrs_cast_mut(_: Self::Ptrs) -> Self::MutPtrs {}

    #[inline(always)]
    unsafe fn ptrs_add(_: Self::Ptrs, _: usize) -> Self::Ptrs {}
    #[inline(always)]
    unsafe fn ptrs_add_mut(_: Self::MutPtrs, _: usize) -> Self::MutPtrs {}
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

    #[inline(always)]
    fn slices_from_raw_parts(_: Self::Ptrs, _: usize) -> Self::SlicePtrs {}
    #[inline(always)]
    fn slices_from_raw_parts_mut(_: Self::MutPtrs, _: usize) -> Self::SliceMutPtrs {}

    #[inline(always)]
    unsafe fn slices_as_refs<'a>(_: Self::SlicePtrs) -> Self::Slices<'a> {}
    #[inline(always)]
    unsafe fn mut_slices_as_refs<'a>(_: Self::SliceMutPtrs) -> Self::SlicesMut<'a> {}

    #[inline(always)]
    fn slice_refs_as_ptrs(_: Self::Slices<'_>) -> Self::SlicePtrs {}
    #[inline(always)]
    fn mut_slice_refs_as_ptrs(_: Self::SlicesMut<'_>) -> Self::SliceMutPtrs {}

    #[inline(always)]
    unsafe fn slices_drop_in_place(_: Self::SliceMutPtrs) {}
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
    ($($types:ident index $indices:tt reversed_index $reversed_indices:tt),* $(,)?) => {
        impl<$($types,)*> SoaTupleConst<($($types,)*)> {
            const LAYOUTS: [Layout; count_idents!($($types,)*)] = [
                $(Layout::new::<$types>(),)*
            ];
        }

        unsafe impl<$($types,)*> Soa for ($($types,)*) {
            type Ptrs = ($(*const $types,)*);
            type MutPtrs = ($(*mut $types,)*);
            type NonNullPtrs = ($(::core::ptr::NonNull<$types>,)*);
            type Offsets = [usize; count_idents!($($types,)*)];

            type Refs<'a>
                = ($(&'a $types,)*)
            where
                Self: 'a;

            type RefsMut<'a>
                = ($(&'a mut $types,)*)
            where
                Self: 'a;

            type SlicePtrs = ($(*const [$types],)*);
            type SliceMutPtrs = ($(*mut [$types],)*);

            type Slices<'a>
                = ($(&'a [$types],)*)
            where
                Self: 'a;

            type SlicesMut<'a>
                = ($(&'a mut [$types],)*)
            where
                Self: 'a;

            #[inline(always)]
            fn packed_size_of() -> usize {
                #[repr(packed)]
                struct PackedSelf<$($types,)*>($($types,)*);

                size_of::<PackedSelf<$($types,)*>>()
            }

            fn buffer_layout_unaligned(
                initial: Layout,
                capacity: usize,
            ) -> Result<(Layout, Self::Offsets), LayoutError> {
                let layouts = SoaTupleConst::<($($types,)*)>::LAYOUTS;
                let permutation = { // lack of compile-time sorting: hope this gets optimized away
                    let mut permutation = [$($indices,)*];
                    permutation.sort_unstable_by_key(|&index| layouts[index].align());
                    permutation
                };

                let layouts = [$(Layout::array::<$types>(capacity)?,)*];
                let mut offsets = Self::Offsets::default();

                let layout = initial;
                $(
                    let (layout, offset) = layout.extend(layouts[permutation[$indices]])?;
                    offsets[permutation[$indices]] = offset;
                )*

                Ok((layout, offsets))
            }

            #[inline(always)]
            fn ptrs_dangling() -> Self::MutPtrs {
                ($(::core::ptr::NonNull::<$types>::dangling().as_ptr(),)*)
            }

            #[inline(always)]
            unsafe fn ptrs(ptr: *mut u8, initial: Layout, capacity: usize) -> Self::MutPtrs {
                let (_, offsets) = Self::buffer_layout_unaligned(initial, capacity).expect("layout size should not exceed `isize::MAX`");
                unsafe { ($(ptr.add(offsets[$indices]).cast(),)*) }
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            unsafe fn ptrs_to_nonnull(ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
                let ($($types,)*) = ptrs;
                unsafe { ($(::core::ptr::NonNull::new_unchecked($types),)*) }
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
            unsafe fn ptrs_swap(a: Self::MutPtrs, b: Self::MutPtrs) {
                unsafe { $(::core::ptr::swap(a.$indices, b.$indices);)* }
            }

            #[inline(always)]
            unsafe fn ptrs_copy(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
                unsafe { $(::core::ptr::copy(src.$indices, dst.$indices, len);)* }
            }

            #[inline(always)]
            unsafe fn ptrs_copy_rev(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
                unsafe { $(::core::ptr::copy(src.$reversed_indices, dst.$reversed_indices, len);)* }
            }

            #[inline(always)]
            unsafe fn ptrs_copy_nonoverlapping(src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
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

            #[inline(always)]
            #[allow(non_snake_case)]
            fn slices_from_raw_parts_mut(ptrs: Self::MutPtrs, len: usize) -> Self::SliceMutPtrs {
                let ($($types,)*) = ptrs;
                ($(::core::ptr::slice_from_raw_parts_mut($types, len),)*)
            }

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
            fn slice_refs_as_ptrs(slices: Self::Slices<'_>) -> Self::SlicePtrs {
                let ($($types,)*) = slices;
                ($($types,)*)
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            fn mut_slice_refs_as_ptrs(slices: Self::SlicesMut<'_>) -> Self::SliceMutPtrs {
                let ($($types,)*) = slices;
                ($($types,)*)
            }

            #[inline(always)]
            #[allow(non_snake_case)]
            unsafe fn slices_drop_in_place(slices: Self::SliceMutPtrs) {
                let ($($types,)*) = slices;
                unsafe { $(::core::ptr::drop_in_place($types);)* }
            }
        }
    };
}

soa_impl!(
    A index 0 reversed_index 0,
);

soa_impl!(
    A index 0 reversed_index 1,
    B index 1 reversed_index 0,
);

soa_impl!(
    A index 0 reversed_index 2,
    B index 1 reversed_index 1,
    C index 2 reversed_index 0,
);

soa_impl!(
    A index 0 reversed_index 3,
    B index 1 reversed_index 2,
    C index 2 reversed_index 1,
    D index 3 reversed_index 0,
);

soa_impl!(
    A index 0 reversed_index 4,
    B index 1 reversed_index 3,
    C index 2 reversed_index 2,
    D index 3 reversed_index 1,
    E index 4 reversed_index 0,
);

soa_impl!(
    A index 0 reversed_index 5,
    B index 1 reversed_index 4,
    C index 2 reversed_index 3,
    D index 3 reversed_index 2,
    E index 4 reversed_index 1,
    F index 5 reversed_index 0,
);

soa_impl!(
    A index 0 reversed_index 6,
    B index 1 reversed_index 5,
    C index 2 reversed_index 4,
    D index 3 reversed_index 3,
    E index 4 reversed_index 2,
    F index 5 reversed_index 1,
    G index 6 reversed_index 0,
);

soa_impl!(
    A index 0 reversed_index 7,
    B index 1 reversed_index 6,
    C index 2 reversed_index 5,
    D index 3 reversed_index 4,
    E index 4 reversed_index 3,
    F index 5 reversed_index 2,
    G index 6 reversed_index 1,
    H index 7 reversed_index 0,
);

soa_impl!(
    A index 0 reversed_index 8,
    B index 1 reversed_index 7,
    C index 2 reversed_index 6,
    D index 3 reversed_index 5,
    E index 4 reversed_index 4,
    F index 5 reversed_index 3,
    G index 6 reversed_index 2,
    H index 7 reversed_index 1,
    I index 8 reversed_index 0,
);

soa_impl!(
    A index 0 reversed_index 9,
    B index 1 reversed_index 8,
    C index 2 reversed_index 7,
    D index 3 reversed_index 6,
    E index 4 reversed_index 5,
    F index 5 reversed_index 4,
    G index 6 reversed_index 3,
    H index 7 reversed_index 2,
    I index 8 reversed_index 1,
    J index 9 reversed_index 0,
);

soa_impl!(
    A index 0  reversed_index 10,
    B index 1  reversed_index 9,
    C index 2  reversed_index 8,
    D index 3  reversed_index 7,
    E index 4  reversed_index 6,
    F index 5  reversed_index 5,
    G index 6  reversed_index 4,
    H index 7  reversed_index 3,
    I index 8  reversed_index 2,
    J index 9  reversed_index 1,
    K index 10 reversed_index 0,
);

soa_impl!(
    A index 0  reversed_index 11,
    B index 1  reversed_index 10,
    C index 2  reversed_index 9,
    D index 3  reversed_index 8,
    E index 4  reversed_index 7,
    F index 5  reversed_index 6,
    G index 6  reversed_index 5,
    H index 7  reversed_index 4,
    I index 8  reversed_index 3,
    J index 9  reversed_index 2,
    K index 10 reversed_index 1,
    L index 11 reversed_index 0,
);
