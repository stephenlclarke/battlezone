use std::io::Stdout;

use anyhow::Result;
use crossterm::{
    cursor::{Hide, Show},
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, size,
        supports_keyboard_enhancement,
    },
};

pub struct TerminalSession {
    keyboard_enhancement_supported: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalGeometry {
    pub cols: u16,
    pub rows: u16,
    pub pixel_width: u16,
    pub pixel_height: u16,
}

impl TerminalSession {
    pub fn enter(stdout: &mut Stdout) -> Result<Self> {
        enable_raw_mode()?;
        let mut keyboard_enhancement_supported = false;
        let result = (|| {
            keyboard_enhancement_supported = supports_keyboard_enhancement().unwrap_or(false);
            if keyboard_enhancement_supported {
                execute!(
                    stdout,
                    EnterAlternateScreen,
                    Hide,
                    PushKeyboardEnhancementFlags(
                        KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                            | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                    )
                )?;
            } else {
                execute!(stdout, EnterAlternateScreen, Hide)?;
            }
            Ok(Self {
                keyboard_enhancement_supported,
            })
        })();

        if result.is_err() {
            if keyboard_enhancement_supported {
                let _ = execute!(
                    stdout,
                    PopKeyboardEnhancementFlags,
                    Show,
                    LeaveAlternateScreen
                );
            } else {
                let _ = execute!(stdout, Show, LeaveAlternateScreen);
            }
            let _ = disable_raw_mode();
        }

        result
    }

    pub fn keyboard_enhancement_supported(&self) -> bool {
        self.keyboard_enhancement_supported
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let mut stdout = std::io::stdout();
        if self.keyboard_enhancement_supported {
            let _ = execute!(
                stdout,
                PopKeyboardEnhancementFlags,
                Show,
                LeaveAlternateScreen
            );
        } else {
            let _ = execute!(stdout, Show, LeaveAlternateScreen);
        }
        let _ = disable_raw_mode();
    }
}

pub fn geometry() -> Result<TerminalGeometry> {
    let (cols, rows) = size()?;
    let (pixel_width, pixel_height) = pixel_size();

    Ok(TerminalGeometry {
        cols,
        rows,
        pixel_width,
        pixel_height,
    })
}

#[cfg(unix)]
fn pixel_size() -> (u16, u16) {
    use std::os::fd::AsRawFd;

    let stdout = std::io::stdout();
    let fd = stdout.as_raw_fd();
    let mut winsize = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let result = unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut winsize) };
    if result == 0 {
        (winsize.ws_xpixel, winsize.ws_ypixel)
    } else {
        (0, 0)
    }
}

#[cfg(not(unix))]
fn pixel_size() -> (u16, u16) {
    (0, 0)
}

#[cfg(test)]
mod tests {
    use super::{TerminalGeometry, TerminalSession, pixel_size};

    #[test]
    fn keyboard_enhancement_flag_round_trips() {
        let session = TerminalSession {
            keyboard_enhancement_supported: true,
        };
        assert!(session.keyboard_enhancement_supported());

        std::mem::forget(session);
    }

    #[test]
    fn pixel_size_returns_a_pair() {
        let (width, height) = pixel_size();
        let _ = TerminalGeometry {
            cols: 80,
            rows: 24,
            pixel_width: width,
            pixel_height: height,
        };
    }
}
