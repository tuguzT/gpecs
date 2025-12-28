pub use self::{
    view::EpochSparseView, view_mut::EpochSparseViewMut, view_mut_ptr::EpochSparseViewMutPtr,
    view_ptr::EpochSparseViewPtr,
};

pub type SparseViewPtr<'ctx, T> = EpochSparseViewPtr<'ctx, usize, T>;
pub type SparseViewMutPtr<'ctx, T> = EpochSparseViewMutPtr<'ctx, usize, T>;

pub type SparseView<'ctx, 'a, T> = EpochSparseView<'ctx, 'a, usize, T>;
pub type SparseViewMut<'ctx, 'a, T> = EpochSparseViewMut<'ctx, 'a, usize, T>;

mod view;
mod view_mut;
mod view_mut_ptr;
mod view_ptr;
