use crate::slice::{SoaSlices, SoaSlicesMut};

// Slightly modified version of one from crate `alloc`: src/vec/partial_eq.rs
#[macro_export]
#[doc(hidden)]
macro_rules! partial_ord_impl {
    ([$($vars:tt)*] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)?) => {
        impl<T, $($vars)*> ::core::cmp::PartialOrd<$rhs> for $lhs
        where
            $($ty: $bound,)?
            T: $crate::traits::SoaOwned + ?::core::marker::Sized,
            for<'_c, '_a> $crate::traits::Slices<'_c, '_a, T>: ::core::cmp::PartialOrd,
        {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<::core::cmp::Ordering> {
                let this = self.as_slices();
                let other = other.as_slices();
                ::core::cmp::PartialOrd::partial_cmp(&this, &other)
            }
        }
    }
}

#[doc(hidden)]
pub use partial_ord_impl;

partial_ord_impl! { [] SoaSlices<'_, '_, T>, Self }
partial_ord_impl! { [] SoaSlices<'_, '_, T>, SoaSlicesMut<'_, '_, T> }

partial_ord_impl! { [] SoaSlicesMut<'_, '_, T>, Self }
partial_ord_impl! { [] SoaSlicesMut<'_, '_, T>, SoaSlices<'_, '_, T> }
