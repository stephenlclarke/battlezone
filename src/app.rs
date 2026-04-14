use std::{
    io::{Write, stdout},
    time::{Duration, Instant},
};

use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEventKind},
    queue,
    terminal::{Clear, ClearType},
};

use crate::{
    game::{Game, InputState},
    kitty::KittyGraphics,
    render::Renderer,
    terminal::{TerminalSession, geometry},
};

const FRAME_TIME: Duration = Duration::from_millis(33);
const MAX_DT: f32 = 0.1;

pub fn run() -> Result<()> {
    KittyGraphics::ensure_supported()?;

    let mut stdout = stdout();
    let _session = TerminalSession::enter(&mut stdout)?;
    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    stdout.flush()?;

    let mut terminal_geometry = geometry()?;
    let mut renderer = Renderer::new(terminal_geometry);
    let mut graphics = KittyGraphics::new(terminal_geometry.cols, terminal_geometry.rows);
    let mut game = Game::new();
    let mut last_tick = Instant::now();

    loop {
        let frame_started = Instant::now();
        let latest_geometry = geometry()?;
        if latest_geometry != terminal_geometry {
            terminal_geometry = latest_geometry;
            renderer.resize(terminal_geometry);
            graphics.resize(terminal_geometry.cols, terminal_geometry.rows);
        }

        let input = poll_input()?;
        if input.quit {
            break;
        }

        let dt = last_tick.elapsed().as_secs_f32().min(MAX_DT);
        last_tick = Instant::now();
        game.update(dt, input);

        let frame = game.frame();
        let image = renderer.render(&frame);
        graphics.draw_frame(&mut stdout, &image)?;
        draw_hud(&mut stdout, terminal_geometry.rows, &frame.hud)?;
        stdout.flush()?;

        let elapsed = frame_started.elapsed();
        if elapsed < FRAME_TIME {
            std::thread::sleep(FRAME_TIME - elapsed);
        }
    }

    graphics.clear(&mut stdout)?;
    stdout.flush()?;

    Ok(())
}

fn poll_input() -> Result<InputState> {
    let mut input = InputState::default();

    while event::poll(Duration::ZERO)? {
        match event::read()? {
            Event::Key(key_event)
                if matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
            {
                match key_event.code {
                    KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => input.forward = true,
                    KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                        input.backward = true
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => {
                        input.turn_left = true
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right => {
                        input.turn_right = true
                    }
                    KeyCode::Char(' ') => input.fire = true,
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => input.quit = true,
                    _ => {}
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    Ok(input)
}

fn draw_hud(stdout: &mut std::io::Stdout, rows: u16, hud: &[String]) -> Result<()> {
    let visible_lines = hud.len().min(rows as usize);
    let start_row = rows.saturating_sub(visible_lines as u16);

    for (offset, line) in hud.iter().take(visible_lines).enumerate() {
        queue!(
            stdout,
            MoveTo(0, start_row + offset as u16),
            Clear(ClearType::CurrentLine)
        )?;
        write!(stdout, "{line}")?;
    }

    Ok(())
}
