//! Generates the README screenshot and animated attract/gameplay preview.

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use battlezone::{
    arcade::ORIGINAL_FRAME_TIME,
    game::Game,
    input::UpdateInput,
    render::{RenderedImage, Renderer},
    terminal::TerminalGeometry,
};
use gif::{Encoder, Frame, Repeat};
use png::{BitDepth, ColorType, Compression, Encoder as PngEncoder};

const GIF_FRAME_DELAY_CS: u16 = 20;
const GIF_CAPTURE_STEP: f32 = 0.2;
const GIF_TITLE_HOLD_DELAY_CS: u16 = 70;
const GIF_FULL_ATTRACT_CYCLE_SECONDS: f32 = 11.6;
const GIF_RETURN_TO_TITLE_SECONDS: f32 = 2.8;
const GIF_GAMEPLAY_READY_SECONDS: f32 = 2.2;
const GIF_GAMEPLAY_SHOWCASE_SECONDS: f32 = 10.8;

fn main() -> Result<()> {
    let screenshot_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("docs/battlezone.png"));
    let gif_path = std::env::args_os()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("docs/start-sequence.gif"));

    ensure_parent_dir(&screenshot_path)?;
    ensure_parent_dir(&gif_path)?;

    let screenshot_geometry = TerminalGeometry {
        cols: 96,
        rows: 40,
        pixel_width: 960,
        pixel_height: 720,
    };
    let gif_geometry = TerminalGeometry {
        cols: 72,
        rows: 30,
        pixel_width: 640,
        pixel_height: 480,
    };
    let mut screenshot_renderer = Renderer::new(screenshot_geometry);
    let mut game = Game::with_seed(0xBADD1E);
    game.set_viewport(
        screenshot_renderer.image_width(),
        screenshot_renderer.image_height(),
    );

    let screenshot = capture_gameplay_screenshot(&mut game, &mut screenshot_renderer);
    write_png(&screenshot_path, &screenshot)?;

    let mut gif_renderer = Renderer::new(gif_geometry);
    let mut game = Game::with_seed(0xBADD1E);
    game.set_viewport(gif_renderer.image_width(), gif_renderer.image_height());
    write_showcase_gif(&gif_path, &mut game, &mut gif_renderer)?;

    println!("wrote {}", screenshot_path.display());
    println!("wrote {}", gif_path.display());
    Ok(())
}

fn capture_gameplay_screenshot(game: &mut Game, renderer: &mut Renderer) -> RenderedImage {
    game.update_with_input(
        ORIGINAL_FRAME_TIME,
        UpdateInput {
            start_requested: true,
            ..UpdateInput::default()
        },
    );
    for _ in 0..84 {
        game.update_with_input(ORIGINAL_FRAME_TIME, UpdateInput::default());
    }
    for _ in 0..60 {
        game.update_with_input(
            ORIGINAL_FRAME_TIME,
            UpdateInput {
                forward: true,
                turn_right: true,
                ..UpdateInput::default()
            },
        );
    }
    renderer.render(&game.frame())
}

fn write_showcase_gif(path: &Path, game: &mut Game, renderer: &mut Renderer) -> Result<()> {
    let file = File::create(path).with_context(|| format!("creating gif {}", path.display()))?;
    let mut encoder = Encoder::new(
        file,
        renderer.image_width() as u16,
        renderer.image_height() as u16,
        &[],
    )
    .with_context(|| format!("creating gif encoder for {}", path.display()))?;
    encoder
        .set_repeat(Repeat::Infinite)
        .context("setting gif repeat mode")?;

    capture_frame(&mut encoder, renderer, game, GIF_TITLE_HOLD_DELAY_CS)?;
    advance_for(
        &mut encoder,
        renderer,
        game,
        GIF_FULL_ATTRACT_CYCLE_SECONDS,
        |_| UpdateInput::default(),
    )?;
    advance_for(
        &mut encoder,
        renderer,
        game,
        GIF_RETURN_TO_TITLE_SECONDS,
        |_| UpdateInput::default(),
    )?;

    game.update_with_input(
        ORIGINAL_FRAME_TIME,
        UpdateInput {
            start_requested: true,
            ..UpdateInput::default()
        },
    );
    capture_frame(&mut encoder, renderer, game, GIF_FRAME_DELAY_CS)?;
    advance_for(
        &mut encoder,
        renderer,
        game,
        GIF_GAMEPLAY_READY_SECONDS,
        |_| UpdateInput::default(),
    )?;
    advance_for(
        &mut encoder,
        renderer,
        game,
        GIF_GAMEPLAY_SHOWCASE_SECONDS,
        readme_gameplay_input,
    )?;
    Ok(())
}

fn advance_for<F>(
    encoder: &mut Encoder<File>,
    renderer: &mut Renderer,
    game: &mut Game,
    duration: f32,
    mut input_for_time: F,
) -> Result<()>
where
    F: FnMut(f32) -> UpdateInput,
{
    let frames = (duration / GIF_CAPTURE_STEP).ceil() as usize;
    let mut elapsed = 0.0;
    for _ in 0..frames {
        advance_game(game, GIF_CAPTURE_STEP, input_for_time(elapsed));
        capture_frame(encoder, renderer, game, GIF_FRAME_DELAY_CS)?;
        elapsed += GIF_CAPTURE_STEP;
    }
    Ok(())
}

fn advance_game(game: &mut Game, duration: f32, input: UpdateInput) {
    let mut remaining = duration;
    while remaining > f32::EPSILON {
        let dt = remaining.min(ORIGINAL_FRAME_TIME);
        game.update_with_input(dt, input.clone());
        remaining -= dt;
    }
}

fn capture_frame(
    encoder: &mut Encoder<File>,
    renderer: &mut Renderer,
    game: &Game,
    delay_cs: u16,
) -> Result<()> {
    let image = renderer.render(&game.frame());
    let mut pixels = image.pixels.clone();
    let mut frame =
        Frame::from_rgba_speed(image.width as u16, image.height as u16, &mut pixels, 10);
    frame.delay = delay_cs;
    encoder.write_frame(&frame).context("writing gif frame")?;
    Ok(())
}

fn readme_gameplay_input(elapsed: f32) -> UpdateInput {
    let mut input = UpdateInput {
        left_tread_forward: true,
        right_tread_forward: true,
        ..UpdateInput::default()
    };

    match elapsed {
        0.0..2.4 => input.right_tread_backward = true,
        2.4..4.6 => {}
        4.6..7.4 => input.left_tread_backward = true,
        7.4..9.8 => input.right_tread_backward = true,
        _ => {}
    }

    if input.left_tread_backward || input.right_tread_backward {
        input.right_tread_forward = !input.right_tread_backward;
        input.left_tread_forward = !input.left_tread_backward;
    }

    const FIRE_TIMES: [f32; 4] = [2.4, 4.8, 7.2, 9.8];
    if FIRE_TIMES
        .iter()
        .any(|&time| (elapsed - time).abs() < GIF_CAPTURE_STEP * 0.3)
    {
        input.fire = true;
    }

    input
}

fn write_png(path: &Path, image: &RenderedImage) -> Result<()> {
    let file = File::create(path).with_context(|| format!("creating png {}", path.display()))?;
    let mut encoder = PngEncoder::new(file, image.width, image.height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_compression(Compression::Fast);
    let mut writer = encoder.write_header().context("writing png header")?;
    writer
        .write_image_data(&image.pixels)
        .context("writing png data")?;
    Ok(())
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating directory {}", parent.display()))?;
    }
    Ok(())
}
