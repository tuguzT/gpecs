pub use self::{
    prim::CoreSliceItemPtrs,
    traits::{
        CastConst, CastMut, ConstPtr, ConstSliceItemPtr, MutPtr, MutSliceItemPtr, NonNullAsPtr,
        NonNullPtr, NonNullSliceItemPtr, SliceItemPtr, SliceItemPtrs,
    },
};

mod prim;
mod traits;
