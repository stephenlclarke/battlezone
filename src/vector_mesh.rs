//! Converts render scenes into GPU-ready vector triangles.

use crate::render::{
    BackgroundStyle, CROSSHAIR_RGBA, GROUND_FAR_RGBA, GROUND_NEAR_RGBA, HORIZON_RGBA,
    SKY_BOTTOM_RGBA, SKY_TOP_RGBA, Scene, ScreenLine, ScreenText, ViewportSize, depth_thickness,
    glyph_advance, glyph_rows, project_segment, text_width, world_color_rgba,
};

pub(crate) const VERTEX_SIZE: wgpu::BufferAddress =
    std::mem::size_of::<Vertex>() as wgpu::BufferAddress;
pub(crate) const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 2] = [
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 0,
        shader_location: 0,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: 8,
        shader_location: 1,
    },
];

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

#[derive(Default)]
pub(crate) struct VectorMesh {
    vertices: Vec<Vertex>,
}

#[derive(Clone, Copy)]
struct PixelRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Copy)]
struct VerticalGradient {
    top: [u8; 4],
    bottom: [u8; 4],
}

impl VectorMesh {
    pub(crate) fn rebuild(&mut self, scene: &Scene, viewport: ViewportSize) {
        self.vertices.clear();
        self.add_background(scene.background, viewport);

        let focal = viewport.width as f32 * 0.58;
        for line in &scene.world_lines {
            if let Some((start, end, depth)) = project_segment(
                scene.camera,
                line.start,
                line.end,
                viewport.width,
                viewport.height,
                focal,
            ) {
                self.add_line(
                    start,
                    end,
                    world_color_rgba(depth, line.brightness, line.color),
                    depth_thickness(depth),
                    viewport,
                );
            }
        }

        for line in &scene.overlay_lines {
            self.add_screen_line(line, viewport);
        }

        for dot in &scene.overlay_dots {
            self.add_dot(
                dot.center.0 as f32,
                dot.center.1 as f32,
                dot.radius.max(1) as f32,
                dot.color,
                viewport,
            );
        }

        for text in &scene.overlay_text {
            self.add_text(text, viewport);
        }

        if scene.show_crosshair {
            self.add_crosshair(viewport);
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    pub(crate) fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub(crate) fn vertex_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }

    fn add_background(&mut self, background: BackgroundStyle, viewport: ViewportSize) {
        match background {
            BackgroundStyle::GradientHorizon => {
                let horizon = viewport.height as f32 * 0.5;
                self.add_rect_gradient(
                    PixelRect {
                        x: 0.0,
                        y: 0.0,
                        width: viewport.width as f32,
                        height: horizon,
                    },
                    VerticalGradient {
                        top: SKY_TOP_RGBA,
                        bottom: SKY_BOTTOM_RGBA,
                    },
                    viewport,
                );
                self.add_rect_gradient(
                    PixelRect {
                        x: 0.0,
                        y: horizon,
                        width: viewport.width as f32,
                        height: viewport.height as f32 - horizon,
                    },
                    VerticalGradient {
                        top: GROUND_FAR_RGBA,
                        bottom: GROUND_NEAR_RGBA,
                    },
                    viewport,
                );
                self.add_line(
                    (0, horizon.round() as i32),
                    (viewport.width as i32 - 1, horizon.round() as i32),
                    HORIZON_RGBA,
                    1,
                    viewport,
                );
            }
            BackgroundStyle::Solid(color) => {
                self.add_rect(
                    PixelRect {
                        x: 0.0,
                        y: 0.0,
                        width: viewport.width as f32,
                        height: viewport.height as f32,
                    },
                    color,
                    viewport,
                );
            }
        }
    }

    fn add_screen_line(&mut self, line: &ScreenLine, viewport: ViewportSize) {
        self.add_line(line.start, line.end, line.color, line.thickness, viewport);
    }

    fn add_line(
        &mut self,
        start: (i32, i32),
        end: (i32, i32),
        color: [u8; 4],
        thickness: i32,
        viewport: ViewportSize,
    ) {
        let start = (start.0 as f32, start.1 as f32);
        let end = (end.0 as f32, end.1 as f32);
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let length = (dx * dx + dy * dy).sqrt();
        if length <= f32::EPSILON {
            let size = thickness.max(1) as f32;
            self.add_rect(
                PixelRect {
                    x: start.0 - size * 0.5,
                    y: start.1 - size * 0.5,
                    width: size,
                    height: size,
                },
                color,
                viewport,
            );
            return;
        }

        let radius = thickness.max(1) as f32 * 0.5;
        let nx = -dy / length * radius;
        let ny = dx / length * radius;
        let color = bright_srgb_color_to_surface_linear(color);
        self.add_triangle(
            vertex(start.0 + nx, start.1 + ny, color, viewport),
            vertex(start.0 - nx, start.1 - ny, color, viewport),
            vertex(end.0 + nx, end.1 + ny, color, viewport),
        );
        self.add_triangle(
            vertex(end.0 + nx, end.1 + ny, color, viewport),
            vertex(start.0 - nx, start.1 - ny, color, viewport),
            vertex(end.0 - nx, end.1 - ny, color, viewport),
        );
    }

    fn add_crosshair(&mut self, viewport: ViewportSize) {
        let cx = (viewport.width / 2) as i32;
        let cy = (viewport.height / 2) as i32;
        for (start, end) in [
            ((cx - 12, cy), (cx - 3, cy)),
            ((cx + 3, cy), (cx + 12, cy)),
            ((cx, cy - 12), (cx, cy - 3)),
            ((cx, cy + 3), (cx, cy + 12)),
        ] {
            self.add_line(start, end, CROSSHAIR_RGBA, 1, viewport);
        }
        self.add_rect(
            PixelRect {
                x: cx as f32,
                y: cy as f32,
                width: 1.0,
                height: 1.0,
            },
            CROSSHAIR_RGBA,
            viewport,
        );
    }

    fn add_dot(
        &mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
        color: [u8; 4],
        viewport: ViewportSize,
    ) {
        let color = bright_srgb_color_to_surface_linear(color);
        let segments = ((radius * 2.5).round() as usize).clamp(12, 32);
        for index in 0..segments {
            let a0 = std::f32::consts::TAU * index as f32 / segments as f32;
            let a1 = std::f32::consts::TAU * (index + 1) as f32 / segments as f32;
            self.add_triangle(
                vertex(center_x, center_y, color, viewport),
                vertex(
                    center_x + a0.cos() * radius,
                    center_y + a0.sin() * radius,
                    color,
                    viewport,
                ),
                vertex(
                    center_x + a1.cos() * radius,
                    center_y + a1.sin() * radius,
                    color,
                    viewport,
                ),
            );
        }
    }

    fn add_text(&mut self, text: &ScreenText, viewport: ViewportSize) {
        let scale = i32::from(text.scale.max(1));
        let width = text_width(&text.text, scale);
        let start_x = if text.centered {
            text.position.0 - width / 2
        } else {
            text.position.0
        };

        for (index, glyph) in text.text.chars().enumerate() {
            let x = start_x + index as i32 * glyph_advance(scale);
            self.add_glyph_rects(x, text.position.1, glyph, text.color, scale, viewport);
        }
    }

    fn add_glyph_rects(
        &mut self,
        x: i32,
        y: i32,
        glyph: char,
        color: [u8; 4],
        scale: i32,
        viewport: ViewportSize,
    ) {
        for (row_index, bits) in glyph_rows(glyph).iter().enumerate() {
            for col in 0..5 {
                if (bits >> (4 - col)) & 1 == 0 {
                    continue;
                }
                self.add_rect(
                    PixelRect {
                        x: (x + col * scale) as f32,
                        y: (y + row_index as i32 * scale) as f32,
                        width: scale as f32,
                        height: scale as f32,
                    },
                    color,
                    viewport,
                );
            }
        }
    }

    fn add_rect(&mut self, rect: PixelRect, color: [u8; 4], viewport: ViewportSize) {
        self.add_rect_gradient(
            rect,
            VerticalGradient {
                top: color,
                bottom: color,
            },
            viewport,
        );
    }

    fn add_rect_gradient(
        &mut self,
        rect: PixelRect,
        gradient: VerticalGradient,
        viewport: ViewportSize,
    ) {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }

        let top_color = bright_srgb_color_to_surface_linear(gradient.top);
        let bottom_color = bright_srgb_color_to_surface_linear(gradient.bottom);
        let x1 = rect.x + rect.width;
        let y1 = rect.y + rect.height;
        self.add_triangle(
            vertex(rect.x, rect.y, top_color, viewport),
            vertex(rect.x, y1, bottom_color, viewport),
            vertex(x1, rect.y, top_color, viewport),
        );
        self.add_triangle(
            vertex(x1, rect.y, top_color, viewport),
            vertex(rect.x, y1, bottom_color, viewport),
            vertex(x1, y1, bottom_color, viewport),
        );
    }

    fn add_triangle(&mut self, a: Vertex, b: Vertex, c: Vertex) {
        self.vertices.extend([a, b, c]);
    }
}

fn bright_srgb_color_to_surface_linear([r, g, b, a]: [u8; 4]) -> [f32; 4] {
    // The swapchain is sRGB. These bytes intentionally describe the current
    // bright display-space palette, so they are emitted as linear intensities
    // and encoded by the surface on presentation.
    [
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ]
}

fn vertex(x: f32, y: f32, color: [f32; 4], viewport: ViewportSize) -> Vertex {
    Vertex {
        position: [
            x / viewport.width as f32 * 2.0 - 1.0,
            1.0 - y / viewport.height as f32 * 2.0,
        ],
        color,
    }
}

#[cfg(test)]
mod tests {
    use super::{PixelRect, VectorMesh, VerticalGradient, bright_srgb_color_to_surface_linear};
    use crate::{
        math::Vec3,
        render::{
            BackgroundStyle, Camera, Scene, ScreenDot, ScreenLine, ScreenText, ViewportSize,
            WorldLine,
        },
    };

    #[test]
    fn vector_mesh_builds_scene_without_bitmap_pixels() {
        let mut scene = Scene::empty(Camera {
            position: Vec3::new(0.0, 0.0, 0.0),
            heading: 0.0,
        });
        scene.world_lines.push(WorldLine {
            start: Vec3::new(-1.0, 0.0, 8.0),
            end: Vec3::new(1.0, 0.0, 8.0),
            brightness: 1.0,
            color: None,
        });
        scene.overlay_lines.push(ScreenLine {
            start: (10, 10),
            end: (30, 10),
            color: [255, 255, 255, 255],
            thickness: 2,
        });
        scene.overlay_dots.push(ScreenDot {
            center: (40, 18),
            color: [255, 255, 255, 255],
            radius: 3,
        });
        scene.overlay_text.push(ScreenText {
            position: (20, 20),
            text: String::from("HI"),
            color: [180, 255, 180, 255],
            scale: 2,
            centered: false,
        });
        scene.overlay_text.push(ScreenText {
            position: (100, 40),
            text: String::from("OK"),
            color: [180, 255, 180, 255],
            scale: 1,
            centered: true,
        });
        scene.show_crosshair = true;

        let mut mesh = VectorMesh::default();
        mesh.rebuild(&scene, ViewportSize::new(640, 360));

        assert!(mesh.vertex_count() > 18);
        assert_eq!(mesh.vertex_count() % 3, 0);
    }

    #[test]
    fn vector_mesh_uses_full_screen_coordinates() {
        let mut mesh = VectorMesh::default();
        mesh.add_rect(
            PixelRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 180.0,
            },
            [255, 255, 255, 255],
            ViewportSize::new(320, 180),
        );

        assert_eq!(mesh.vertices[0].position, [-1.0, 1.0]);
        assert_eq!(mesh.vertices[5].position, [1.0, -1.0]);
    }

    #[test]
    fn vector_mesh_renders_solid_background_and_skips_empty_rectangles() {
        let mut mesh = VectorMesh::default();
        mesh.add_background(
            BackgroundStyle::Solid([10, 20, 30, 255]),
            ViewportSize::new(320, 180),
        );
        let solid_vertices = mesh.vertex_count();

        mesh.add_rect_gradient(
            PixelRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 10.0,
            },
            VerticalGradient {
                top: [255, 255, 255, 255],
                bottom: [0, 0, 0, 255],
            },
            ViewportSize::new(320, 180),
        );

        assert_eq!(solid_vertices, 6);
        assert_eq!(mesh.vertex_count(), solid_vertices);
    }

    #[test]
    fn vector_mesh_centers_degenerate_lines() {
        let mut mesh = VectorMesh::default();
        mesh.add_line(
            (50, 50),
            (50, 50),
            [255, 255, 255, 255],
            4,
            ViewportSize::new(100, 100),
        );

        let min_x = mesh
            .vertices
            .iter()
            .map(|vertex| vertex.position[0])
            .fold(f32::INFINITY, f32::min);
        let max_x = mesh
            .vertices
            .iter()
            .map(|vertex| vertex.position[0])
            .fold(f32::NEG_INFINITY, f32::max);

        assert!((min_x + 0.04).abs() < 0.001);
        assert!((max_x - 0.04).abs() < 0.001);
    }

    #[test]
    fn bright_srgb_palette_is_preserved_for_srgb_surface_output() {
        let color = bright_srgb_color_to_surface_linear([50, 120, 50, 255]);

        assert!((color[0] - 50.0 / 255.0).abs() < f32::EPSILON);
        assert!((color[1] - 120.0 / 255.0).abs() < f32::EPSILON);
        assert_eq!(color[3], 1.0);
    }
}
