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
const SCREENSHOT_SHOWCASE_FRAMES: usize = 46;
const OUTPUT_WIDTH: u32 = 3456;
const OUTPUT_HEIGHT: u32 = 1864;
const WINDOW_X: u32 = 112;
const WINDOW_Y: u32 = 76;
const WINDOW_WIDTH: u32 = 3232;
const WINDOW_HEIGHT: u32 = 1650;
const WINDOW_RADIUS: u32 = 42;
const CONTENT_X: u32 = 160;
const CONTENT_Y: u32 = 204;
const CONTENT_WIDTH: u32 = 3135;
const CONTENT_HEIGHT: u32 = 1444;
const WINDOW_COLOR: [u8; 4] = [30, 30, 45, 255];
const WINDOW_SHADOW: [u8; 4] = [0, 0, 0, 52];
const CONTENT_BACKGROUND: [u8; 4] = [0, 0, 0, 255];
const TRAFFIC_RED: [u8; 4] = [255, 95, 87, 255];
const TRAFFIC_YELLOW: [u8; 4] = [254, 188, 46, 255];
const TRAFFIC_GREEN: [u8; 4] = [40, 200, 64, 255];
const GHOST_BADGE: [u8; 4] = [240, 240, 248, 255];
const GHOST_EYE: [u8; 4] = [18, 18, 26, 255];

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
        cols: 72,
        rows: 30,
        pixel_width: 640,
        pixel_height: 480,
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
    let screenshot = compose_windowed_screenshot(&screenshot);
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
    advance_game(game, GIF_FULL_ATTRACT_CYCLE_SECONDS, UpdateInput::default());
    advance_game(game, GIF_RETURN_TO_TITLE_SECONDS, UpdateInput::default());
    game.update_with_input(
        ORIGINAL_FRAME_TIME,
        UpdateInput {
            start_requested: true,
            ..UpdateInput::default()
        },
    );
    advance_game(game, GIF_GAMEPLAY_READY_SECONDS, UpdateInput::default());

    let mut elapsed = 0.0;
    for _ in 0..SCREENSHOT_SHOWCASE_FRAMES {
        advance_game(game, GIF_CAPTURE_STEP, readme_gameplay_input(elapsed));
        elapsed += GIF_CAPTURE_STEP;
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

fn write_png(path: &Path, image: &RgbaImage) -> Result<()> {
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

fn compose_windowed_screenshot(image: &RenderedImage) -> RgbaImage {
    let mut framed = RgbaImage::new(OUTPUT_WIDTH, OUTPUT_HEIGHT, [0, 0, 0, 0]);

    draw_rounded_rect(
        &mut framed,
        WINDOW_X + 18,
        WINDOW_Y + 26,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WINDOW_RADIUS,
        WINDOW_SHADOW,
    );
    draw_rounded_rect(
        &mut framed,
        WINDOW_X,
        WINDOW_Y,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WINDOW_RADIUS,
        WINDOW_COLOR,
    );
    fill_rect(
        &mut framed,
        CONTENT_X,
        CONTENT_Y,
        CONTENT_WIDTH,
        CONTENT_HEIGHT,
        CONTENT_BACKGROUND,
    );

    let button_y = WINDOW_Y + 34;
    draw_circle(&mut framed, WINDOW_X + 34, button_y, 14, TRAFFIC_RED);
    draw_circle(&mut framed, WINDOW_X + 82, button_y, 14, TRAFFIC_YELLOW);
    draw_circle(&mut framed, WINDOW_X + 130, button_y, 14, TRAFFIC_GREEN);
    draw_ghost_badge(&mut framed, WINDOW_X + WINDOW_WIDTH / 2, WINDOW_Y + 42, 2);

    blit_scaled_to_fit(
        image,
        &mut framed,
        CONTENT_X,
        CONTENT_Y,
        CONTENT_WIDTH,
        CONTENT_HEIGHT,
    );
    framed
}

fn fill_rect(image: &mut RgbaImage, x: u32, y: u32, width: u32, height: u32, color: [u8; 4]) {
    for py in y..y + height {
        for px in x..x + width {
            blend_pixel(image, px, py, color);
        }
    }
}

fn draw_circle(image: &mut RgbaImage, center_x: u32, center_y: u32, radius: u32, color: [u8; 4]) {
    let radius = radius as i32;
    let center_x = center_x as i32;
    let center_y = center_y as i32;

    for py in center_y - radius..=center_y + radius {
        for px in center_x - radius..=center_x + radius {
            let dx = px - center_x;
            let dy = py - center_y;
            if dx * dx + dy * dy <= radius * radius
                && let (Ok(px), Ok(py)) = (u32::try_from(px), u32::try_from(py))
            {
                blend_pixel(image, px, py, color);
            }
        }
    }
}

fn draw_rounded_rect(
    image: &mut RgbaImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radius: u32,
    color: [u8; 4],
) {
    for py in y..y + height {
        for px in x..x + width {
            if point_inside_rounded_rect(px, py, x, y, width, height, radius) {
                blend_pixel(image, px, py, color);
            }
        }
    }
}

fn point_inside_rounded_rect(
    px: u32,
    py: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radius: u32,
) -> bool {
    let px = px as i32;
    let py = py as i32;
    let x = x as i32;
    let y = y as i32;
    let width = width as i32;
    let height = height as i32;
    let radius = radius as i32;
    let max_x = x + width - 1;
    let max_y = y + height - 1;

    if px < x || px > max_x || py < y || py > max_y {
        return false;
    }

    if px >= x + radius && px <= max_x - radius {
        return true;
    }

    if py >= y + radius && py <= max_y - radius {
        return true;
    }

    let center_x = if px < x + radius {
        x + radius
    } else {
        max_x - radius
    };
    let center_y = if py < y + radius {
        y + radius
    } else {
        max_y - radius
    };
    let dx = px - center_x;
    let dy = py - center_y;

    dx * dx + dy * dy <= radius * radius
}

fn draw_ghost_badge(image: &mut RgbaImage, center_x: u32, center_y: u32, scale: u32) {
    draw_circle(
        image,
        center_x,
        center_y - 5 * scale,
        8 * scale,
        GHOST_BADGE,
    );
    fill_rect(
        image,
        center_x - 8 * scale,
        center_y - 5 * scale,
        16 * scale,
        11 * scale,
        GHOST_BADGE,
    );
    draw_circle(
        image,
        center_x - 5 * scale,
        center_y + 5 * scale,
        3 * scale,
        GHOST_BADGE,
    );
    draw_circle(
        image,
        center_x,
        center_y + 6 * scale,
        3 * scale,
        GHOST_BADGE,
    );
    draw_circle(
        image,
        center_x + 5 * scale,
        center_y + 5 * scale,
        3 * scale,
        GHOST_BADGE,
    );
    draw_circle(
        image,
        center_x - 3 * scale,
        center_y - 3 * scale,
        scale,
        GHOST_EYE,
    );
    draw_circle(
        image,
        center_x + 3 * scale,
        center_y - 3 * scale,
        scale,
        GHOST_EYE,
    );
}

fn blit_scaled_to_fit(
    source: &RenderedImage,
    destination: &mut RgbaImage,
    frame_x: u32,
    frame_y: u32,
    frame_width: u32,
    frame_height: u32,
) {
    let width_ratio = frame_width as f32 / source.width as f32;
    let height_ratio = frame_height as f32 / source.height as f32;
    let scale = width_ratio.min(height_ratio);
    let target_width = (source.width as f32 * scale).round() as u32;
    let target_height = (source.height as f32 * scale).round() as u32;
    let offset_x = frame_x + (frame_width - target_width) / 2;
    let offset_y = frame_y + (frame_height - target_height) / 2;

    for y in 0..target_height {
        let source_y = (y as f32 * source.height as f32 / target_height as f32).floor() as u32;
        for x in 0..target_width {
            let source_x = (x as f32 * source.width as f32 / target_width as f32).floor() as u32;
            let source_index = ((source_y * source.width + source_x) * 4) as usize;
            blend_pixel(
                destination,
                offset_x + x,
                offset_y + y,
                source.pixels[source_index..source_index + 4]
                    .try_into()
                    .unwrap_or(CONTENT_BACKGROUND),
            );
        }
    }
}

fn blend_pixel(image: &mut RgbaImage, x: u32, y: u32, color: [u8; 4]) {
    if x >= image.width || y >= image.height {
        return;
    }

    let index = ((y * image.width + x) * 4) as usize;
    let destination = &mut image.pixels[index..index + 4];
    let alpha = color[3] as u32;
    let inverse = 255 - alpha;
    let destination_alpha = destination[3] as u32;

    for channel in 0..3 {
        destination[channel] = (((color[channel] as u32 * alpha)
            + (destination[channel] as u32 * inverse))
            / 255) as u8;
    }
    destination[3] = (alpha + (destination_alpha * inverse) / 255) as u8;
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating directory {}", parent.display()))?;
    }
    Ok(())
}

struct RgbaImage {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl RgbaImage {
    fn new(width: u32, height: u32, color: [u8; 4]) -> Self {
        let mut pixels = vec![0; (width * height * 4) as usize];
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&color);
        }

        Self {
            width,
            height,
            pixels,
        }
    }
}
