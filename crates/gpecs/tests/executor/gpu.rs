use gpecs::prelude::*;

#[test]
fn execute_simple() {
    let mut context = Context::new();
    let mut executor = GpuExecutor::new(&mut context);

    executor.execute();

    let _context = pollster::block_on(executor.into_context());
}
