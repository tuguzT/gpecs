use crate::component::Component;

pub mod registry;

pub trait GpuComponent: Component + Copy {}
