use std::hint::black_box;

use gpecs_soa::{prelude::*, traits::SoaVecs};

#[cfg(feature = "erased")]
use gpecs_soa_erased::erased::ErasedSoa;

use crate::{Big, Large, Medium, Small, Tiny, Zero};

pub trait Push: SoaVecs {
    fn soa_slf_push(vec: &mut SoaVec<Self>, value: Self) {
        let value = black_box(value);
        vec.push(value);
    }

    #[cfg(feature = "erased")]
    fn soa_ser_push(vec: &mut SoaVec<ErasedSoa>, value: ErasedSoa) {
        let value = black_box(value);
        vec.push(value);
    }

    fn soa_std_push(vecs: &mut Self::Vecs, value: Self);

    fn aos_std_push(vec: &mut Vec<Self>, value: Self) {
        let value = black_box(value);
        vec.push(value);
    }
}

impl Push for Zero {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let value = black_box(value);
        vecs.push(value);
    }
}

impl Push for Tiny {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (values,) = vecs;
        let (value,) = black_box(value);
        values.push(value);
    }
}

impl Push for Small {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (xs, ys, zs) = vecs;
        let (x, y, z) = black_box(value);
        xs.push(x);
        ys.push(y);
        zs.push(z);
    }
}

impl Push for Medium {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (smalls1, smalls2, smalls3) = vecs;
        let (small1, small2, small3) = black_box(value);
        smalls1.push(small1);
        smalls2.push(small2);
        smalls3.push(small3);
    }
}

impl Push for Big {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
        let (smalls1, smalls2, arrays, strs1, strs2) = vecs;
        let (small1, small2, array, str1, str2) = black_box(value);
        smalls1.push(small1);
        smalls2.push(small2);
        arrays.push(array);
        strs1.push(str1);
        strs2.push(str2);
    }
}

impl Push for Large {
    fn soa_std_push(vecs: &mut Self::Vecs, value: Self) {
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
        let (array1, array2, array3, array4, array5, array6, array7, array8, array9, array10) =
            black_box(value);
        arrays1.push(array1);
        arrays2.push(array2);
        arrays3.push(array3);
        arrays4.push(array4);
        arrays5.push(array5);
        arrays6.push(array6);
        arrays7.push(array7);
        arrays8.push(array8);
        arrays9.push(array9);
        arrays10.push(array10);
    }
}
