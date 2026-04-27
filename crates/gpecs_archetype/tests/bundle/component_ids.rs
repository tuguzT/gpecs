#![cfg(feature = "alloc")]

use gpecs_archetype::{bundle::Bundle, erased::error::DuplicateComponentError};
use gpecs_component::erased::error::NotRegisteredError;

use crate::common::{Components, Name, Position, Tag};

#[test]
fn get_components() {
    let mut components = Components::new();

    let error = <(Position,)>::get_components(&components.as_view())
        .expect_err("`Position` component should not be registered yet");
    assert_eq!(error, NotRegisteredError::of::<Position>().into());

    let position_id = components.register_component::<Position>();
    let ids = <(Position,)>::get_components(&components.as_view())
        .expect("`Position` component should have already been registered");
    assert_eq!(ids, [position_id]);

    let error = <(Position, Position)>::get_components(&components.as_view())
        .expect_err("`Position` component was duplicated");
    assert_eq!(error, DuplicateComponentError::new(position_id).into());

    let error = <(Position, Tag)>::get_components(&components.as_view())
        .expect_err("`Tag` component should not be registered yet");
    assert_eq!(error, NotRegisteredError::of::<Tag>().into());

    let tag_id = components.register_component::<Tag>();
    let ids = <(Position, Tag)>::get_components(&components.as_view())
        .expect("`Tag` component should have already been registered");
    assert_eq!(ids, [tag_id, position_id]);

    let error = <(Position, Name, Tag)>::get_components(&components.as_view())
        .expect_err("`Name` component should not be registered yet");
    assert_eq!(error, NotRegisteredError::of::<Name>().into());

    let name_id = components.register_component::<Name>();
    let ids = <(Name, Position, Tag)>::get_components(&components.as_view())
        .expect("all the components should have already been registered");
    assert_eq!(ids, [tag_id, name_id, position_id]);

    let error = <(Position, Name, Position, Tag, Tag)>::get_components(&components.as_view())
        .expect_err("`Position` component was duplicated");
    assert_eq!(error, DuplicateComponentError::new(position_id).into());
}

#[test]
fn register_components() {
    let mut components = Components::new();

    let position_id = components.component_id::<Position>();
    assert_eq!(position_id, None);

    let ids = <(Position,)>::register_components(&mut components)
        .expect("archetype of only `Position` should have unique components");
    let position_id = components
        .component_id::<Position>()
        .expect("`Position` component should be already registered");
    assert_eq!(ids, [position_id]);

    let error = <(Position, Position)>::register_components(&mut components)
        .expect_err("`Position` component was duplicated");
    assert_eq!(error, DuplicateComponentError::new(position_id));

    let ids = <(Position, Tag)>::register_components(&mut components)
        .expect("archetype of `Position` & `Tag` should have unique components");
    let tag_id = components
        .component_id::<Tag>()
        .expect("`Tag` component should be already registered");
    assert_eq!(ids, [tag_id, position_id]);

    let name_id = components.register_component::<Name>();
    let ids = <(Name, Position, Tag)>::register_components(&mut components)
        .expect("archetype of `Name`, `Position` & `Tag` should have unique components");
    assert_eq!(ids, [tag_id, name_id, position_id]);

    let error = <(Position, Name, Position, Tag, Tag)>::register_components(&mut components)
        .expect_err("`Position` component was duplicated");
    assert_eq!(error, DuplicateComponentError::new(position_id));
}
