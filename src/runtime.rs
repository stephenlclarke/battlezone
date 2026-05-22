//! Runs the game simulation on its own thread and hands completed frames to the UI thread.

use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use winit::event_loop::EventLoopProxy;

use crate::{
    arcade::ORIGINAL_FRAME_TIME,
    audio::AudioManager,
    constants::MAX_DT,
    game::Game,
    input::{InputEvent, InputTracker, UpdateInput},
    render::{Scene, ViewportSize},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeEvent {
    FrameReady,
    QuitRequested,
}

pub struct GameRuntime {
    command_tx: Sender<RuntimeCommand>,
    scenes: SceneMailbox,
    handle: Option<JoinHandle<()>>,
}

#[derive(Clone, Default)]
struct SceneMailbox {
    buffers: Arc<Mutex<DoubleBuffer>>,
}

#[derive(Default)]
struct DoubleBuffer {
    front: Option<Scene>,
    back: Option<Scene>,
}

enum RuntimeCommand {
    Input(InputEvent),
    Resize(ViewportSize),
    ClearInput,
    Shutdown,
}

struct RuntimeWorker {
    command_rx: Receiver<RuntimeCommand>,
    scenes: SceneMailbox,
    event_proxy: EventLoopProxy<RuntimeEvent>,
    game: Game,
    audio: AudioManager,
    input_tracker: InputTracker,
}

impl GameRuntime {
    pub fn spawn(size: ViewportSize, event_proxy: EventLoopProxy<RuntimeEvent>) -> Result<Self> {
        let (command_tx, command_rx) = mpsc::channel();
        let scenes = SceneMailbox::default();
        let worker_scenes = scenes.clone();
        let game = Game::load().context("loading Battlezone game state")?;
        let handle = thread::Builder::new()
            .name(String::from("battlezone-game"))
            .spawn(move || {
                RuntimeWorker::new(command_rx, worker_scenes, event_proxy, size, game).run()
            })
            .context("spawning Battlezone game thread")?;

        Ok(Self {
            command_tx,
            scenes,
            handle: Some(handle),
        })
    }

    pub fn send_input(&self, event: InputEvent) {
        self.send(RuntimeCommand::Input(event));
    }

    pub fn resize(&self, size: ViewportSize) {
        self.send(RuntimeCommand::Resize(size));
    }

    pub fn clear_input(&self) {
        self.send(RuntimeCommand::ClearInput);
    }

    pub fn take_scene(&self) -> Option<Scene> {
        self.scenes.take()
    }

    pub fn shutdown(&mut self) {
        self.send(RuntimeCommand::Shutdown);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    fn send(&self, command: RuntimeCommand) {
        let _ = self.command_tx.send(command);
    }
}

impl Drop for GameRuntime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl SceneMailbox {
    fn publish(&self, scene: Scene) {
        let mut buffers = self
            .buffers
            .lock()
            .expect("scene mailbox lock should not be poisoned");
        buffers.publish(scene);
    }

    fn take(&self) -> Option<Scene> {
        let mut buffers = self
            .buffers
            .lock()
            .expect("scene mailbox lock should not be poisoned");
        buffers.take()
    }
}

impl DoubleBuffer {
    fn publish(&mut self, scene: Scene) {
        self.back = Some(scene);
        std::mem::swap(&mut self.front, &mut self.back);
    }

    fn take(&mut self) -> Option<Scene> {
        self.front.take()
    }
}

impl RuntimeWorker {
    fn new(
        command_rx: Receiver<RuntimeCommand>,
        scenes: SceneMailbox,
        event_proxy: EventLoopProxy<RuntimeEvent>,
        size: ViewportSize,
        mut game: Game,
    ) -> Self {
        game.set_viewport(size.width, size.height);

        Self {
            command_rx,
            scenes,
            event_proxy,
            game,
            audio: AudioManager::new(),
            input_tracker: InputTracker::new(),
        }
    }

    fn run(mut self) {
        drain_audio_events(&mut self.game, &mut self.audio);
        self.publish_frame();

        let frame_time = ORIGINAL_FRAME_TIME;
        let frame_duration = Duration::from_secs_f32(frame_time);
        let mut accumulator = 0.0f32;
        let mut last_tick = Instant::now();

        loop {
            let frame_started = Instant::now();
            let Some(input) = self.collect_frame_input() else {
                break;
            };
            if input.quit_requested {
                let _ = self.event_proxy.send_event(RuntimeEvent::QuitRequested);
                break;
            }

            let dt = last_tick.elapsed().as_secs_f32().min(MAX_DT);
            last_tick = Instant::now();
            accumulator = (accumulator + dt).min(frame_time * 6.0);

            let fixed_steps = consume_fixed_steps(&mut accumulator, frame_time);
            let updated = fixed_steps > 0;
            if updated {
                run_fixed_updates(
                    &mut self.game,
                    &mut self.audio,
                    frame_time,
                    fixed_steps,
                    input.clone(),
                );
            }

            self.publish_frame();
            sleep_until_next_frame(frame_started, frame_duration);
        }
    }

    fn collect_frame_input(&mut self) -> Option<UpdateInput> {
        let mut input = UpdateInput::default();
        while let Ok(command) = self.command_rx.try_recv() {
            match command {
                RuntimeCommand::Input(event) => self.input_tracker.handle_event(event, &mut input),
                RuntimeCommand::Resize(size) => self.resize(size),
                RuntimeCommand::ClearInput => self.input_tracker.clear_held_state(),
                RuntimeCommand::Shutdown => return None,
            }
        }
        self.input_tracker.apply_held_state(&mut input);
        Some(input)
    }

    fn resize(&mut self, size: ViewportSize) {
        self.game.set_viewport(size.width, size.height);
    }

    fn publish_frame(&mut self) {
        self.scenes.publish(self.game.frame());
        let _ = self.event_proxy.send_event(RuntimeEvent::FrameReady);
    }
}

fn sleep_until_next_frame(frame_started: Instant, frame_duration: Duration) {
    let elapsed = frame_started.elapsed();
    if elapsed < frame_duration {
        thread::sleep(frame_duration - elapsed);
    }
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
    use super::{SceneMailbox, consume_fixed_steps, repeated_input};
    use crate::{
        input::UpdateInput,
        math::Vec3,
        render::{Camera, Scene},
    };

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

    #[test]
    fn scene_mailbox_returns_latest_published_scene_once() {
        let scenes = SceneMailbox::default();
        scenes.publish(test_scene(0.0));
        scenes.publish(test_scene(1.0));

        let latest = scenes.take().expect("latest scene should be available");
        assert_eq!(latest.camera.heading, 1.0);
        assert!(scenes.take().is_none());
    }

    fn test_scene(heading: f32) -> Scene {
        Scene::empty(Camera {
            position: Vec3::new(0.0, 0.0, 0.0),
            heading,
        })
    }
}
