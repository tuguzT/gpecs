use core::alloc::Layout;

use crate::{
    error::{InsufficientAlignError, check_sufficient_align},
    soa::layout::WithLayout,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dangling {
    pub addr: usize,
    pub capacity: usize,
}

pub fn dangling<D, A>(layouts: D) -> Result<Dangling, InsufficientAlignError>
where
    D: IntoIterator<Item: WithLayout>,
{
    let mut packed_size = 0;
    let addr = layouts.into_iter().try_fold(1, |max_align, item| {
        let layout = item.layout();
        check_sufficient_align(layout, Layout::new::<A>())?;

        packed_size += layout.size();
        Ok(usize::max(max_align, layout.align()))
    })?;
    let capacity = match packed_size {
        0 => usize::MAX,
        _ => 0,
    };

    let dangling = Dangling { addr, capacity };
    Ok(dangling)
}
