use crate::{bundle::error::DuplicateComponentError, prelude::ComponentId};

#[inline]
pub fn try_collect_component_ids<S, I, F>(
    component_ids: I,
    mut insert_fn: F,
) -> Result<S, DuplicateComponentError>
where
    S: Default,
    I: IntoIterator<Item = ComponentId>,
    F: FnMut(&mut S, ComponentId) -> bool,
{
    let mut set = S::default();
    for component_id in component_ids {
        let is_unique = insert_fn(&mut set, component_id);
        if is_unique {
            continue;
        }
        return Err(DuplicateComponentError::new(component_id));
    }
    Ok(set)
}
