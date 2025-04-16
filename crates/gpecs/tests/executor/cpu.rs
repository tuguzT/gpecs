use gpecs::prelude::*;

#[test]
fn new() {
    let mut context = Context::new();

    let mut executor = CpuExecutor::new(&mut context);
    executor.execute();
}
