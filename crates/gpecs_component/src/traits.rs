pub use bytemuck::NoUninit;

pub trait Component: 'static {}

pub trait GpuComponent: Component + NoUninit + Send + Sync {}
