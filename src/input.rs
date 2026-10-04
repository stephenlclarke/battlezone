//! Native window keyboard state.

use std::collections::HashSet;
use winit::keyboard::KeyCode;

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

#[derive(Default)]
pub struct InputTracker {
    held: HashSet<KeyCode>,
    pending: UpdateInput,
}

impl InputTracker {
    pub fn key(&mut self, key: KeyCode, pressed: bool, repeat: bool) {
        if !pressed {
            self.held.remove(&key);
            return;
        }
        self.held.insert(key);
        if repeat {
            return;
        }
        match key {
            KeyCode::Escape => self.pending.quit_requested = true,
            KeyCode::Enter | KeyCode::Digit1 => {
                self.pending.start_requested = true;
                self.pending.initials_confirm = true;
            }
            KeyCode::KeyH => self.pending.autopilot_toggle_requested = true,
            KeyCode::ArrowLeft | KeyCode::KeyP => self.pending.initials_previous = true,
            KeyCode::ArrowRight | KeyCode::KeyL => self.pending.initials_next = true,
            KeyCode::Space => {
                self.pending.fire = true;
                self.pending.initials_confirm = true;
            }
            _ => {}
        }
    }

    pub fn text(&mut self, text: &str) {
        self.pending.typed_chars.extend(
            text.chars()
                .filter(char::is_ascii_alphabetic)
                .map(|c| c.to_ascii_lowercase()),
        );
    }

    pub fn clear(&mut self) {
        self.held.clear();
        self.pending = UpdateInput::default();
    }

    pub fn drain(&mut self) -> UpdateInput {
        let mut input = std::mem::take(&mut self.pending);
        input.forward = self.held.contains(&KeyCode::ArrowUp);
        input.backward = self.held.contains(&KeyCode::ArrowDown);
        input.turn_left = self.held.contains(&KeyCode::ArrowLeft);
        input.turn_right = self.held.contains(&KeyCode::ArrowRight);
        input.left_tread_forward = self.held.contains(&KeyCode::KeyQ);
        input.left_tread_backward = self.held.contains(&KeyCode::KeyA);
        input.right_tread_forward = self.held.contains(&KeyCode::KeyP);
        input.right_tread_backward = self.held.contains(&KeyCode::KeyL);
        input.fire |= self.held.contains(&KeyCode::Space);
        input
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
    use super::*;
    #[test]
    fn held_controls_release_and_focus_loss() {
        let mut tracker = InputTracker::default();
        tracker.key(KeyCode::KeyQ, true, false);
        tracker.key(KeyCode::KeyP, true, false);
        tracker.key(KeyCode::Space, true, false);
        let input = tracker.drain();
        assert_eq!((input.left_tread_axis(), input.right_tread_axis()), (1, 1));
        assert!(input.fire && input.initials_confirm);
        assert!(!tracker.drain().initials_confirm);
        tracker.key(KeyCode::KeyQ, false, false);
        assert_eq!(tracker.drain().left_tread_axis(), 0);
        tracker.clear();
        assert_eq!(tracker.drain(), UpdateInput::default());
    }
    #[test]
    fn one_shots_ignore_repeat_and_capture_text() {
        let mut tracker = InputTracker::default();
        tracker.key(KeyCode::KeyH, true, false);
        tracker.text("XYZ");
        assert!(tracker.drain().autopilot_toggle_requested);
        tracker.key(KeyCode::KeyH, true, true);
        assert!(!tracker.drain().autopilot_toggle_requested);
        tracker.key(KeyCode::Escape, true, false);
        assert!(tracker.drain().quit_requested);
        tracker.text("XYZ");
        assert_eq!(tracker.drain().typed_chars, vec!['x', 'y', 'z']);
    }
    #[test]
    fn quick_fire_press_survives_release_before_tick() {
        let mut tracker = InputTracker::default();
        tracker.key(KeyCode::Space, true, false);
        tracker.key(KeyCode::Space, false, false);
        assert!(tracker.drain().fire);
        assert!(!tracker.drain().fire);
    }

    #[test]
    fn direct_tracks_override_arrows() {
        let input = UpdateInput {
            forward: true,
            left_tread_backward: true,
            ..UpdateInput::default()
        };
        assert_eq!((input.left_tread_axis(), input.right_tread_axis()), (-1, 1));
    }
}
