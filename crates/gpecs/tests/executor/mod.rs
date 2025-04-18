use gpecs::prelude::*;

#[test]
fn execute_simple() {
    let mut context = Context::new();
    let mut executor = CpuExecutor::new(&mut context);

    let system1 = executor.register_system(|| println!("Hello from the simple system!"));
    let system2 = executor.register_system(|| unreachable!());

    let added = executor.add_system(system2);
    assert!(added, "system {system2:?} should not be scheduled yet");

    let added = executor.add_system(system1);
    assert!(added, "system {system1:?} should not be scheduled yet");

    let added = executor.add_system(system2);
    assert!(!added, "system {system2:?} should be scheduled before");

    let removed = executor.remove_system(system2);
    assert!(removed, "system {system2:?} should be scheduled before");

    let removed = executor.remove_system(system2);
    assert!(!removed, "system {system2:?} should not be scheduled yet");

    executor.execute();
}
