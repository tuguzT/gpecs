use crate::{
    slice::{SoaSlice, SoaSlices, SoaSlicesMut},
    traits::SoaTrustedFields,
};

// Slightly modified version of one from crate `alloc`: src/vec/partial_eq.rs
macro_rules! partial_ord_impl {
    ([$($vars:tt)*] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)?) => {
        impl<T, $($vars)*> PartialOrd<$rhs> for $lhs
        where
            $($ty: $bound,)?
            T: ?Sized,
            for<'_a> T: $crate::traits::Soa<'_a>,
            for<'_c, '_a> $crate::traits::Slices<'_c, '_a, T>: PartialOrd,
        {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<::core::cmp::Ordering> {
                let this = self.as_slices();
                let other = other.as_slices();
                PartialOrd::partial_cmp(&this, &other)
            }
        }
    }
}

pub(crate) use partial_ord_impl;

partial_ord_impl! { [] SoaSlices<'_, '_, T>, Self }
partial_ord_impl! { [] SoaSlices<'_, '_, T>, SoaSlicesMut<'_, '_, T> }
partial_ord_impl! { [] SoaSlices<'_, '_, T>, SoaSlice<T> where T: SoaTrustedFields }
partial_ord_impl! { [] SoaSlices<'_, '_, T>, &SoaSlice<T> where T: SoaTrustedFields }
partial_ord_impl! { [] SoaSlices<'_, '_, T>, &mut SoaSlice<T> where T: SoaTrustedFields }

partial_ord_impl! { [] SoaSlicesMut<'_, '_, T>, Self }
partial_ord_impl! { [] SoaSlicesMut<'_, '_, T>, SoaSlices<'_, '_, T> }
partial_ord_impl! { [] SoaSlicesMut<'_, '_, T>, SoaSlice<T> where T: SoaTrustedFields }
partial_ord_impl! { [] SoaSlicesMut<'_, '_, T>, &SoaSlice<T> where T: SoaTrustedFields }
partial_ord_impl! { [] SoaSlicesMut<'_, '_, T>, &mut SoaSlice<T> where T: SoaTrustedFields }

partial_ord_impl! { [] SoaSlice<T>, Self where T: SoaTrustedFields }
partial_ord_impl! { [] SoaSlice<T>, SoaSlices<'_, '_, T> where T: SoaTrustedFields }
partial_ord_impl! { [] SoaSlice<T>, SoaSlicesMut<'_, '_, T> where T: SoaTrustedFields }
partial_ord_impl! { [] &SoaSlice<T>, SoaSlices<'_, '_, T> where T: SoaTrustedFields }
partial_ord_impl! { [] &SoaSlice<T>, SoaSlicesMut<'_, '_, T> where T: SoaTrustedFields }
partial_ord_impl! { [] &mut SoaSlice<T>, SoaSlices<'_, '_, T> where T: SoaTrustedFields }
partial_ord_impl! { [] &mut SoaSlice<T>, SoaSlicesMut<'_, '_, T> where T: SoaTrustedFields }
