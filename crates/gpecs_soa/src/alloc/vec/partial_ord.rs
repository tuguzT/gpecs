use crate::{
    slice::SoaSlice,
    traits::{Soa, SoaTrustedFields},
};

use super::SoaVec;

// Slightly modified version of one from crate `alloc`: src/vec/partial_eq.rs
macro_rules! __impl_slice_ord {
    ([$($vars:tt)*] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)?) => {
        impl<T, $($vars)*> PartialOrd<$rhs> for $lhs
        where
            T: SoaTrustedFields,
            for<'c, 'any> T::Slices<'c, 'any>: PartialOrd,
            $($ty: $bound)?
        {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<::core::cmp::Ordering> {
                PartialOrd::partial_cmp(&self.as_slices(), &other.as_slices())
            }
        }
    }
}

impl<T> PartialOrd<SoaVec<T>> for SoaVec<T>
where
    T: Soa,
    for<'c, 'any> T::Slices<'c, 'any>: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &SoaVec<T>) -> Option<::core::cmp::Ordering> {
        PartialOrd::partial_cmp(&self.as_slices(), &other.as_slices())
    }
}

__impl_slice_ord! { [] SoaVec<T>, SoaSlice<T> }
__impl_slice_ord! { [] SoaVec<T>, &SoaSlice<T> }
__impl_slice_ord! { [] SoaVec<T>, &mut SoaSlice<T> }

__impl_slice_ord! { [] SoaSlice<T>, SoaVec<T> }
__impl_slice_ord! { [] &SoaSlice<T>, SoaVec<T> }
__impl_slice_ord! { [] &mut SoaSlice<T>, SoaVec<T> }
