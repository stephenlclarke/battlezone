use crate::game::{Camera, FrameData, WorldLine};
use crate::math::{Vec3, rotate_y};
use crate::terminal::TerminalGeometry;

const NEAR_PLANE: f32 = 0.2;
const SKY_TOP: Color = Color(6, 12, 10, 255);
const SKY_BOTTOM: Color = Color(12, 42, 22, 255);
const GROUND_NEAR: Color = Color(18, 56, 18, 255);
const GROUND_FAR: Color = Color(6, 16, 6, 255);
const HORIZON_COLOR: Color = Color(50, 120, 50, 255);
const CROSSHAIR_COLOR: Color = Color(160, 255, 160, 255);

type ScreenPoint = (i32, i32);
type ProjectedSegment = (ScreenPoint, ScreenPoint, f32);

#[derive(Clone, Copy)]
struct Color(u8, u8, u8, u8);

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

struct PixelBuffer {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
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

    pub fn render(&self, frame: &FrameData) -> RenderedImage {
        let mut buffer = PixelBuffer::new(self.image_width as usize, self.image_height as usize);
        buffer.fill_gradient(self.image_height / 2);
        buffer.draw_horizon(self.image_height / 2, HORIZON_COLOR);

        for WorldLine { start, end } in &frame.lines {
            if let Some(((x0, y0), (x1, y1), depth)) = project_segment(
                frame.camera,
                *start,
                *end,
                self.image_width,
                self.image_height,
                self.focal,
            ) {
                let color = depth_color(depth);
                let thickness = depth_thickness(depth);
                buffer.draw_line(x0, y0, x1, y1, color, thickness);
            }
        }

        buffer.draw_crosshair(CROSSHAIR_COLOR);

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
        let y = horizon as i32;
        self.draw_line(0, y, self.width as i32 - 1, y, color, 1);
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

    fn draw_crosshair(&mut self, color: Color) {
        let cx = (self.width / 2) as i32;
        let cy = (self.height / 2) as i32;
        self.draw_line(cx - 12, cy, cx - 3, cy, color, 1);
        self.draw_line(cx + 3, cy, cx + 12, cy, color, 1);
        self.draw_line(cx, cy - 12, cx, cy - 3, color, 1);
        self.draw_line(cx, cy + 3, cx, cy + 12, color, 1);
        self.stamp(cx, cy, color, 1);
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

fn depth_color(depth: f32) -> Color {
    if depth < 8.0 {
        Color(170, 255, 170, 255)
    } else if depth < 20.0 {
        Color(100, 225, 100, 255)
    } else if depth < 40.0 {
        Color(65, 185, 65, 255)
    } else {
        Color(35, 105, 35, 255)
    }
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
    let depth = (start.z + end.z) * 0.5;
    Some((projected_start, projected_end, depth))
}

fn world_to_camera(camera: Camera, point: Vec3) -> Vec3 {
    let relative = point - camera.position;
    rotate_y(relative, -camera.heading)
}

fn clip_to_near_plane(mut start: Vec3, mut end: Vec3) -> Option<(Vec3, Vec3)> {
    if start.z < NEAR_PLANE && end.z < NEAR_PLANE {
        return None;
    }

    if start.z < NEAR_PLANE {
        start = interpolate_to_near(start, end);
    }
    if end.z < NEAR_PLANE {
        end = interpolate_to_near(end, start);
    }

    Some((start, end))
}

fn interpolate_to_near(behind: Vec3, front: Vec3) -> Vec3 {
    let t = (NEAR_PLANE - behind.z) / (front.z - behind.z);
    Vec3::new(
        behind.x + (front.x - behind.x) * t,
        behind.y + (front.y - behind.y) * t,
        NEAR_PLANE,
    )
}

fn project_point(point: Vec3, width: u32, height: u32, focal: f32) -> Option<(i32, i32)> {
    if point.z <= NEAR_PLANE {
        return None;
    }

    let half_width = width as f32 * 0.5;
    let half_height = height as f32 * 0.5;
    let x = half_width + (point.x / point.z) * focal;
    let y = half_height - (point.y / point.z) * focal * 0.65;
    Some((x.round() as i32, y.round() as i32))
}

#[cfg(test)]
mod tests {
    use crate::game::Camera;
    use crate::math::Vec3;
    use crate::terminal::TerminalGeometry;

    use super::{Renderer, clip_to_near_plane, project_segment, scale_to_fit};

    #[test]
    fn clip_to_near_plane_keeps_visible_portion() {
        let start = Vec3::new(0.0, 0.0, -1.0);
        let end = Vec3::new(0.0, 0.0, 2.0);
        let clipped = clip_to_near_plane(start, end).expect("segment should remain visible");
        assert!(clipped.0.z >= 0.2);
        assert!((clipped.1.z - 2.0).abs() < 0.001);
    }

    #[test]
    fn project_segment_renders_forward_line() {
        let camera = Camera {
            position: Vec3::new(0.0, 0.0, 0.0),
            heading: 0.0,
        };
        let projection = project_segment(
            camera,
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(1.0, 0.0, 5.0),
            640,
            360,
            300.0,
        );

        assert!(projection.is_some());
    }

    #[test]
    fn renderer_produces_rgba_frame() {
        let renderer = Renderer::new(TerminalGeometry {
            cols: 80,
            rows: 24,
            pixel_width: 800,
            pixel_height: 600,
        });

        let image = renderer.render(&crate::game::Game::new().frame());
        assert_eq!(
            image.pixels.len(),
            image.width as usize * image.height as usize * 4
        );
    }

    #[test]
    fn scale_to_fit_preserves_bounds() {
        let (width, height) = scale_to_fit(1600, 900, 960, 720);
        assert!(width <= 960);
        assert!(height <= 720);
    }
}
