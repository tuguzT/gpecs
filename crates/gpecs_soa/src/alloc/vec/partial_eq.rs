use crate::{
    slices::{SoaSlice, SoaView, SoaViewMut, partial_eq_impl},
    traits::{AllocSoa, AllocSoaTrusted},
    vec::SoaVec,
};

partial_eq_impl! { [] SoaVec<T>, Self where T: AllocSoa }
partial_eq_impl! { [] SoaVec<T>, SoaView<'_, '_, T> where T: AllocSoa }
partial_eq_impl! { [] SoaVec<T>, SoaViewMut<'_, '_, T> where T: AllocSoa }
partial_eq_impl! { [] SoaVec<T>, SoaSlice<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaVec<T>, &SoaSlice<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaVec<T>, &mut SoaSlice<T> where T: AllocSoaTrusted }

partial_eq_impl! { [] SoaView<'_, '_, T>, SoaVec<T> where T: AllocSoa }
partial_eq_impl! { [] SoaViewMut<'_, '_, T>, SoaVec<T> where T: AllocSoa }
partial_eq_impl! { [] SoaSlice<T>, SoaVec<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] &SoaSlice<T>, SoaVec<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] &mut SoaSlice<T>, SoaVec<T> where T: AllocSoaTrusted }
