pub trait Component: 'static {}

pub trait GpuComponent: Component + bytemuck::NoUninit + Send + Sync {}
