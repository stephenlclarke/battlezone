//! Keyboard input translation for the Battlezone application.

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

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

pub fn poll_input() -> Result<UpdateInput> {
    let mut input = UpdateInput::default();

    while event::poll(Duration::ZERO)? {
        match event::read()? {
            Event::Key(key_event)
                if matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
            {
                match key_event.code {
                    KeyCode::Char(character) => {
                        let character = character.to_ascii_lowercase();
                        if character.is_ascii_alphabetic() {
                            input.typed_chars.push(character);
                        }
                        match character {
                            'w' => input.forward = true,
                            's' => input.backward = true,
                            'e' => input.left_tread_forward = true,
                            'd' => input.left_tread_backward = true,
                            'i' => {
                                input.right_tread_forward = true;
                                input.initials_previous = true;
                            }
                            'k' => {
                                input.right_tread_backward = true;
                                input.initials_next = true;
                            }
                            'a' => {
                                input.turn_left = true;
                                input.initials_previous = true;
                            }
                            ' ' => {
                                input.fire = true;
                                input.initials_confirm = true;
                            }
                            '1' => {
                                input.start_requested = true;
                                input.initials_confirm = true;
                            }
                            'q' => input.quit_requested = true,
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
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    Ok(input)
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
    use super::UpdateInput;

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
}
