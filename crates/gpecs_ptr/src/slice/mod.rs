pub use self::{
    prim::CoreSliceItemPtrs,
    traits::{
        CastConst, CastMut, ConstPtr, ConstSliceItemPtr, MutPtr, MutSliceItemPtr, NonNullAsMutPtr,
        NonNullAsPtr, NonNullPtr, NonNullSliceItemPtr, PtrsItem, SliceItemPtr, SliceItemPtrs,
    },
};

mod prim;
mod traits;
