pub use self::{
    prim::CoreSlicePtrsAtIndex,
    traits::{
        ConstPtr, ConstSlicePtrAtIndex, MutPtr, MutSlicePtrAtIndex, NonNullPtr,
        NonNullSlicePtrAtIndex, PtrsItem, SlicePtrAtIndex, SlicePtrsAtIndex,
    },
};

mod prim;
mod traits;
