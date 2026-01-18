use crate::{
    slice::{SoaSlice, SoaSlices, SoaSlicesMut, partial_ord_impl},
    traits::{AllocSoa, AllocSoaTrusted},
    vec::SoaVec,
};

partial_ord_impl! { [] SoaVec<T>, Self where T: AllocSoa }
partial_ord_impl! { [] SoaVec<T>, SoaSlices<'_, '_, T> where T: AllocSoa }
partial_ord_impl! { [] SoaVec<T>, SoaSlicesMut<'_, '_, T> where T: AllocSoa }
partial_ord_impl! { [] SoaVec<T>, SoaSlice<T> where T: AllocSoaTrusted }
partial_ord_impl! { [] SoaVec<T>, &SoaSlice<T> where T: AllocSoaTrusted }
partial_ord_impl! { [] SoaVec<T>, &mut SoaSlice<T> where T: AllocSoaTrusted }

partial_ord_impl! { [] SoaSlices<'_, '_, T>, SoaVec<T> where T: AllocSoa }
partial_ord_impl! { [] SoaSlicesMut<'_, '_, T>, SoaVec<T> where T: AllocSoa }
partial_ord_impl! { [] SoaSlice<T>, SoaVec<T> where T: AllocSoaTrusted }
partial_ord_impl! { [] &SoaSlice<T>, SoaVec<T> where T: AllocSoaTrusted }
partial_ord_impl! { [] &mut SoaSlice<T>, SoaVec<T> where T: AllocSoaTrusted }
