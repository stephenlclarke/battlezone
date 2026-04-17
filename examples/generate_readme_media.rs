//! Generates the README screenshot and animated attract preview.

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

const GIF_FRAME_DELAY_CS: u16 = 15;
const GIF_DURATION_SECONDS: f32 = 4.0;
const GIF_CAPTURE_STEP: f32 = 0.15;

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
    write_attract_gif(&gif_path, &mut game, &mut gif_renderer)?;

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
    for _ in 0..75 {
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

fn write_attract_gif(path: &Path, game: &mut Game, renderer: &mut Renderer) -> Result<()> {
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

    let frames = (GIF_DURATION_SECONDS / GIF_CAPTURE_STEP) as usize;
    let advance_steps = (GIF_CAPTURE_STEP / ORIGINAL_FRAME_TIME).round() as usize;
    for _ in 0..frames {
        let image = renderer.render(&game.frame());
        let mut pixels = image.pixels.clone();
        let mut frame =
            Frame::from_rgba_speed(image.width as u16, image.height as u16, &mut pixels, 10);
        frame.delay = GIF_FRAME_DELAY_CS;
        encoder.write_frame(&frame).context("writing gif frame")?;
        for _ in 0..advance_steps {
            game.update_with_input(ORIGINAL_FRAME_TIME, UpdateInput::default());
        }
    }
    Ok(())
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
