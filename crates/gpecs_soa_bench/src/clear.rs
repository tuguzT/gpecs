use gpecs_soa::{prelude::*, traits::SoaVecs};
use gpecs_soa_erased::erased::ErasedSoa;

use crate::{Big, Large, Medium, Small, Tiny, Zero};

pub trait Clear: SoaVecs {
    fn soa_slf_clear(vec: &mut SoaVec<Self>) {
        vec.clear();
    }

    fn soa_ser_clear(vec: &mut SoaVec<ErasedSoa>) {
        vec.clear();
    }

    fn soa_std_clear(vecs: &mut Self::Vecs);

    fn aos_std_clear(vec: &mut Vec<Self>) {
        vec.clear();
    }
}

impl Clear for Zero {
    fn soa_std_clear(vecs: &mut Self::Vecs) {
        vecs.clear();
    }
}

impl Clear for Tiny {
    fn soa_std_clear(vecs: &mut Self::Vecs) {
        let (values,) = vecs;
        values.clear();
    }
}

impl Clear for Small {
    fn soa_std_clear(vecs: &mut Self::Vecs) {
        let (xs, ys, zs) = vecs;
        xs.clear();
        ys.clear();
        zs.clear();
    }
}

impl Clear for Medium {
    fn soa_std_clear(vecs: &mut Self::Vecs) {
        let (smalls1, smalls2, smalls3) = vecs;
        smalls1.clear();
        smalls2.clear();
        smalls3.clear();
    }
}

impl Clear for Big {
    fn soa_std_clear(vecs: &mut Self::Vecs) {
        let (smalls1, smalls2, arrays, strs1, strs2) = vecs;
        smalls1.clear();
        smalls2.clear();
        arrays.clear();
        strs1.clear();
        strs2.clear();
    }
}

impl Clear for Large {
    fn soa_std_clear(vecs: &mut Self::Vecs) {
        let (
            arrays1,
            arrays2,
            arrays3,
            arrays4,
            arrays5,
            arrays6,
            arrays7,
            arrays8,
            arrays9,
            arrays10,
        ) = vecs;
        arrays1.clear();
        arrays2.clear();
        arrays3.clear();
        arrays4.clear();
        arrays5.clear();
        arrays6.clear();
        arrays7.clear();
        arrays8.clear();
        arrays9.clear();
        arrays10.clear();
    }
}
