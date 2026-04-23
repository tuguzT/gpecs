use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use gpecs_ecs_benchmark_types::framebuffer::Framebuffer;

use crate::FRAMEBUFFER_WIDTH;

pub fn save_framebuffer_to_file<B>(framebuffer: &Framebuffer<B>, path: &str, index: usize)
where
    B: AsRef<[u32]>,
{
    let path = format!("./dump/{path}/{index}.txt");
    let path = Path::new(&path);

    let prefix = path.parent().expect("failed to get parent directory");
    fs::create_dir_all(prefix).expect("failed to create parent directory");

    let mut framebuffer_file = File::create(path).expect("failed to create framebuffer file");
    for chunk in framebuffer
        .buffer()
        .as_ref()
        .chunks_exact(FRAMEBUFFER_WIDTH)
    {
        for &char in chunk {
            let char = u8::try_from(char).expect("failed to convert character to `u8`");
            assert!(char.is_ascii(), "character should be ASCII");
            framebuffer_file
                .write_all(&[char])
                .expect("failed to write character to file");
        }
        framebuffer_file
            .write_all(b"\n")
            .expect("failed to write newline to file");
    }
}
