use crate::{
    slices::{SoaSlice, SoaView, SoaViewMut, partial_eq_impl},
    traits::AllocSoaTrusted,
};

partial_eq_impl! { [] SoaView<'_, '_, T>, SoaSlice<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaView<'_, '_, T>, &SoaSlice<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaView<'_, '_, T>, &mut SoaSlice<T> where T: AllocSoaTrusted }

partial_eq_impl! { [] SoaViewMut<'_, '_, T>, SoaSlice<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaViewMut<'_, '_, T>, &SoaSlice<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaViewMut<'_, '_, T>, &mut SoaSlice<T> where T: AllocSoaTrusted }

partial_eq_impl! { [] SoaSlice<T>, Self where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaSlice<T>, SoaView<'_, '_, T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaSlice<T>, SoaViewMut<'_, '_, T> where T: AllocSoaTrusted }
partial_eq_impl! { [] &SoaSlice<T>, SoaView<'_, '_, T> where T: AllocSoaTrusted }
partial_eq_impl! { [] &SoaSlice<T>, SoaViewMut<'_, '_, T> where T: AllocSoaTrusted }
partial_eq_impl! { [] &mut SoaSlice<T>, SoaView<'_, '_, T> where T: AllocSoaTrusted }
partial_eq_impl! { [] &mut SoaSlice<T>, SoaViewMut<'_, '_, T> where T: AllocSoaTrusted }
