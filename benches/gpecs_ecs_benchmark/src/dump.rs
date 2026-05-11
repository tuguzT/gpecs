use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use gpecs_ecs_benchmark_types::framebuffer::Framebuffer;

pub fn dump_framebuffer_into_file<B>(
    framebuffer: &Framebuffer<B>,
    group: &str,
    index: u128,
) -> io::Result<()>
where
    B: AsRef<[u32]>,
{
    let path = format!("./dump/{group}/framebuffer-{index}.txt");
    let path = Path::new(&path);

    let prefix = path.parent().expect("path should have a parent directory");
    fs::create_dir_all(prefix)?;

    let framebuffer_file = File::create(path)?;
    dump_framebuffer(framebuffer, framebuffer_file)
}

pub fn dump_framebuffer<B, W>(framebuffer: &Framebuffer<B>, mut writer: W) -> io::Result<()>
where
    B: AsRef<[u32]>,
    W: Write,
{
    let chunk_size = framebuffer
        .desc()
        .width
        .try_into()
        .expect("framebuffer width should fit into `u32`");
    for chunk in framebuffer.buffer().as_ref().chunks_exact(chunk_size) {
        for &char in chunk {
            let char = u8::try_from(char).expect("failed to convert character to `u8`");
            assert!(char.is_ascii(), "character should be ASCII");
            writer.write_all(&[char])?;
        }
        writer.write_all(b"\n")?;
    }
    Ok(())
}
