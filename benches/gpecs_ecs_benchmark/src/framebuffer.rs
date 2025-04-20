#[derive(Debug)]
#[repr(C, align(16))]
pub struct Framebuffer<'a> {
    width: u32,
    height: u32,
    buffer: &'a mut [u32],
}

impl<'a> Framebuffer<'a> {
    pub fn new(width: u32, height: u32, buffer: &'a mut [u32]) -> Self {
        assert!(
            buffer.len() <= (width * height) as usize,
            "buffer is too small for the given width {width} and height {height}",
        );
        Self {
            width,
            height,
            buffer,
        }
    }

    pub fn draw(&mut self, x: i32, y: i32, char: u32) {
        let Self {
            width,
            height,
            ref mut buffer,
        } = *self;

        if y >= 0 && y < height as i32 {
            if x >= 0 && x < width as i32 {
                buffer[(x + y * width as i32) as usize] = char;
            }
        }
    }
}
