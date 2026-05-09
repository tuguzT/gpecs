use crate::{
    slice::{SoaSlice, SoaSlices, SoaSlicesMut},
    traits::AllocSoaTrusted,
};

// Slightly modified version of one from crate `alloc`: src/vec/partial_eq.rs
macro_rules! partial_eq_impl {
    ([$($vars:tt)*] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)?) => {
        impl<T, $($vars)*> PartialEq<$rhs> for $lhs
        where
            $($ty: $bound,)?
            T: $crate::traits::SoaOwned + ?Sized,
            for<'_c, '_a> $crate::traits::Slices<'_c, '_a, T>: PartialEq,
        {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                self.as_slices() == other.as_slices()
            }
        }
    }
}

#[cfg_attr(not(feature = "alloc"), expect(unused))]
pub(crate) use partial_eq_impl;

partial_eq_impl! { [] SoaSlices<'_, '_, T>, Self }
partial_eq_impl! { [] SoaSlices<'_, '_, T>, SoaSlicesMut<'_, '_, T> }
partial_eq_impl! { [] SoaSlices<'_, '_, T>, SoaSlice<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaSlices<'_, '_, T>, &SoaSlice<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaSlices<'_, '_, T>, &mut SoaSlice<T> where T: AllocSoaTrusted }

partial_eq_impl! { [] SoaSlicesMut<'_, '_, T>, Self }
partial_eq_impl! { [] SoaSlicesMut<'_, '_, T>, SoaSlices<'_, '_, T> }
partial_eq_impl! { [] SoaSlicesMut<'_, '_, T>, SoaSlice<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaSlicesMut<'_, '_, T>, &SoaSlice<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaSlicesMut<'_, '_, T>, &mut SoaSlice<T> where T: AllocSoaTrusted }

partial_eq_impl! { [] SoaSlice<T>, Self where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaSlice<T>, SoaSlices<'_, '_, T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaSlice<T>, SoaSlicesMut<'_, '_, T> where T: AllocSoaTrusted }
partial_eq_impl! { [] &SoaSlice<T>, SoaSlices<'_, '_, T> where T: AllocSoaTrusted }
partial_eq_impl! { [] &SoaSlice<T>, SoaSlicesMut<'_, '_, T> where T: AllocSoaTrusted }
partial_eq_impl! { [] &mut SoaSlice<T>, SoaSlices<'_, '_, T> where T: AllocSoaTrusted }
partial_eq_impl! { [] &mut SoaSlice<T>, SoaSlicesMut<'_, '_, T> where T: AllocSoaTrusted }
