//! Runs the windowed application loop and presents frames from the game thread.

use std::sync::Arc;

use anyhow::{Context, Result};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

use crate::{
    gpu::GpuPresenter,
    input::{InputEvent, InputKey, InputPhase},
    render::{RenderedImage, ViewportSize},
    runtime::{GameRuntime, RuntimeEvent},
};

const WINDOW_TITLE: &str = "Battlezone";
const INITIAL_WINDOW_WIDTH: f64 = 960.0;
const INITIAL_WINDOW_HEIGHT: f64 = 720.0;
const MIN_WINDOW_WIDTH: f64 = 320.0;
const MIN_WINDOW_HEIGHT: f64 = 180.0;

pub fn run() -> Result<()> {
    let event_loop = EventLoop::<RuntimeEvent>::with_user_event()
        .build()
        .context("creating Battlezone event loop")?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let event_proxy = event_loop.create_proxy();
    let mut app = BattlezoneApp::new(event_proxy);
    event_loop
        .run_app(&mut app)
        .context("running Battlezone event loop")?;
    app.finish()
}

struct BattlezoneApp {
    event_proxy: EventLoopProxy<RuntimeEvent>,
    window: Option<Arc<Window>>,
    window_id: Option<WindowId>,
    gpu: Option<GpuPresenter>,
    runtime: Option<GameRuntime>,
    latest_frame: Option<RenderedImage>,
    redraw_requested: bool,
    error: Option<anyhow::Error>,
}

impl BattlezoneApp {
    fn new(event_proxy: EventLoopProxy<RuntimeEvent>) -> Self {
        Self {
            event_proxy,
            window: None,
            window_id: None,
            gpu: None,
            runtime: None,
            latest_frame: None,
            redraw_requested: false,
            error: None,
        }
    }

    fn finish(mut self) -> Result<()> {
        if let Some(runtime) = self.runtime.as_mut() {
            runtime.shutdown();
        }
        if let Some(error) = self.error.take() {
            Err(error)
        } else {
            Ok(())
        }
    }

    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        if self.window.is_some() {
            return Ok(());
        }

        let window = Arc::new(event_loop.create_window(window_attributes())?);
        let gpu = pollster::block_on(GpuPresenter::new(window.clone()))?;
        let runtime = GameRuntime::spawn(viewport_for_window(&window), self.event_proxy.clone())?;

        self.window_id = Some(window.id());
        self.gpu = Some(gpu);
        self.runtime = Some(runtime);
        self.window = Some(window);
        Ok(())
    }

    fn handle_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if Some(window_id) != self.window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => self.exit(event_loop),
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::Focused(false) => {
                if let Some(runtime) = &self.runtime {
                    runtime.clear_input();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(input) = input_event_for_key_event(&event)
                    && let Some(runtime) = &self.runtime
                {
                    runtime.send_input(input);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.draw() {
                    self.record_error(error, event_loop);
                }
            }
            _ => {}
        }
    }

    fn handle_runtime_event(&mut self, event_loop: &ActiveEventLoop, event: RuntimeEvent) {
        match event {
            RuntimeEvent::FrameReady => {
                if self.take_latest_frame() {
                    self.request_redraw();
                }
            }
            RuntimeEvent::QuitRequested => self.exit(event_loop),
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.resize(size);
        }
        if let Some(runtime) = &self.runtime {
            runtime.resize(ViewportSize::new(size.width, size.height));
        }
        self.request_redraw();
    }

    fn draw(&mut self) -> Result<()> {
        self.redraw_requested = false;
        self.take_latest_frame();

        let (Some(window), Some(gpu), Some(frame)) = (
            self.window.as_ref(),
            self.gpu.as_mut(),
            self.latest_frame.as_ref(),
        ) else {
            return Ok(());
        };

        window.pre_present_notify();
        gpu.present(frame)
    }

    fn take_latest_frame(&mut self) -> bool {
        let Some(runtime) = &self.runtime else {
            return false;
        };
        let Some(frame) = runtime.take_frame() else {
            return false;
        };
        self.latest_frame = Some(frame);
        true
    }

    fn request_redraw(&mut self) {
        if self.redraw_requested {
            return;
        }
        if let Some(window) = &self.window {
            window.request_redraw();
            self.redraw_requested = true;
        }
    }

    fn exit(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(runtime) = self.runtime.as_mut() {
            runtime.shutdown();
        }
        event_loop.exit();
    }

    fn record_error(&mut self, error: anyhow::Error, event_loop: &ActiveEventLoop) {
        if self.error.is_none() {
            self.error = Some(error);
        }
        self.exit(event_loop);
    }
}

impl ApplicationHandler<RuntimeEvent> for BattlezoneApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.initialize(event_loop) {
            self.record_error(error, event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        self.handle_window_event(event_loop, window_id, event);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: RuntimeEvent) {
        self.handle_runtime_event(event_loop, event);
    }
}

fn window_attributes() -> winit::window::WindowAttributes {
    Window::default_attributes()
        .with_title(WINDOW_TITLE)
        .with_inner_size(LogicalSize::new(
            INITIAL_WINDOW_WIDTH,
            INITIAL_WINDOW_HEIGHT,
        ))
        .with_min_inner_size(LogicalSize::new(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT))
}

fn viewport_for_window(window: &Window) -> ViewportSize {
    let size = window.inner_size();
    ViewportSize::new(size.width, size.height)
}

fn input_event_for_key_event(event: &KeyEvent) -> Option<InputEvent> {
    let key = input_key_for_winit_key(event.logical_key.as_ref())?;
    let phase = match event.state {
        ElementState::Pressed => InputPhase::Pressed,
        ElementState::Released => InputPhase::Released,
    };
    Some(InputEvent { key, phase })
}

fn input_key_for_winit_key(key: Key<&str>) -> Option<InputKey> {
    match key {
        Key::Named(NamedKey::ArrowUp) => Some(InputKey::Up),
        Key::Named(NamedKey::ArrowDown) => Some(InputKey::Down),
        Key::Named(NamedKey::ArrowLeft) => Some(InputKey::Left),
        Key::Named(NamedKey::ArrowRight) => Some(InputKey::Right),
        Key::Named(NamedKey::Enter) => Some(InputKey::Enter),
        Key::Named(NamedKey::Escape) => Some(InputKey::Escape),
        Key::Named(NamedKey::Space) => Some(InputKey::Character(' ')),
        Key::Character(text) => single_character_key(text).map(InputKey::Character),
        _ => None,
    }
}

fn single_character_key(text: &str) -> Option<char> {
    let mut chars = text.chars();
    let character = chars.next()?;
    if chars.next().is_none() {
        Some(character)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{input_key_for_winit_key, single_character_key};
    use crate::input::InputKey;
    use winit::keyboard::{Key, NamedKey};

    #[test]
    fn single_character_key_rejects_empty_or_multi_character_text() {
        assert_eq!(single_character_key("q"), Some('q'));
        assert_eq!(single_character_key(""), None);
        assert_eq!(single_character_key("xy"), None);
    }

    #[test]
    fn winit_key_mapping_covers_game_controls() {
        assert_eq!(
            input_key_for_winit_key(Key::Named(NamedKey::ArrowUp)),
            Some(InputKey::Up)
        );
        assert_eq!(
            input_key_for_winit_key(Key::Named(NamedKey::Space)),
            Some(InputKey::Character(' '))
        );
        assert_eq!(
            input_key_for_winit_key(Key::Character("q")),
            Some(InputKey::Character('q'))
        );
    }
}
