use crate::{
    slice::SoaSlice,
    traits::{Soa, SoaTrustedFields},
};

use super::SoaVec;

// Slightly modified version of one from crate `alloc`: src/vec/partial_eq.rs
macro_rules! __impl_slice_eq {
    ([$($vars:tt)*] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)?) => {
        impl<T, $($vars)*> PartialEq<$rhs> for $lhs
        where
            T: SoaTrustedFields + ?Sized,
            for<'c, 'any> T::Slices<'c, 'any>: PartialEq,
            $($ty: $bound)?
        {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool { self.as_slices() == other.as_slices() }
        }
    }
}

impl<T> PartialEq<Self> for SoaVec<T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slices() == other.as_slices()
    }
}

__impl_slice_eq! { [] SoaVec<T>, SoaSlice<T> }
__impl_slice_eq! { [] SoaVec<T>, &SoaSlice<T> }
__impl_slice_eq! { [] SoaVec<T>, &mut SoaSlice<T> }

__impl_slice_eq! { [] SoaSlice<T>, SoaVec<T> }
__impl_slice_eq! { [] &SoaSlice<T>, SoaVec<T> }
__impl_slice_eq! { [] &mut SoaSlice<T>, SoaVec<T> }
