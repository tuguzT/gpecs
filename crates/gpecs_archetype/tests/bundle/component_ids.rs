#![cfg(feature = "alloc")]

use gpecs_archetype::bundle::Bundle;
use gpecs_component::erased::error::NotRegisteredError;

use crate::common::{Components, Name, Position, Tag};

#[test]
fn get_components() {
    let mut components = Components::new();

    let error = <(Position,)>::get_components(&components.as_view())
        .expect_err("`Position` component should not be registered yet");
    assert_eq!(error, NotRegisteredError::of::<Position>());

    let position_id = components.register_component::<Position>();
    let ids = <(Position,)>::get_components(&components.as_view())
        .expect("`Position` component should have already been registered");
    assert_eq!(ids, [position_id]);

    // TODO: this should return an error!
    let ids = <(Position, Position)>::get_components(&components.as_view())
        .expect("`Position` component should have already been registered");
    assert_eq!(ids, [position_id, position_id]);

    let error = <(Position, Tag)>::get_components(&components.as_view())
        .expect_err("`Tag` component should not be registered yet");
    assert_eq!(error, NotRegisteredError::of::<Tag>());

    let tag_id = components.register_component::<Tag>();
    let ids = <(Position, Tag)>::get_components(&components.as_view())
        .expect("`Tag` component should have already been registered");
    assert_eq!(ids, [tag_id, position_id]);

    let error = <(Position, Name, Tag)>::get_components(&components.as_view())
        .expect_err("`Name` component should not be registered yet");
    assert_eq!(error, NotRegisteredError::of::<Name>());

    let name_id = components.register_component::<Name>();
    let ids = <(Name, Position, Tag)>::get_components(&components.as_view())
        .expect("all the components should have already been registered");
    assert_eq!(ids, [tag_id, name_id, position_id]);

    // TODO: this should return an error!
    let ids = <(Position, Name, Position, Tag, Tag)>::get_components(&components.as_view())
        .expect("all the components should have already been registered");
    assert_eq!(ids, [tag_id, tag_id, name_id, position_id, position_id]);
}

#[test]
fn register_components() {
    let mut components = Components::new();

    let position_id = components.component_id::<Position>();
    assert_eq!(position_id, None);

    let ids = <(Position,)>::register_components(&mut components);
    let position_id = components
        .component_id::<Position>()
        .expect("`Position` component should be already registered");
    assert_eq!(ids, [position_id]);

    // TODO: this should return an error!
    let ids = <(Position, Position)>::register_components(&mut components);
    assert_eq!(ids, [position_id, position_id]);

    let ids = <(Position, Tag)>::register_components(&mut components);
    let tag_id = components
        .component_id::<Tag>()
        .expect("`Tag` component should be already registered");
    assert_eq!(ids, [tag_id, position_id]);

    let name_id = components.register_component::<Name>();
    let ids = <(Name, Position, Tag)>::register_components(&mut components);
    assert_eq!(ids, [tag_id, name_id, position_id]);

    // TODO: this should return an error!
    let ids = <(Position, Name, Position, Tag, Tag)>::register_components(&mut components);
    assert_eq!(ids, [tag_id, tag_id, name_id, position_id, position_id]);
}
