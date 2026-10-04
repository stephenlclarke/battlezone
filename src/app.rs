//! Native window lifecycle, fixed-timestep game updates, and GPU presentation.
use crate::{
    arcade::ORIGINAL_FRAME_TIME,
    audio::AudioManager,
    constants::MAX_DT,
    game::Game,
    graphics::Graphics,
    input::InputTracker,
    render::{RenderSize, Renderer},
};
use anyhow::{Context, Result};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

pub fn run() -> Result<()> {
    let event_loop = EventLoop::new().context("creating window event loop")?;
    let mut app = App::new();
    event_loop
        .run_app(&mut app)
        .context("running window event loop")?;
    match app.error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

struct App {
    window: Option<Arc<Window>>,
    graphics: Option<Graphics>,
    renderer: Renderer,
    game: Game,
    audio: AudioManager,
    input: InputTracker,
    last_tick: Instant,
    accumulator: f32,
    error: Option<anyhow::Error>,
}

impl App {
    fn new() -> Self {
        let renderer = Renderer::new(RenderSize {
            width: 960,
            height: 720,
        });
        let mut game = Game::load();
        game.set_viewport(renderer.image_width(), renderer.image_height());
        Self {
            window: None,
            graphics: None,
            renderer,
            game,
            audio: AudioManager::new(),
            input: InputTracker::default(),
            last_tick: Instant::now(),
            accumulator: 0.0,
            error: None,
        }
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: anyhow::Error) {
        self.error = Some(error);
        event_loop.exit();
    }

    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        if self.window.is_some() {
            return Ok(());
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Battlezone")
                        .with_inner_size(winit::dpi::LogicalSize::new(960.0, 720.0)),
                )
                .context("creating Battlezone window")?,
        );
        self.graphics = Some(pollster::block_on(Graphics::new(window.clone()))?);
        self.resize(window.inner_size());
        self.window = Some(window);
        self.last_tick = Instant::now();
        Ok(())
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if let Some(graphics) = &mut self.graphics {
            graphics.resize(size);
        }
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.renderer.resize(RenderSize {
            width: size.width,
            height: size.height,
        });
        self.game
            .set_viewport(self.renderer.image_width(), self.renderer.image_height());
    }

    fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32().min(MAX_DT);
        self.last_tick = now;
        self.accumulator = (self.accumulator + dt).min(ORIGINAL_FRAME_TIME * 6.0);
        let steps = consume_fixed_steps(&mut self.accumulator, ORIGINAL_FRAME_TIME);
        for _ in 0..steps {
            self.game
                .update_with_input(ORIGINAL_FRAME_TIME, self.input.drain());
            for event in self.game.drain_events() {
                self.audio.handle_event(event);
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.initialize(event_loop) {
            self.fail(event_loop, error);
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if !self.window.as_ref().is_some_and(|window| window.id() == id) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Focused(false) => self.input.clear(),
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                if let PhysicalKey::Code(key) = event.physical_key {
                    if key == winit::keyboard::KeyCode::Escape && pressed {
                        event_loop.exit();
                    }
                    self.input.key(key, pressed, event.repeat);
                }
                if pressed
                    && !event.repeat
                    && let Some(text) = event.text
                {
                    self.input.text(&text);
                }
            }
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    window.pre_present_notify();
                }
                let image = self.renderer.render(&self.game.frame());
                if let Some(graphics) = &mut self.graphics
                    && let Err(error) = graphics.draw(&image)
                {
                    self.fail(event_loop, error);
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.update();
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        let remaining = (ORIGINAL_FRAME_TIME - self.accumulator).max(0.001);
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_secs_f32(remaining),
        ));
    }
    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.graphics = None;
        self.window = None;
        self.input.clear();
        self.accumulator = 0.0;
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
#[cfg(test)]
mod tests {
    use super::consume_fixed_steps;

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
}
