use num_traits::ToPrimitive;

use crate::{
    components::{Position, Sprite},
    framebuffer::Framebuffer,
};

pub fn render_sprite<B>(position: &Position, sprite: &Sprite, framebuffer: &mut Framebuffer<B>)
where
    B: AsMut<[u32]>,
{
    framebuffer.draw(
        position.x.to_i32().unwrap(),
        position.y.to_i32().unwrap(),
        sprite.character,
    );
}
