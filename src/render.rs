//! Projects Battlezone world geometry into a Kitty graphics frame and draws overlay text.

use crate::math::{Vec3, rotate_y};
use crate::terminal::TerminalGeometry;

const NEAR_PLANE: f32 = 0.2;
const SKY_TOP: Color = Color(6, 10, 8, 255);
const SKY_BOTTOM: Color = Color(12, 32, 18, 255);
const GROUND_NEAR: Color = Color(14, 42, 14, 255);
const GROUND_FAR: Color = Color(4, 12, 4, 255);
const HORIZON_COLOR: Color = Color(50, 120, 50, 255);
const CROSSHAIR_COLOR: Color = Color(180, 255, 180, 255);

type ProjectedPoint = (f32, f32);
type ScreenPoint = (i32, i32);
type ProjectedSegment = (ScreenPoint, ScreenPoint, f32);

const OUT_LEFT: u8 = 0b0001;
const OUT_RIGHT: u8 = 0b0010;
const OUT_TOP: u8 = 0b0100;
const OUT_BOTTOM: u8 = 0b1000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub position: Vec3,
    pub heading: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldLine {
    pub start: Vec3,
    pub end: Vec3,
    pub brightness: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenLine {
    pub start: (i32, i32),
    pub end: (i32, i32),
    pub color: [u8; 4],
    pub thickness: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenDot {
    pub center: (i32, i32),
    pub color: [u8; 4],
    pub radius: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenText {
    pub position: (i32, i32),
    pub text: String,
    pub color: [u8; 4],
    pub scale: u8,
    pub centered: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub camera: Camera,
    pub world_lines: Vec<WorldLine>,
    pub overlay_lines: Vec<ScreenLine>,
    pub overlay_dots: Vec<ScreenDot>,
    pub overlay_text: Vec<ScreenText>,
    pub show_crosshair: bool,
}

pub struct RenderedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub struct Renderer {
    image_width: u32,
    image_height: u32,
    focal: f32,
}

#[derive(Clone, Copy)]
struct Color(u8, u8, u8, u8);

struct PixelBuffer {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

impl Scene {
    pub fn empty(camera: Camera) -> Self {
        Self {
            camera,
            world_lines: Vec::new(),
            overlay_lines: Vec::new(),
            overlay_dots: Vec::new(),
            overlay_text: Vec::new(),
            show_crosshair: false,
        }
    }
}

impl Renderer {
    pub fn new(geometry: TerminalGeometry) -> Self {
        let (image_width, image_height) = raster_size(geometry);
        Self {
            image_width,
            image_height,
            focal: image_width as f32 * 0.58,
        }
    }

    pub fn resize(&mut self, geometry: TerminalGeometry) {
        *self = Self::new(geometry);
    }

    pub fn image_width(&self) -> u32 {
        self.image_width
    }

    pub fn image_height(&self) -> u32 {
        self.image_height
    }

    pub fn render(&self, scene: &Scene) -> RenderedImage {
        let mut buffer = PixelBuffer::new(self.image_width as usize, self.image_height as usize);
        let horizon = self.image_height / 2;
        buffer.fill_gradient(horizon);
        buffer.draw_horizon(horizon, HORIZON_COLOR);

        for line in &scene.world_lines {
            if let Some(((x0, y0), (x1, y1), depth)) = project_segment(
                scene.camera,
                line.start,
                line.end,
                self.image_width,
                self.image_height,
                self.focal,
            ) {
                let color = world_color(depth, line.brightness);
                let thickness = depth_thickness(depth);
                buffer.draw_line(x0, y0, x1, y1, color, thickness);
            }
        }

        for line in &scene.overlay_lines {
            buffer.draw_line(
                line.start.0,
                line.start.1,
                line.end.0,
                line.end.1,
                Color::from_rgba(line.color),
                line.thickness,
            );
        }

        for dot in &scene.overlay_dots {
            buffer.draw_dot(
                dot.center.0,
                dot.center.1,
                Color::from_rgba(dot.color),
                dot.radius,
            );
        }

        for text in &scene.overlay_text {
            buffer.draw_text(text);
        }

        if scene.show_crosshair {
            buffer.draw_crosshair(CROSSHAIR_COLOR);
        }

        RenderedImage {
            width: self.image_width,
            height: self.image_height,
            pixels: buffer.pixels,
        }
    }
}

impl PixelBuffer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height * 4],
        }
    }

    fn fill_gradient(&mut self, horizon: u32) {
        for y in 0..self.height {
            let color = if y < horizon as usize {
                let t = y as f32 / horizon.max(1) as f32;
                lerp_color(SKY_TOP, SKY_BOTTOM, t)
            } else {
                let distance = (y - horizon as usize) as f32
                    / (self.height.saturating_sub(horizon as usize).max(1)) as f32;
                lerp_color(GROUND_FAR, GROUND_NEAR, distance)
            };

            for x in 0..self.width {
                self.put_pixel(x as i32, y as i32, color);
            }
        }
    }

    fn draw_horizon(&mut self, horizon: u32, color: Color) {
        self.draw_line(
            0,
            horizon as i32,
            self.width as i32 - 1,
            horizon as i32,
            color,
            1,
        );
    }

    fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color, thickness: i32) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let (mut x, mut y) = (x0, y0);

        loop {
            self.stamp(x, y, color, thickness);
            if x == x1 && y == y1 {
                break;
            }
            let doubled = err * 2;
            if doubled >= dy {
                err += dy;
                x += sx;
            }
            if doubled <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_dot(&mut self, cx: i32, cy: i32, color: Color, radius: i32) {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    self.put_pixel(cx + dx, cy + dy, color);
                }
            }
        }
    }

    fn draw_crosshair(&mut self, color: Color) {
        let cx = (self.width / 2) as i32;
        let cy = (self.height / 2) as i32;
        self.draw_line(cx - 12, cy, cx - 3, cy, color, 1);
        self.draw_line(cx + 3, cy, cx + 12, cy, color, 1);
        self.draw_line(cx, cy - 12, cx, cy - 3, color, 1);
        self.draw_line(cx, cy + 3, cx, cy + 12, color, 1);
        self.stamp(cx, cy, color, 1);
    }

    fn draw_text(&mut self, text: &ScreenText) {
        let scale = i32::from(text.scale.max(1));
        let width = text_width(&text.text, scale);
        let start_x = if text.centered {
            text.position.0 - width / 2
        } else {
            text.position.0
        };

        for (index, glyph) in text.text.chars().enumerate() {
            let x = start_x + index as i32 * glyph_advance(scale);
            self.draw_char(
                x,
                text.position.1,
                glyph,
                Color::from_rgba(text.color),
                scale,
            );
        }
    }

    fn draw_char(&mut self, x: i32, y: i32, glyph: char, color: Color, scale: i32) {
        for (row_index, bits) in glyph_rows(glyph).iter().enumerate() {
            for col in 0..5 {
                if (bits >> (4 - col)) & 1 == 0 {
                    continue;
                }
                for sy in 0..scale {
                    for sx in 0..scale {
                        self.put_pixel(
                            x + col * scale + sx,
                            y + row_index as i32 * scale + sy,
                            color,
                        );
                    }
                }
            }
        }
    }

    fn stamp(&mut self, x: i32, y: i32, color: Color, thickness: i32) {
        let radius = thickness.saturating_sub(1);
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                self.put_pixel(x + dx, y + dy, color);
            }
        }
    }

    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 {
            return;
        }

        let x = usize::try_from(x).ok();
        let y = usize::try_from(y).ok();
        let (Some(x), Some(y)) = (x, y) else {
            return;
        };

        if x >= self.width || y >= self.height {
            return;
        }

        let index = (y * self.width + x) * 4;
        self.pixels[index] = color.0;
        self.pixels[index + 1] = color.1;
        self.pixels[index + 2] = color.2;
        self.pixels[index + 3] = color.3;
    }
}

impl Color {
    fn from_rgba([r, g, b, a]: [u8; 4]) -> Self {
        Self(r, g, b, a)
    }
}

fn raster_size(geometry: TerminalGeometry) -> (u32, u32) {
    let source_width = if geometry.pixel_width > 0 {
        geometry.pixel_width as u32
    } else {
        u32::from(geometry.cols.max(40)) * 16
    };
    let source_height = if geometry.pixel_height > 0 {
        geometry.pixel_height as u32
    } else {
        u32::from(geometry.rows.max(18)) * 32
    };

    scale_to_fit(source_width, source_height, 960, 720)
}

fn scale_to_fit(width: u32, height: u32, max_width: u32, max_height: u32) -> (u32, u32) {
    if width == 0 || height == 0 {
        return (640, 360);
    }

    let scale = (max_width as f32 / width as f32)
        .min(max_height as f32 / height as f32)
        .min(1.0);

    let scaled_width = ((width as f32 * scale).round() as u32).max(320);
    let scaled_height = ((height as f32 * scale).round() as u32).max(180);
    (scaled_width, scaled_height)
}

fn lerp_color(start: Color, end: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let blend = |a: u8, b: u8| -> u8 { (a as f32 + (b as f32 - a as f32) * t).round() as u8 };
    Color(
        blend(start.0, end.0),
        blend(start.1, end.1),
        blend(start.2, end.2),
        blend(start.3, end.3),
    )
}

fn world_color(depth: f32, brightness: f32) -> Color {
    let base = if depth < 8.0 {
        Color(170, 255, 170, 255)
    } else if depth < 20.0 {
        Color(110, 220, 110, 255)
    } else if depth < 40.0 {
        Color(72, 180, 72, 255)
    } else {
        Color(35, 105, 35, 255)
    };
    let brightness = brightness.clamp(0.3, 1.5);
    Color(
        ((base.0 as f32 * brightness).min(255.0)) as u8,
        ((base.1 as f32 * brightness).min(255.0)) as u8,
        ((base.2 as f32 * brightness).min(255.0)) as u8,
        base.3,
    )
}

fn depth_thickness(depth: f32) -> i32 {
    if depth < 10.0 {
        3
    } else if depth < 24.0 {
        2
    } else {
        1
    }
}

fn world_to_camera(camera: Camera, point: Vec3) -> Vec3 {
    let translated = point - camera.position;
    rotate_y(translated, camera.heading)
}

fn clip_to_near_plane(mut start: Vec3, mut end: Vec3) -> Option<(Vec3, Vec3)> {
    if start.z < NEAR_PLANE && end.z < NEAR_PLANE {
        return None;
    }

    if start.z < NEAR_PLANE {
        let t = (NEAR_PLANE - start.z) / (end.z - start.z);
        start = start + (end - start) * t;
    }
    if end.z < NEAR_PLANE {
        let t = (NEAR_PLANE - end.z) / (start.z - end.z);
        end = end + (start - end) * t;
    }
    Some((start, end))
}

fn project_point(point: Vec3, width: u32, height: u32, focal: f32) -> Option<ProjectedPoint> {
    if point.z <= NEAR_PLANE {
        return None;
    }

    let x = point.x * focal / point.z + width as f32 * 0.5;
    let y = height as f32 * 0.5 - point.y * focal / point.z;
    Some((x, y))
}

fn project_segment(
    camera: Camera,
    start: Vec3,
    end: Vec3,
    width: u32,
    height: u32,
    focal: f32,
) -> Option<ProjectedSegment> {
    let start = world_to_camera(camera, start);
    let end = world_to_camera(camera, end);
    let (start, end) = clip_to_near_plane(start, end)?;

    let projected_start = project_point(start, width, height, focal)?;
    let projected_end = project_point(end, width, height, focal)?;

    let clipped = clip_to_viewport(projected_start, projected_end, width as i32, height as i32)?;
    let depth = start.z.min(end.z);
    Some((clipped.0, clipped.1, depth))
}

fn clip_to_viewport(
    start: ProjectedPoint,
    end: ProjectedPoint,
    width: i32,
    height: i32,
) -> Option<(ScreenPoint, ScreenPoint)> {
    let mut x0 = start.0;
    let mut y0 = start.1;
    let mut x1 = end.0;
    let mut y1 = end.1;

    let mut code0 = out_code(x0, y0, width, height);
    let mut code1 = out_code(x1, y1, width, height);

    loop {
        if code0 | code1 == 0 {
            return Some((
                (x0.round() as i32, y0.round() as i32),
                (x1.round() as i32, y1.round() as i32),
            ));
        }
        if code0 & code1 != 0 {
            return None;
        }

        let code_out = if code0 != 0 { code0 } else { code1 };
        let (mut x, mut y) = (0.0, 0.0);

        if code_out & OUT_TOP != 0 {
            x = x0 + (x1 - x0) * (0.0 - y0) / (y1 - y0);
            y = 0.0;
        } else if code_out & OUT_BOTTOM != 0 {
            x = x0 + (x1 - x0) * ((height - 1) as f32 - y0) / (y1 - y0);
            y = (height - 1) as f32;
        } else if code_out & OUT_RIGHT != 0 {
            y = y0 + (y1 - y0) * ((width - 1) as f32 - x0) / (x1 - x0);
            x = (width - 1) as f32;
        } else if code_out & OUT_LEFT != 0 {
            y = y0 + (y1 - y0) * (0.0 - x0) / (x1 - x0);
            x = 0.0;
        }

        if code_out == code0 {
            x0 = x;
            y0 = y;
            code0 = out_code(x0, y0, width, height);
        } else {
            x1 = x;
            y1 = y;
            code1 = out_code(x1, y1, width, height);
        }
    }
}

fn out_code(x: f32, y: f32, width: i32, height: i32) -> u8 {
    let mut code = 0;
    if x < 0.0 {
        code |= OUT_LEFT;
    } else if x >= width as f32 {
        code |= OUT_RIGHT;
    }
    if y < 0.0 {
        code |= OUT_TOP;
    } else if y >= height as f32 {
        code |= OUT_BOTTOM;
    }
    code
}

fn glyph_advance(scale: i32) -> i32 {
    6 * scale
}

fn text_width(text: &str, scale: i32) -> i32 {
    (text.chars().count() as i32 * glyph_advance(scale)).saturating_sub(scale)
}

fn glyph_rows(glyph: char) -> [u8; 7] {
    match glyph.to_ascii_uppercase() {
        'A' => [0x0e, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'B' => [0x1e, 0x11, 0x11, 0x1e, 0x11, 0x11, 0x1e],
        'C' => [0x0f, 0x10, 0x10, 0x10, 0x10, 0x10, 0x0f],
        'D' => [0x1e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1e],
        'E' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x1f],
        'F' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x10],
        'G' => [0x0f, 0x10, 0x10, 0x17, 0x11, 0x11, 0x0f],
        'H' => [0x11, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'I' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x1f],
        'J' => [0x1f, 0x02, 0x02, 0x02, 0x12, 0x12, 0x0c],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1f],
        'M' => [0x11, 0x1b, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'P' => [0x1e, 0x11, 0x11, 0x1e, 0x10, 0x10, 0x10],
        'Q' => [0x0e, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0d],
        'R' => [0x1e, 0x11, 0x11, 0x1e, 0x14, 0x12, 0x11],
        'S' => [0x0f, 0x10, 0x10, 0x0e, 0x01, 0x01, 0x1e],
        'T' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0a, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x1b, 0x11],
        'X' => [0x11, 0x11, 0x0a, 0x04, 0x0a, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0a, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1f],
        '0' => [0x0e, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0e],
        '1' => [0x04, 0x0c, 0x04, 0x04, 0x04, 0x04, 0x0e],
        '2' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1f],
        '3' => [0x1e, 0x01, 0x01, 0x06, 0x01, 0x01, 0x1e],
        '4' => [0x02, 0x06, 0x0a, 0x12, 0x1f, 0x02, 0x02],
        '5' => [0x1f, 0x10, 0x10, 0x1e, 0x01, 0x01, 0x1e],
        '6' => [0x07, 0x08, 0x10, 0x1e, 0x11, 0x11, 0x0e],
        '7' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0e, 0x11, 0x11, 0x0e, 0x11, 0x11, 0x0e],
        '9' => [0x0e, 0x11, 0x11, 0x0f, 0x01, 0x02, 0x1c],
        '-' => [0x00, 0x00, 0x00, 0x1f, 0x00, 0x00, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x06],
        ':' => [0x00, 0x06, 0x06, 0x00, 0x06, 0x06, 0x00],
        '!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0x00, 0x04],
        '>' => [0x10, 0x08, 0x04, 0x02, 0x04, 0x08, 0x10],
        '<' => [0x01, 0x02, 0x04, 0x08, 0x04, 0x02, 0x01],
        '/' => [0x01, 0x02, 0x04, 0x08, 0x10, 0x00, 0x00],
        ' ' => [0x00; 7],
        _ => [0x1f, 0x11, 0x01, 0x02, 0x04, 0x00, 0x04],
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Camera, Scene, ScreenDot, ScreenLine, ScreenText, WorldLine, clip_to_near_plane,
        project_segment, raster_size, scale_to_fit,
    };
    use crate::math::Vec3;
    use crate::terminal::TerminalGeometry;

    #[test]
    fn scale_to_fit_preserves_bounds() {
        assert_eq!(scale_to_fit(1920, 1080, 960, 720), (960, 540));
        assert_eq!(scale_to_fit(640, 480, 960, 720), (640, 480));
        assert_eq!(scale_to_fit(0, 0, 960, 720), (640, 360));
    }

    #[test]
    fn project_segment_renders_forward_line() {
        let camera = Camera {
            position: Vec3::new(0.0, 0.0, 0.0),
            heading: 0.0,
        };
        let segment = project_segment(
            camera,
            Vec3::new(-1.0, 0.0, 8.0),
            Vec3::new(1.0, 0.0, 8.0),
            640,
            360,
            320.0,
        );
        assert!(segment.is_some());
    }

    #[test]
    fn clip_to_near_plane_keeps_visible_portion() {
        let clipped = clip_to_near_plane(Vec3::new(0.0, 0.0, 0.1), Vec3::new(0.0, 0.0, 2.0))
            .expect("segment should clip");
        assert!((clipped.0.z - 0.2).abs() < 0.001);
        assert!((clipped.1.z - 2.0).abs() < 0.001);
    }

    #[test]
    fn viewport_clipping_bounds_extreme_segments() {
        let camera = Camera {
            position: Vec3::new(0.0, 0.0, 0.0),
            heading: 0.0,
        };
        let segment = project_segment(
            camera,
            Vec3::new(-50.0, 0.0, 10.0),
            Vec3::new(50.0, 0.0, 10.0),
            320,
            180,
            160.0,
        )
        .expect("segment should clip into viewport");
        assert!(segment.0.0 >= 0);
        assert!(segment.1.0 < 320);
    }

    #[test]
    fn raster_size_uses_terminal_pixels_when_available() {
        let geometry = TerminalGeometry {
            cols: 100,
            rows: 40,
            pixel_width: 1200,
            pixel_height: 800,
        };
        assert_eq!(raster_size(geometry), (960, 640));
    }

    #[test]
    fn renderer_scene_types_are_constructible() {
        let mut scene = Scene::empty(Camera {
            position: Vec3::new(0.0, 0.0, 0.0),
            heading: 0.0,
        });
        scene.world_lines.push(WorldLine {
            start: Vec3::new(0.0, 0.0, 8.0),
            end: Vec3::new(1.0, 0.0, 8.0),
            brightness: 1.0,
        });
        scene.overlay_lines.push(ScreenLine {
            start: (0, 0),
            end: (10, 10),
            color: [255, 255, 255, 255],
            thickness: 1,
        });
        scene.overlay_dots.push(ScreenDot {
            center: (12, 12),
            color: [255, 255, 255, 255],
            radius: 2,
        });
        scene.overlay_text.push(ScreenText {
            position: (20, 20),
            text: String::from("BATTLEZONE"),
            color: [255, 255, 255, 255],
            scale: 2,
            centered: true,
        });
        assert_eq!(scene.overlay_text.len(), 1);
    }
}
