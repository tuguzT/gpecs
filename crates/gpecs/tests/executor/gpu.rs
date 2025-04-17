use gpecs::prelude::*;

#[test]
fn execute_simple() {
    let mut context = Context::new();
    let mut executor = GpuExecutor::new(&mut context);

    executor.execute();
}
