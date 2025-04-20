use crate::{
    components::{Position, Sprite},
    framebuffer::Framebuffer,
};

pub fn render_sprite(position: &Position, sprite: &Sprite, framebuffer: &mut Framebuffer) {
    framebuffer.draw(position.x as i32, position.y as i32, sprite.character);
}
