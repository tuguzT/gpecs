use gpecs::{prelude::*, soa::identity::Identity};

use crate::common::{Mass, Position};

#[test]
fn register_data() {
    let mut context = Context::new();
    let mut executor = GpuExecutor::new(&mut context);

    let component_id = executor.register_component::<Position>();
    assert_eq!(component_id.into_inner(), 0);

    let archetype_id = executor
        .register_archetype::<Identity<Mass>>()
        .expect("archetype of just `Mass` should contain unique component ids");
    assert_eq!(archetype_id.into_inner(), 0);

    let component_id = executor
        .component_id::<Mass>()
        .expect("`Mass` component should be registered after registering archetype");
    assert_eq!(component_id.into_inner(), 1);

    executor.execute();
}
