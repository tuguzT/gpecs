pub mod cpu;
pub mod gpu;

pub trait Executor {
    fn execute(&mut self);
}
