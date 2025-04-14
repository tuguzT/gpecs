use crate::component::registry::ComponentId;

use super::error::{ComponentNotRegisteredError, DuplicateComponentError, GetComponentsError};

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

#[inline]
pub fn try_collect_maybe_component_ids<S, I, F>(
    component_ids: I,
    mut insert_fn: F,
) -> Result<S, GetComponentsError>
where
    S: Default,
    I: IntoIterator<Item = Option<ComponentId>>,
    F: FnMut(&mut S, ComponentId) -> bool,
{
    let mut set = S::default();
    component_ids
        .into_iter()
        .try_for_each::<_, Result<_, GetComponentsError>>(|component_id| {
            let Some(component_id) = component_id else {
                return Err(ComponentNotRegisteredError.into());
            };
            let is_unique = insert_fn(&mut set, component_id);
            is_unique
                .then(Default::default)
                .ok_or_else(|| DuplicateComponentError::new(component_id).into())
        })?;
    Ok(set)
}
