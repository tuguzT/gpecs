use std::{
    error::Error,
    io, panic,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use crossterm::{
    ExecutableCommand, QueueableCommand, SynchronizedUpdate, cursor, event, style, terminal,
};
use gpecs::prelude::*;
use gpecs_ecs_benchmark_core::gpu;
use gpecs_ecs_benchmark_types::{components::NONE_SPRITE, framebuffer::Framebuffer};

fn main() -> Result<(), Box<dyn Error>> {
    let is_running = Arc::new(AtomicBool::new(true));

    let is_running_clone = is_running.clone();
    let join_handle = thread::spawn(move || -> io::Result<()> {
        loop {
            if event::poll(Duration::from_millis(100))?
                && let event::Event::Key(key_event) = event::read()?
                && key_event.code == event::KeyCode::Char('c')
                && key_event.modifiers.contains(event::KeyModifiers::CONTROL)
            {
                is_running_clone.store(false, Ordering::SeqCst);
                break Ok(());
            }
        }
    });

    let (columns, rows) = terminal::size()?;

    let framebuffer_width = u32::from(columns);
    let framebuffer_height = u32::from(rows);
    let framebuffer_size = (framebuffer_width * framebuffer_height).try_into()?;
    let framebuffer = Framebuffer::new(
        framebuffer_width,
        framebuffer_height,
        vec![NONE_SPRITE; framebuffer_size],
    );

    let entity_count = framebuffer_width * framebuffer_height;
    let spawn_area_margin = u32::max(framebuffer_width, framebuffer_height) / 2;

    let mut stdout = io::stdout().lock();

    terminal::enable_raw_mode()?;
    stdout
        .execute(cursor::Hide)?
        .execute(terminal::EnterAlternateScreen)?
        .execute(terminal::DisableLineWrap)?;

    let context = &mut Context::new();
    let render = |_, _, _, framebuffer: &Framebuffer<Vec<_>>| -> Result<(), Option<io::Error>> {
        stdout
            .queue(cursor::MoveTo(0, 0))?
            .queue(terminal::Clear(terminal::ClearType::All))?;
        let operations = |stdout: &mut io::StdoutLock<'_>| -> io::Result<()> {
            let chunk_size = usize::from(columns);
            for chunk in framebuffer.buffer().chunks_exact(chunk_size) {
                for &char in chunk {
                    let char = u8::try_from(char).expect("failed to convert character to `u8`");
                    assert!(char.is_ascii(), "character should be ASCII");
                    stdout.queue(style::Print(char::from(char)))?;
                }
                stdout.queue(cursor::MoveToNextLine(1))?;
            }
            Ok(())
        };
        stdout.sync_update(operations).flatten()?;

        if !is_running.load(Ordering::SeqCst) {
            return Err(None);
        }
        Ok(())
    };
    let result = gpu::run(
        context,
        entity_count,
        None,
        framebuffer,
        spawn_area_margin,
        render,
    );
    if let Err(Some(error)) = result {
        return Err(error.into());
    }

    stdout
        .execute(terminal::EnableLineWrap)?
        .execute(terminal::LeaveAlternateScreen)?
        .execute(cursor::Show)?;
    terminal::disable_raw_mode()?;

    join_handle
        .join()
        .unwrap_or_else(|payload| panic::resume_unwind(payload))?;

    Ok(())
}
