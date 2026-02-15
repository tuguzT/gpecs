use core::alloc::Layout;

use crate::{
    error::{InsufficientAlignError, check_sufficient_align},
    soa::field::{FieldDescriptor, IntoCopiedFieldDescriptors},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dangling {
    pub addr: usize,
    pub capacity: usize,
}

pub fn dangling<D, A>(descriptors: D) -> Result<Dangling, InsufficientAlignError>
where
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    let mut packed_size = 0;
    let addr = descriptors
        .copied_field_descriptors()
        .try_fold(1, |max_align, desc| {
            let layout = desc.layout();
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
