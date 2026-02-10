pub use self::{
    gpu::{GpuSliceItemPtr, GpuSliceItemPtrs},
    prim::CoreSliceItemPtrs,
    traits::{
        CastConstPtr, CastMutPtr, ConstSliceItemPtr, MutSliceItemPtr, NonNullAsPtr,
        NonNullSliceItemPtr, SliceItemPtr, SliceItemPtrs,
    },
};

mod gpu;
mod prim;
mod traits;
