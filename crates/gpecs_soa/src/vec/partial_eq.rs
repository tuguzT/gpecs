use super::{Soa, SoaSlice, SoaVec};

// Slightly modified version of one from crate `alloc`: src/vec/partial_eq.rs
macro_rules! __impl_slice_eq {
    ([$($vars:tt)*] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)?) => {
        impl<T, $($vars)*> PartialEq<$rhs> for $lhs
        where
            T: Soa,
            for<'a> T::Slices<'a>: PartialEq,
            $($ty: $bound)?
        {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool { self.as_slices() == other.as_slices() }
            #[inline]
            #[allow(clippy::partialeq_ne_impl)]
            fn ne(&self, other: &$rhs) -> bool { self.as_slices() != other.as_slices() }
        }
    }
}

__impl_slice_eq! { [] SoaVec<T>, SoaVec<T> }

__impl_slice_eq! { [] SoaVec<T>, SoaSlice<T> }
__impl_slice_eq! { [] SoaVec<T>, &SoaSlice<T> }
__impl_slice_eq! { [] SoaVec<T>, &mut SoaSlice<T> }

__impl_slice_eq! { [] SoaSlice<T>, SoaVec<T> }
__impl_slice_eq! { [] &SoaSlice<T>, SoaVec<T> }
__impl_slice_eq! { [] &mut SoaSlice<T>, SoaVec<T> }
