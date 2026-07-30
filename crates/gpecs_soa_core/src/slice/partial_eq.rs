use crate::slice::{SoaSlices, SoaSlicesMut};

// Slightly modified version of one from crate `alloc`: src/vec/partial_eq.rs
#[macro_export]
#[doc(hidden)]
macro_rules! partial_eq_impl {
    ([$($vars:tt)*] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)?) => {
        impl<T, $($vars)*> ::core::cmp::PartialEq<$rhs> for $lhs
        where
            $($ty: $bound,)?
            T: $crate::traits::SoaOwned + ?::core::marker::Sized,
            for<'_c, '_a> $crate::traits::Slices<'_c, '_a, T>: ::core::cmp::PartialEq,
        {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                self.as_slices() == other.as_slices()
            }
        }
    }
}

#[doc(hidden)]
pub use partial_eq_impl;

partial_eq_impl! { [] SoaSlices<'_, '_, T>, Self }
partial_eq_impl! { [] SoaSlices<'_, '_, T>, SoaSlicesMut<'_, '_, T> }

partial_eq_impl! { [] SoaSlicesMut<'_, '_, T>, Self }
partial_eq_impl! { [] SoaSlicesMut<'_, '_, T>, SoaSlices<'_, '_, T> }
