pub const ENTITY_COUNT: usize = 1_000_000;
pub const EXEC_COUNT: usize = 10;

pub const CPU_PATH: &str = "cpu";
pub const GPU_PATH: &str = "gpu";

pub const FRAMEBUFFER_WIDTH: usize = 320;
pub const FRAMEBUFFER_HEIGHT: usize = 240;
pub const FRAMEBUFFER_SIZE: usize = FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT;
pub const SPAWN_AREA_MARGIN: u32 = 100;

pub mod cpu;
pub mod gpu;
pub mod save;
pub mod setup;
