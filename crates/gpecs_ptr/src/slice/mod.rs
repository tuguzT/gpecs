pub use self::{
    prim::CoreSliceItemPtrs,
    traits::{
        CastConst, CastMut, ConstSliceItemPtr, MutSliceItemPtr, NonNullAsPtr, NonNullSliceItemPtr,
        SliceItemPtr, SliceItemPtrs,
    },
};

mod prim;
mod traits;
