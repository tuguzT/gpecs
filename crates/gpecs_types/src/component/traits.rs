pub trait Component: 'static {}

pub trait GpuComponent: Component + Copy {}
