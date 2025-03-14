pub mod registry;

pub trait Component: 'static {}

impl Component for () {}
