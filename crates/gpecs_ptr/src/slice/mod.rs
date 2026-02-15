pub use self::{
    prim::CoreSliceItemPtrs,
    traits::{
        CastConstPtr, CastMutPtr, ConstSliceItemPtr, MutSliceItemPtr, NonNullAsPtr,
        NonNullSliceItemPtr, SliceItemPtr, SliceItemPtrs,
    },
};

mod prim;
mod traits;
