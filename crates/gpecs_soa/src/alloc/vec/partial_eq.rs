use crate::{
    slice::{SoaSlice, SoaSlices, SoaSlicesMut, partial_eq_impl},
    traits::{Soa, SoaTrustedFields},
    vec::SoaVec,
};

partial_eq_impl! { [] SoaVec<T>, Self where T: Soa }
partial_eq_impl! { [] SoaVec<T>, SoaSlices<'_, '_, T> where T: Soa }
partial_eq_impl! { [] SoaVec<T>, SoaSlicesMut<'_, '_, T> where T: Soa }
partial_eq_impl! { [] SoaVec<T>, SoaSlice<T> where T: SoaTrustedFields }
partial_eq_impl! { [] SoaVec<T>, &SoaSlice<T> where T: SoaTrustedFields }
partial_eq_impl! { [] SoaVec<T>, &mut SoaSlice<T> where T: SoaTrustedFields }

partial_eq_impl! { [] SoaSlices<'_, '_, T>, SoaVec<T> where T: Soa }
partial_eq_impl! { [] SoaSlicesMut<'_, '_, T>, SoaVec<T> where T: Soa }
partial_eq_impl! { [] SoaSlice<T>, SoaVec<T> where T: SoaTrustedFields }
partial_eq_impl! { [] &SoaSlice<T>, SoaVec<T> where T: SoaTrustedFields }
partial_eq_impl! { [] &mut SoaSlice<T>, SoaVec<T> where T: SoaTrustedFields }
