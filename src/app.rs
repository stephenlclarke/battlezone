//! Runs the terminal application loop, input polling, fixed-timestep updates, and frame presentation.

use std::{
    io::{Write, stdout},
    time::{Duration, Instant},
};

use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    queue,
    terminal::{Clear, ClearType},
};

use crate::{
    arcade::ORIGINAL_FRAME_TIME,
    audio::AudioManager,
    constants::MAX_DT,
    game::Game,
    input::{InputTracker, UpdateInput},
    kitty::KittyGraphics,
    render::Renderer,
    terminal::{TerminalSession, geometry},
};

pub fn run() -> Result<()> {
    KittyGraphics::ensure_supported()?;

    let mut stdout = stdout();
    let session = TerminalSession::enter(&mut stdout)?;
    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    stdout.flush()?;

    let mut terminal_geometry = geometry()?;
    let mut renderer = Renderer::new(terminal_geometry);
    let mut graphics = KittyGraphics::new(terminal_geometry.cols, terminal_geometry.rows);
    let mut game = Game::new();
    let mut audio = AudioManager::new();
    let mut input_tracker = InputTracker::new(session.keyboard_enhancement_supported());
    game.set_viewport(renderer.image_width(), renderer.image_height());
    for event in game.drain_events() {
        audio.handle_event(event);
    }

    let frame_time = ORIGINAL_FRAME_TIME;
    let frame_duration = Duration::from_secs_f32(frame_time);
    let mut accumulator = 0.0f32;
    let mut last_tick = Instant::now();

    loop {
        let frame_started = Instant::now();
        sync_terminal_geometry(
            &mut terminal_geometry,
            &mut renderer,
            &mut graphics,
            &mut game,
        )?;

        let input = input_tracker.poll()?;
        if input.quit_requested {
            break;
        }

        let dt = last_tick.elapsed().as_secs_f32().min(MAX_DT);
        last_tick = Instant::now();
        accumulator = (accumulator + dt).min(frame_time * 6.0);

        let mut step_input = input.clone();
        let mut updated = false;
        while accumulator >= frame_time {
            game.update_with_input(frame_time, step_input);
            for event in game.drain_events() {
                audio.handle_event(event);
            }
            accumulator -= frame_time;
            step_input = repeated_input(input.clone());
            updated = true;
        }

        if !updated {
            game.update_with_input(dt, input);
            for event in game.drain_events() {
                audio.handle_event(event);
            }
        }

        let scene = game.frame();
        let image = renderer.render(&scene);
        graphics.draw_frame(&mut stdout, &image)?;
        stdout.flush()?;

        let elapsed = frame_started.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }

    graphics.clear(&mut stdout)?;
    stdout.flush()?;

    Ok(())
}

fn sync_terminal_geometry(
    terminal_geometry: &mut crate::terminal::TerminalGeometry,
    renderer: &mut Renderer,
    graphics: &mut KittyGraphics,
    game: &mut Game,
) -> Result<()> {
    let latest_geometry = geometry()?;
    if latest_geometry != *terminal_geometry {
        *terminal_geometry = latest_geometry;
        renderer.resize(*terminal_geometry);
        graphics.resize(terminal_geometry.cols, terminal_geometry.rows);
        game.set_viewport(renderer.image_width(), renderer.image_height());
    }
    Ok(())
}

fn repeated_input(input: UpdateInput) -> UpdateInput {
    UpdateInput {
        forward: input.forward,
        backward: input.backward,
        turn_left: input.turn_left,
        turn_right: input.turn_right,
        left_tread_forward: input.left_tread_forward,
        left_tread_backward: input.left_tread_backward,
        right_tread_forward: input.right_tread_forward,
        right_tread_backward: input.right_tread_backward,
        ..UpdateInput::default()
    }
}
