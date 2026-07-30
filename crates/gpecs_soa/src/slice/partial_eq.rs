use crate::{
    slice::{SoaSlice, SoaSlices, SoaSlicesMut, partial_eq_impl},
    traits::AllocSoaTrusted,
};

partial_eq_impl! { [] SoaSlices<'_, '_, T>, SoaSlice<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaSlices<'_, '_, T>, &SoaSlice<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaSlices<'_, '_, T>, &mut SoaSlice<T> where T: AllocSoaTrusted }

partial_eq_impl! { [] SoaSlicesMut<'_, '_, T>, SoaSlice<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaSlicesMut<'_, '_, T>, &SoaSlice<T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaSlicesMut<'_, '_, T>, &mut SoaSlice<T> where T: AllocSoaTrusted }

partial_eq_impl! { [] SoaSlice<T>, Self where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaSlice<T>, SoaSlices<'_, '_, T> where T: AllocSoaTrusted }
partial_eq_impl! { [] SoaSlice<T>, SoaSlicesMut<'_, '_, T> where T: AllocSoaTrusted }
partial_eq_impl! { [] &SoaSlice<T>, SoaSlices<'_, '_, T> where T: AllocSoaTrusted }
partial_eq_impl! { [] &SoaSlice<T>, SoaSlicesMut<'_, '_, T> where T: AllocSoaTrusted }
partial_eq_impl! { [] &mut SoaSlice<T>, SoaSlices<'_, '_, T> where T: AllocSoaTrusted }
partial_eq_impl! { [] &mut SoaSlice<T>, SoaSlicesMut<'_, '_, T> where T: AllocSoaTrusted }
