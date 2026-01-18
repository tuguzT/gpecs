use gpecs_soa_erased::soa::prelude::*;

use crate::{Big, Large, Medium, Small, Tiny, Zero};

pub trait SoaVecs: SoaOwned + AllocSoa {
    type Vecs;
}

impl SoaVecs for Zero {
    type Vecs = Vec<Self>;
}

impl SoaVecs for Tiny {
    type Vecs = (Vec<u32>,);
}

impl SoaVecs for Small {
    type Vecs = (Vec<f64>, Vec<f64>, Vec<f64>);
}

impl SoaVecs for Medium {
    type Vecs = (Vec<Small>, Vec<Small>, Vec<Small>);
}

impl SoaVecs for Big {
    type Vecs = (
        Vec<Small>,
        Vec<Small>,
        Vec<[usize; 18]>,
        Vec<String>,
        Vec<String>,
    );
}

impl SoaVecs for Large {
    type Vecs = (
        Vec<[u32; 32]>,
        Vec<[u32; 32]>,
        Vec<[u32; 32]>,
        Vec<[u32; 32]>,
        Vec<[u32; 32]>,
        Vec<[u32; 32]>,
        Vec<[u32; 32]>,
        Vec<[u32; 32]>,
        Vec<[u32; 32]>,
        Vec<[u32; 32]>,
    );
}
