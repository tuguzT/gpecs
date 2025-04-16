pub mod cpu;

pub trait Executor {
    fn execute(&mut self);
}
