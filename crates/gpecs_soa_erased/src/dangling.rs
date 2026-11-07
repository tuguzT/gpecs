use core::{alloc::Layout, num::NonZeroUsize};

use crate::{
    error::{InsufficientAlignError, check_sufficient_align},
    soa::layout::WithLayout,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dangling {
    pub addr: NonZeroUsize,
    pub capacity: usize,
}

pub fn dangling<D, A>(layouts: D) -> Result<Dangling, InsufficientAlignError>
where
    D: IntoIterator<Item: WithLayout>,
{
    let mut packed_size = 0;
    let addr = layouts
        .into_iter()
        .try_fold(NonZeroUsize::MIN, |max_align, item| {
            let layout = item.layout();
            check_sufficient_align(layout, Layout::new::<A>())?;

            // SAFETY: layout align is power of two, and so is non-null
            let align = unsafe { NonZeroUsize::new_unchecked(layout.align()) };
            let align = NonZeroUsize::max(max_align, align);

            packed_size += layout.size();
            Ok(align)
        })?;
    let capacity = match packed_size {
        0 => usize::MAX,
        _ => 0,
    };

    let dangling = Dangling { addr, capacity };
    Ok(dangling)
}
