use std::ptr;

use gpecs_archetype::bundle::Bundle;
use gpecs_component::erased::{
    ErasedComponentPtr,
    error::{DowncastErrorKind, NotRegisteredError, TryFromPtrError},
};

use crate::common::{Components, Name, Position, Tag};

#[test]
fn ptrs_from_erased() {
    let mut components = Components::new();

    let position = Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        padding: Default::default(),
    };

    let error =
        ErasedComponentPtr::<*const u8>::try_from(&components.as_view(), ptr::from_ref(&position))
            .expect_err("`Position` component should not be registered yet");
    assert_eq!(
        error,
        TryFromPtrError::NotRegistered(NotRegisteredError::of::<Position>()),
    );

    let _position_id = components.register_component::<Position>();
    let erased_position_ptr =
        ErasedComponentPtr::<*const u8>::try_from(&components.as_view(), ptr::from_ref(&position))
            .expect("pointer of `Position` component should be created successfully");

    let erased_ptrs = [erased_position_ptr; 0];
    let error = <(Position,)>::ptrs_from_erased(&components.as_view(), erased_ptrs)
        .expect_err("there should not be enough pointers provided");
    assert!(matches!(
        error,
        DowncastErrorKind::ComponentNotRegistered(_),
    ));

    let erased_ptrs = [erased_position_ptr];
    let (position_ptr,) = <(Position,)>::ptrs_from_erased(&components.as_view(), erased_ptrs)
        .expect("pointer for an archetype of only `Position` should be created successfully");
    assert_eq!(position_ptr, &position, "pointers should be equal");
    let position_ref = unsafe { position_ptr.as_ref_unchecked() };
    assert_eq!(position_ref, &position, "positions should be equal");

    let erased_ptrs = [erased_position_ptr; 2];
    let (position_ptr,) = <(Position,)>::ptrs_from_erased(&components.as_view(), erased_ptrs)
        .expect("pointer for an archetype of only `Position` should be created successfully");
    assert_eq!(position_ptr, &position, "pointers should be equal");
    let position_ref = unsafe { position_ptr.as_ref_unchecked() };
    assert_eq!(position_ref, &position, "positions should be equal");

    let erased_ptrs = [erased_position_ptr];
    // TODO: this should return another error!
    let error = <(Position, Position)>::ptrs_from_erased(&components.as_view(), erased_ptrs)
        .expect_err("there should not be enough pointers provided");
    assert!(matches!(
        error,
        DowncastErrorKind::ComponentNotRegistered(_),
    ));

    let erased_ptrs = [erased_position_ptr; 2];
    // TODO: this should return an error!
    let (position_ptr, other_position_ptr) =
        <(Position, Position)>::ptrs_from_erased(&components.as_view(), erased_ptrs).unwrap();
    assert_eq!(position_ptr, &position, "pointers should be equal");
    assert_eq!(position_ptr, other_position_ptr);
    let position_ref = unsafe { position_ptr.as_ref_unchecked() };
    assert_eq!(position_ref, &position, "positions should be equal");

    let tag = Tag;

    let erased_ptrs = [erased_position_ptr];
    let error = <(Tag,)>::ptrs_from_erased(&components.as_view(), erased_ptrs)
        .expect_err("pointer for an archetype of only `Tag` should be created successfully");
    assert_eq!(
        error,
        DowncastErrorKind::ComponentNotRegistered(NotRegisteredError::of::<Tag>()),
    );

    let _tag_id = components.register_component::<Tag>();
    let erased_tag_ptr =
        ErasedComponentPtr::<*const u8>::try_from(&components.as_view(), ptr::from_ref(&tag))
            .expect("pointer of `Tag` component should be created successfully");

    let erased_ptrs = [erased_tag_ptr];
    let (tag_ptr,) = <(Tag,)>::ptrs_from_erased(&components.as_view(), erased_ptrs)
        .expect("pointer for an archetype of only `Position` should be created successfully");
    assert_eq!(tag_ptr, &tag, "pointers should be equal");
    let tag_ref = unsafe { tag_ptr.as_ref_unchecked() };
    assert_eq!(tag_ref, &tag, "tags should be equal");

    let erased_ptrs = [erased_position_ptr, erased_tag_ptr];
    let (tag_ptr, position_ptr) =
        <(Tag, Position)>::ptrs_from_erased(&components.as_view(), erased_ptrs).expect(
            "pointers for an archetype of `Position` and `Tag` should be created successfully",
        );
    assert_eq!(position_ptr, &position, "pointers should be equal");
    assert_eq!(tag_ptr, &tag, "pointers should be equal");
    let (position_ref, tag_ref) =
        unsafe { (position_ptr.as_ref_unchecked(), tag_ptr.as_ref_unchecked()) };
    assert_eq!(
        (&position, &tag),
        (position_ref, tag_ref),
        "positions & tags should be equal",
    );

    let name = Name {
        value: "Hello World".to_owned(),
    };
    let _name_id = components.register_component::<Name>();

    let erased_name_ptr =
        ErasedComponentPtr::<*const u8>::try_from(&components.as_view(), ptr::from_ref(&name))
            .expect("pointer of `Name` component should be created successfully");

    let erased_ptrs = [erased_name_ptr];
    let (name_ptr,) = <(Name,)>::ptrs_from_erased(&components.as_view(), erased_ptrs)
        .expect("pointer for an archetype of only `Name` should be created successfully");
    assert_eq!(name_ptr, &name, "pointers should be equal");
    let name_ref = unsafe { name_ptr.as_ref_unchecked() };
    assert_eq!(name_ref, &name, "names should be equal");

    let erased_ptrs = [erased_position_ptr, erased_name_ptr, erased_tag_ptr];
    let (tag_ptr, position_ptr, name_ptr) =
        <(Tag, Position, Name)>::ptrs_from_erased(&components.as_view(), erased_ptrs)
            .expect("pointers for an archetype should be created successfully");
    assert_eq!(position_ptr, &position, "pointers should be equal");
    assert_eq!(name_ptr, &name, "pointers should be equal");
    assert_eq!(tag_ptr, &tag, "pointers should be equal");
    let (position_ref, name_ref, tag_ref) = (
        unsafe { position_ptr.as_ref_unchecked() },
        unsafe { name_ptr.as_ref_unchecked() },
        unsafe { tag_ptr.as_ref_unchecked() },
    );
    assert_eq!(
        (&position, &name, &tag),
        (position_ref, name_ref, tag_ref),
        "components should be equal",
    );

    // TODO: this should return an error!
    let (position_ptr, name_ptr, other_position_ptr, tag_ptr, other_tag_ptr) =
        <(Position, Name, Position, Tag, Tag)>::ptrs_from_erased(
            &components.as_view(),
            [
                erased_position_ptr,
                erased_name_ptr,
                erased_position_ptr,
                erased_tag_ptr,
                erased_tag_ptr,
            ],
        )
        .expect("pointers for an archetype should be created successfully");
    assert_eq!(position_ptr, &position, "pointers should be equal");
    assert_eq!(position_ptr, other_position_ptr);
    assert_eq!(name_ptr, &name, "pointers should be equal");
    assert_eq!(tag_ptr, &tag, "pointers should be equal");
    assert_eq!(tag_ptr, other_tag_ptr);
}
