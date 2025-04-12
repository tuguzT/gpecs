use crate::component::registry::ComponentId;

#[cold]
#[track_caller]
#[inline(never)]
pub fn get_component_info_fail(component_id: &ComponentId) -> ! {
    panic!("info of component {component_id:?} should be present")
}
