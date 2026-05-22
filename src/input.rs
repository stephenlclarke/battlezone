//! Keyboard input translation for the Battlezone application.

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputKey {
    Character(char),
    Up,
    Down,
    Left,
    Right,
    Enter,
    Escape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputPhase {
    Pressed,
    Released,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputEvent {
    pub key: InputKey,
    pub phase: InputPhase,
}

impl InputEvent {
    pub fn pressed(key: InputKey) -> Self {
        Self {
            key,
            phase: InputPhase::Pressed,
        }
    }

    pub fn released(key: InputKey) -> Self {
        Self {
            key,
            phase: InputPhase::Released,
        }
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

#[derive(Default)]
pub struct InputTracker {
    held: HeldInputState,
}

impl InputTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_event(&mut self, event: InputEvent, input: &mut UpdateInput) {
        let pressed = event.phase == InputPhase::Pressed;

        match event.key {
            InputKey::Character(character) => self.handle_char_key(character, pressed, input),
            InputKey::Up => self.held.forward = pressed,
            InputKey::Down => self.held.backward = pressed,
            InputKey::Right => self.handle_turn_key(true, pressed, input),
            InputKey::Left => self.handle_turn_key(false, pressed, input),
            InputKey::Enter if pressed => trigger_start(input),
            InputKey::Escape if pressed => input.quit_requested = true,
            _ => {}
        }
    }

    pub fn apply_held_state(&self, input: &mut UpdateInput) {
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

    pub fn clear_held_state(&mut self) {
        self.held = HeldInputState::default();
    }

    fn handle_char_key(&mut self, character: char, pressed: bool, input: &mut UpdateInput) {
        let character = character.to_ascii_lowercase();
        if pressed && character.is_ascii_alphabetic() {
            input.typed_chars.push(character);
        }

        match character {
            'q' => self.held.left_tread_forward = pressed,
            'a' => self.held.left_tread_backward = pressed,
            'p' => self.handle_tread_key(false, true, pressed, input),
            'l' => self.handle_tread_key(false, false, pressed, input),
            'h' if pressed => input.autopilot_toggle_requested = true,
            ' ' => self.handle_fire_key(pressed, input),
            '1' if pressed => trigger_start(input),
            _ => {}
        }
    }

    fn handle_tread_key(
        &mut self,
        left: bool,
        forward: bool,
        pressed: bool,
        input: &mut UpdateInput,
    ) {
        match (left, forward) {
            (true, true) => self.held.left_tread_forward = pressed,
            (true, false) => self.held.left_tread_backward = pressed,
            (false, true) => {
                self.held.right_tread_forward = pressed;
                if pressed {
                    input.initials_previous = true;
                }
            }
            (false, false) => {
                self.held.right_tread_backward = pressed;
                if pressed {
                    input.initials_next = true;
                }
            }
        }
    }

    fn handle_turn_key(&mut self, right: bool, pressed: bool, input: &mut UpdateInput) {
        if right {
            self.held.turn_right = pressed;
            if pressed {
                input.initials_next = true;
            }
        } else {
            self.held.turn_left = pressed;
            if pressed {
                input.initials_previous = true;
            }
        }
    }

    fn handle_fire_key(&mut self, pressed: bool, input: &mut UpdateInput) {
        self.held.fire = pressed;
        if pressed {
            input.fire = true;
            input.initials_confirm = true;
        }
    }
}

fn trigger_start(input: &mut UpdateInput) {
    input.start_requested = true;
    input.initials_confirm = true;
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
    use super::{InputEvent, InputKey, InputTracker, UpdateInput};

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
        let mut tracker = InputTracker::new();
        let mut input = UpdateInput::default();
        tracker.handle_event(InputEvent::pressed(InputKey::Character('Q')), &mut input);
        assert!(!input.quit_requested);
        assert!(tracker.held.left_tread_forward);

        let mut input = UpdateInput::default();
        tracker.handle_event(InputEvent::pressed(InputKey::Escape), &mut input);
        assert!(input.quit_requested);
    }

    #[test]
    fn h_requests_autopilot_toggle() {
        let mut tracker = InputTracker::new();
        let mut input = UpdateInput::default();
        tracker.handle_event(InputEvent::pressed(InputKey::Character('H')), &mut input);
        assert!(input.autopilot_toggle_requested);
    }

    #[test]
    fn held_space_keeps_fire_active_until_release() {
        let mut tracker = InputTracker::new();
        let mut input = UpdateInput::default();
        tracker.handle_event(InputEvent::pressed(InputKey::Character(' ')), &mut input);
        assert!(tracker.held.fire);
        assert!(input.fire);

        let mut held_input = UpdateInput::default();
        tracker.apply_held_state(&mut held_input);
        assert!(held_input.fire);

        tracker.handle_event(
            InputEvent::released(InputKey::Character(' ')),
            &mut UpdateInput::default(),
        );
        assert!(!tracker.held.fire);
    }

    #[test]
    fn releasing_a_track_key_clears_its_held_state() {
        let mut tracker = InputTracker::new();
        tracker.handle_event(
            InputEvent::pressed(InputKey::Character('P')),
            &mut UpdateInput::default(),
        );
        assert!(tracker.held.right_tread_forward);

        tracker.handle_event(
            InputEvent::released(InputKey::Character('P')),
            &mut UpdateInput::default(),
        );
        assert!(!tracker.held.right_tread_forward);
    }

    #[test]
    fn uppercase_secret_letters_are_stored_lowercased() {
        let mut tracker = InputTracker::new();
        let mut input = UpdateInput::default();

        for character in ['X', 'Y', 'Z'] {
            tracker.handle_event(
                InputEvent::pressed(InputKey::Character(character)),
                &mut input,
            );
        }

        assert_eq!(input.typed_chars, vec!['x', 'y', 'z']);
    }

    #[test]
    fn start_and_turn_keys_set_one_shot_actions() {
        let mut tracker = InputTracker::new();
        let mut input = UpdateInput::default();

        tracker.handle_event(InputEvent::pressed(InputKey::Enter), &mut input);
        tracker.handle_event(InputEvent::pressed(InputKey::Left), &mut input);

        assert!(input.start_requested);
        assert!(input.initials_confirm);
        assert!(input.initials_previous);

        tracker.apply_held_state(&mut input);
        assert!(input.turn_left);
    }

    #[test]
    fn release_events_do_not_type_secret_letters() {
        let mut tracker = InputTracker::new();
        let mut input = UpdateInput::default();
        tracker.handle_event(InputEvent::released(InputKey::Character('q')), &mut input);
        assert_eq!(input.left_tread_axis(), 0);
        assert!(input.typed_chars.is_empty());
    }

    #[test]
    fn clear_held_state_releases_all_controls() {
        let mut tracker = InputTracker::new();
        tracker.handle_event(
            InputEvent::pressed(InputKey::Character('q')),
            &mut UpdateInput::default(),
        );
        tracker.handle_event(
            InputEvent::pressed(InputKey::Character('p')),
            &mut UpdateInput::default(),
        );

        tracker.clear_held_state();

        let mut input = UpdateInput::default();
        tracker.apply_held_state(&mut input);
        assert_eq!(input.left_tread_axis(), 0);
        assert_eq!(input.right_tread_axis(), 0);
    }
}
