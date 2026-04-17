//! Keyboard input translation for the Battlezone application.

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UpdateInput {
    pub forward: bool,
    pub backward: bool,
    pub turn_left: bool,
    pub turn_right: bool,
    pub left_tread_forward: bool,
    pub left_tread_backward: bool,
    pub right_tread_forward: bool,
    pub right_tread_backward: bool,
    pub fire: bool,
    pub start_requested: bool,
    pub quit_requested: bool,
    pub autopilot_toggle_requested: bool,
    pub initials_previous: bool,
    pub initials_next: bool,
    pub initials_confirm: bool,
    pub typed_chars: Vec<char>,
}

impl UpdateInput {
    pub fn left_tread_axis(&self) -> i8 {
        let direct = axis(self.left_tread_forward, self.left_tread_backward);
        if direct != 0 {
            return direct;
        }
        legacy_left_axis(self)
    }

    pub fn right_tread_axis(&self) -> i8 {
        let direct = axis(self.right_tread_forward, self.right_tread_backward);
        if direct != 0 {
            return direct;
        }
        legacy_right_axis(self)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct HeldInputState {
    forward: bool,
    backward: bool,
    turn_left: bool,
    turn_right: bool,
    left_tread_forward: bool,
    left_tread_backward: bool,
    right_tread_forward: bool,
    right_tread_backward: bool,
    fire: bool,
}

pub struct InputTracker {
    held_key_tracking: bool,
    held: HeldInputState,
}

impl InputTracker {
    pub fn new(held_key_tracking: bool) -> Self {
        Self {
            held_key_tracking,
            held: HeldInputState::default(),
        }
    }

    pub fn poll(&mut self) -> Result<UpdateInput> {
        let mut input = UpdateInput::default();

        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(key_event) => {
                    if self.held_key_tracking {
                        self.handle_key_event(key_event, &mut input);
                    } else {
                        self.handle_event_based_key_event(key_event, &mut input);
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }

        if self.held_key_tracking {
            self.apply_held_state(&mut input);
        }
        Ok(input)
    }

    fn apply_held_state(&self, input: &mut UpdateInput) {
        input.forward = self.held.forward;
        input.backward = self.held.backward;
        input.turn_left = self.held.turn_left;
        input.turn_right = self.held.turn_right;
        input.left_tread_forward = self.held.left_tread_forward;
        input.left_tread_backward = self.held.left_tread_backward;
        input.right_tread_forward = self.held.right_tread_forward;
        input.right_tread_backward = self.held.right_tread_backward;
        input.fire = self.held.fire;
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, input: &mut UpdateInput) {
        let pressed = matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat);
        if !pressed && key_event.kind != KeyEventKind::Release {
            return;
        }

        match key_event.code {
            KeyCode::Char(character) => {
                let character = character.to_ascii_lowercase();
                if pressed && character.is_ascii_alphabetic() {
                    input.typed_chars.push(character);
                }
                match character {
                    'q' => self.held.left_tread_forward = pressed,
                    'a' => self.held.left_tread_backward = pressed,
                    'p' => {
                        self.held.right_tread_forward = pressed;
                        if pressed {
                            input.initials_previous = true;
                        }
                    }
                    'l' => {
                        self.held.right_tread_backward = pressed;
                        if pressed {
                            input.initials_next = true;
                        }
                    }
                    'h' if pressed => input.autopilot_toggle_requested = true,
                    ' ' => {
                        self.held.fire = pressed;
                        if pressed {
                            input.fire = true;
                            input.initials_confirm = true;
                        }
                    }
                    '1' if pressed => {
                        input.start_requested = true;
                        input.initials_confirm = true;
                    }
                    _ => {}
                }
            }
            KeyCode::Up => self.held.forward = pressed,
            KeyCode::Down => self.held.backward = pressed,
            KeyCode::Right => {
                self.held.turn_right = pressed;
                if pressed {
                    input.initials_next = true;
                }
            }
            KeyCode::Left => {
                self.held.turn_left = pressed;
                if pressed {
                    input.initials_previous = true;
                }
            }
            KeyCode::Enter if pressed => {
                input.start_requested = true;
                input.initials_confirm = true;
            }
            KeyCode::Esc if pressed => input.quit_requested = true,
            _ => {}
        }
    }

    fn handle_event_based_key_event(&mut self, key_event: KeyEvent, input: &mut UpdateInput) {
        if !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return;
        }

        match key_event.code {
            KeyCode::Char(character) => {
                let character = character.to_ascii_lowercase();
                if character.is_ascii_alphabetic() {
                    input.typed_chars.push(character);
                }
                match character {
                    'q' => input.left_tread_forward = true,
                    'a' => input.left_tread_backward = true,
                    'p' => {
                        input.right_tread_forward = true;
                        input.initials_previous = true;
                    }
                    'l' => {
                        input.right_tread_backward = true;
                        input.initials_next = true;
                    }
                    'h' => input.autopilot_toggle_requested = true,
                    ' ' => {
                        input.fire = true;
                        input.initials_confirm = true;
                    }
                    '1' => {
                        input.start_requested = true;
                        input.initials_confirm = true;
                    }
                    _ => {}
                }
            }
            KeyCode::Up => input.forward = true,
            KeyCode::Down => input.backward = true,
            KeyCode::Right => {
                input.turn_right = true;
                input.initials_next = true;
            }
            KeyCode::Left => {
                input.turn_left = true;
                input.initials_previous = true;
            }
            KeyCode::Enter => {
                input.start_requested = true;
                input.initials_confirm = true;
            }
            KeyCode::Esc => input.quit_requested = true,
            _ => {}
        }
    }
}

fn axis(forward: bool, backward: bool) -> i8 {
    match (forward, backward) {
        (true, false) => 1,
        (false, true) => -1,
        _ => 0,
    }
}

fn legacy_left_axis(input: &UpdateInput) -> i8 {
    let mut axis = axis(input.forward, input.backward);
    if input.turn_left {
        axis -= 1;
    }
    if input.turn_right {
        axis += 1;
    }
    axis.clamp(-1, 1)
}

fn legacy_right_axis(input: &UpdateInput) -> i8 {
    let mut axis = axis(input.forward, input.backward);
    if input.turn_left {
        axis += 1;
    }
    if input.turn_right {
        axis -= 1;
    }
    axis.clamp(-1, 1)
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    use super::{InputTracker, UpdateInput};

    #[test]
    fn legacy_turn_maps_to_counter_rotating_treads() {
        let input = UpdateInput {
            turn_left: true,
            ..UpdateInput::default()
        };
        assert_eq!(input.left_tread_axis(), -1);
        assert_eq!(input.right_tread_axis(), 1);
    }

    #[test]
    fn direct_tread_input_overrides_legacy_aliases() {
        let input = UpdateInput {
            forward: true,
            left_tread_backward: true,
            right_tread_forward: true,
            ..UpdateInput::default()
        };
        assert_eq!(input.left_tread_axis(), -1);
        assert_eq!(input.right_tread_axis(), 1);
    }

    #[test]
    fn typed_chars_capture_secret_letters_lowercased() {
        let input = UpdateInput {
            typed_chars: vec!['x', 'y', 'z'],
            ..UpdateInput::default()
        };
        assert_eq!(input.typed_chars, vec!['x', 'y', 'z']);
    }

    #[test]
    fn simultaneous_track_keys_map_to_both_treads() {
        let input = UpdateInput {
            left_tread_forward: true,
            right_tread_forward: true,
            ..UpdateInput::default()
        };
        assert_eq!(input.left_tread_axis(), 1);
        assert_eq!(input.right_tread_axis(), 1);

        let input = UpdateInput {
            left_tread_backward: true,
            right_tread_backward: true,
            ..UpdateInput::default()
        };
        assert_eq!(input.left_tread_axis(), -1);
        assert_eq!(input.right_tread_axis(), -1);
    }

    #[test]
    fn q_controls_left_track_and_escape_quits() {
        let mut tracker = InputTracker::new(true);
        let mut input = UpdateInput::default();
        tracker.handle_key_event(
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
            &mut input,
        );
        assert!(!input.quit_requested);
        assert!(tracker.held.left_tread_forward);

        let mut input = UpdateInput::default();
        tracker.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), &mut input);
        assert!(input.quit_requested);
    }

    #[test]
    fn h_requests_autopilot_toggle() {
        let mut tracker = InputTracker::new(true);
        let mut input = UpdateInput::default();
        tracker.handle_key_event(
            KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
            &mut input,
        );
        assert!(input.autopilot_toggle_requested);
    }

    #[test]
    fn held_space_keeps_fire_active_until_release() {
        let mut tracker = InputTracker::new(true);
        let mut input = UpdateInput::default();
        tracker.handle_key_event(
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
            &mut input,
        );
        assert!(tracker.held.fire);
        assert!(input.fire);

        let mut held_input = UpdateInput::default();
        tracker.apply_held_state(&mut held_input);
        assert!(held_input.fire);

        tracker.handle_key_event(
            KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Release,
                state: crossterm::event::KeyEventState::NONE,
            },
            &mut UpdateInput::default(),
        );
        assert!(!tracker.held.fire);
    }
}
