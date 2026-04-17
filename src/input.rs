//! Keyboard input translation for the Battlezone application.

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UpdateInput {
    pub forward: bool,
    pub backward: bool,
    pub turn_left: bool,
    pub turn_right: bool,
    pub fire: bool,
    pub start_requested: bool,
    pub quit_requested: bool,
    pub initials_previous: bool,
    pub initials_next: bool,
    pub initials_confirm: bool,
}

pub fn poll_input() -> Result<UpdateInput> {
    let mut input = UpdateInput::default();

    while event::poll(Duration::ZERO)? {
        match event::read()? {
            Event::Key(key_event)
                if matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
            {
                match key_event.code {
                    KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => input.forward = true,
                    KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                        input.backward = true
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        input.turn_left = true;
                        input.initials_previous = true;
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        input.turn_right = true;
                        input.initials_next = true;
                    }
                    KeyCode::Left => {
                        input.turn_left = true;
                        input.initials_previous = true;
                    }
                    KeyCode::Right => {
                        input.turn_right = true;
                        input.initials_next = true;
                    }
                    KeyCode::Char(' ') => {
                        input.fire = true;
                        input.initials_confirm = true;
                    }
                    KeyCode::Enter | KeyCode::Char('1') => {
                        input.start_requested = true;
                        input.initials_confirm = true;
                    }
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                        input.quit_requested = true
                    }
                    _ => {}
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    Ok(input)
}
