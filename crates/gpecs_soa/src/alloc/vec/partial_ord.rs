use crate::{
    slices::{SoaSlice, SoaView, SoaViewMut, partial_ord_impl},
    traits::{AllocSoa, AllocSoaTrusted},
    vec::SoaVec,
};

partial_ord_impl! { [] SoaVec<T>, Self where T: AllocSoa }
partial_ord_impl! { [] SoaVec<T>, SoaView<'_, '_, T> where T: AllocSoa }
partial_ord_impl! { [] SoaVec<T>, SoaViewMut<'_, '_, T> where T: AllocSoa }
partial_ord_impl! { [] SoaVec<T>, SoaSlice<T> where T: AllocSoaTrusted }
partial_ord_impl! { [] SoaVec<T>, &SoaSlice<T> where T: AllocSoaTrusted }
partial_ord_impl! { [] SoaVec<T>, &mut SoaSlice<T> where T: AllocSoaTrusted }

partial_ord_impl! { [] SoaView<'_, '_, T>, SoaVec<T> where T: AllocSoa }
partial_ord_impl! { [] SoaViewMut<'_, '_, T>, SoaVec<T> where T: AllocSoa }
partial_ord_impl! { [] SoaSlice<T>, SoaVec<T> where T: AllocSoaTrusted }
partial_ord_impl! { [] &SoaSlice<T>, SoaVec<T> where T: AllocSoaTrusted }
partial_ord_impl! { [] &mut SoaSlice<T>, SoaVec<T> where T: AllocSoaTrusted }
