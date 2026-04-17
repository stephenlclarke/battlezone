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
    drain_audio_events(&mut game, &mut audio);

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

        let fixed_steps = consume_fixed_steps(&mut accumulator, frame_time);
        let updated = fixed_steps > 0;
        if updated {
            run_fixed_updates(
                &mut game,
                &mut audio,
                frame_time,
                fixed_steps,
                input.clone(),
            );
        }

        if !updated {
            game.update_with_input(dt, input);
            drain_audio_events(&mut game, &mut audio);
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

fn drain_audio_events(game: &mut Game, audio: &mut AudioManager) {
    for event in game.drain_events() {
        audio.handle_event(event);
    }
}

fn consume_fixed_steps(accumulator: &mut f32, frame_time: f32) -> usize {
    if frame_time <= 0.0 {
        return 0;
    }

    let steps = (*accumulator / frame_time).floor() as usize;
    *accumulator -= frame_time * steps as f32;
    steps
}

fn run_fixed_updates(
    game: &mut Game,
    audio: &mut AudioManager,
    frame_time: f32,
    fixed_steps: usize,
    input: UpdateInput,
) {
    let repeated = repeated_input(input.clone());
    let mut step_input = input;
    for _ in 0..fixed_steps {
        game.update_with_input(frame_time, step_input);
        drain_audio_events(game, audio);
        step_input = repeated.clone();
    }
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

#[cfg(test)]
mod tests {
    use super::{consume_fixed_steps, repeated_input};
    use crate::input::UpdateInput;

    #[test]
    fn consume_fixed_steps_returns_whole_frame_updates() {
        let mut accumulator = 0.19;
        let steps = consume_fixed_steps(&mut accumulator, 0.05);
        assert_eq!(steps, 3);
        assert!((accumulator - 0.04).abs() < 0.0001);
    }

    #[test]
    fn consume_fixed_steps_ignores_non_positive_frame_time() {
        let mut accumulator = 0.5;
        assert_eq!(consume_fixed_steps(&mut accumulator, 0.0), 0);
        assert_eq!(accumulator, 0.5);
    }

    #[test]
    fn repeated_input_preserves_only_movement_state() {
        let repeated = repeated_input(UpdateInput {
            forward: true,
            backward: true,
            turn_left: true,
            turn_right: true,
            left_tread_forward: true,
            left_tread_backward: true,
            right_tread_forward: true,
            right_tread_backward: true,
            fire: true,
            start_requested: true,
            quit_requested: true,
            autopilot_toggle_requested: true,
            initials_previous: true,
            initials_next: true,
            initials_confirm: true,
            typed_chars: vec!['x'],
        });

        assert!(repeated.forward);
        assert!(repeated.backward);
        assert!(repeated.turn_left);
        assert!(repeated.turn_right);
        assert!(repeated.left_tread_forward);
        assert!(repeated.left_tread_backward);
        assert!(repeated.right_tread_forward);
        assert!(repeated.right_tread_backward);
        assert!(!repeated.fire);
        assert!(!repeated.start_requested);
        assert!(!repeated.quit_requested);
        assert!(!repeated.autopilot_toggle_requested);
        assert!(!repeated.initials_previous);
        assert!(!repeated.initials_next);
        assert!(!repeated.initials_confirm);
        assert!(repeated.typed_chars.is_empty());
    }
}
