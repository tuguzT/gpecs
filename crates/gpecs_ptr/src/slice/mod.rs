pub use self::{
    prim::CoreSliceItemPtrs,
    traits::{
        ConstPtr, ConstSliceItemPtr, MutPtr, MutSliceItemPtr, NonNullPtr, NonNullSliceItemPtr,
        PtrsItem, SliceItemPtr, SliceItemPtrs,
    },
};

mod prim;
mod traits;
