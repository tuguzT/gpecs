use std::error::Error;

use winit::event_loop::{ControlFlow, EventLoop};

use crate::app::App;

mod app;
mod graphics;
mod state;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    env_logger::builder().try_init()?;

    let event_loop = EventLoop::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app)?;

    Ok(())
}
