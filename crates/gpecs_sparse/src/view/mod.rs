pub use self::{view::EpochSparseView, view_mut::EpochSparseViewMut, view_ptr::EpochSparseViewPtr};

pub type SparseViewPtr<'c, T> = EpochSparseViewPtr<'c, usize, T>;

pub type SparseView<'c, 'a, T> = EpochSparseView<'c, 'a, usize, T>;
pub type SparseViewMut<'c, 'a, T> = EpochSparseViewMut<'c, 'a, usize, T>;

mod assert;
mod view;
mod view_mut;
mod view_ptr;
