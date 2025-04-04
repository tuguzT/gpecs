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
    component_ids.into_iter().try_for_each(|component_id| {
        let is_unique = insert_fn(&mut set, component_id);
        is_unique
            .then(Default::default)
            .ok_or_else(|| DuplicateComponentError::new(component_id))
    })?;
    Ok(set)
}
