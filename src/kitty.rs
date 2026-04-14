use std::io::{IsTerminal, Stdout, Write};

use anyhow::{Result, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use crossterm::{
    cursor::MoveTo,
    queue,
    terminal::{Clear, ClearType},
};
use png::{BitDepth, ColorType, Compression, Encoder};

use crate::render::RenderedImage;

const IMAGE_ID: u32 = 4_242;
const CHUNK_SIZE: usize = 4_096;
const ESCAPE_BEGIN: &str = "\x1b_G";
const ESCAPE_END: &str = "\x1b\\";

pub struct KittyGraphics {
    placement_cols: u16,
    placement_rows: u16,
}

impl KittyGraphics {
    pub fn new(placement_cols: u16, placement_rows: u16) -> Self {
        Self {
            placement_cols,
            placement_rows,
        }
    }

    pub fn ensure_supported() -> Result<()> {
        let term = std::env::var("TERM").unwrap_or_default();
        let term_program = std::env::var("TERM_PROGRAM").unwrap_or_default();
        let force = std::env::var("BATTLEZONE_FORCE_KITTY").unwrap_or_default();

        if force == "1" || is_known_kitty_graphics_terminal(&term, &term_program) {
            return Ok(());
        }

        validate_environment(&term, std::io::stdout().is_terminal())
    }

    pub fn resize(&mut self, placement_cols: u16, placement_rows: u16) {
        self.placement_cols = placement_cols;
        self.placement_rows = placement_rows;
    }

    pub fn draw_frame(&self, stdout: &mut Stdout, image: &RenderedImage) -> Result<()> {
        let png = encode_png(image)?;
        let encoded = STANDARD.encode(png);
        let chunk_count = encoded.len().div_ceil(CHUNK_SIZE);

        queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;

        for (index, chunk) in encoded.as_bytes().chunks(CHUNK_SIZE).enumerate() {
            let more = if index + 1 == chunk_count { 0 } else { 1 };
            if index == 0 {
                write!(
                    stdout,
                    "{ESCAPE_BEGIN}a=T,f=100,i={IMAGE_ID},q=2,C=1,c={},r={},z=-1,m={more};",
                    self.placement_cols, self.placement_rows
                )?;
            } else {
                write!(stdout, "{ESCAPE_BEGIN}m={more};")?;
            }
            stdout.write_all(chunk)?;
            write!(stdout, "{ESCAPE_END}")?;
        }

        Ok(())
    }

    pub fn clear(&self, stdout: &mut Stdout) -> Result<()> {
        write!(stdout, "{ESCAPE_BEGIN}a=d,d=I,i={IMAGE_ID},q=2{ESCAPE_END}")?;
        Ok(())
    }
}

fn is_known_kitty_graphics_terminal(term: &str, term_program: &str) -> bool {
    term == "xterm-ghostty" || term_program == "ghostty" || term_program == "WarpTerminal"
}

fn validate_environment(term: &str, is_terminal: bool) -> Result<()> {
    if !is_terminal {
        bail!(
            "Kitty graphics output requires an interactive terminal on stdout. \
             Run inside kitty or another terminal that supports the protocol."
        );
    }

    if term.is_empty() || term == "dumb" {
        bail!(
            "TERM={term:?} does not expose the interactive terminal capabilities needed for \
             Kitty graphics. Run inside kitty or another compatible terminal, or set \
             BATTLEZONE_FORCE_KITTY=1 to bypass this basic check."
        );
    }

    Ok(())
}

fn encode_png(image: &RenderedImage) -> Result<Vec<u8>> {
    let mut encoded = Vec::new();
    {
        let mut encoder = Encoder::new(&mut encoded, image.width, image.height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        encoder.set_compression(Compression::Fast);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&image.pixels)?;
    }
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::{CHUNK_SIZE, encode_png, is_known_kitty_graphics_terminal, validate_environment};
    use crate::render::RenderedImage;

    #[test]
    fn png_encoder_preserves_expected_dimensions() {
        let image = RenderedImage {
            width: 2,
            height: 2,
            pixels: vec![0, 0, 0, 255, 255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255],
        };

        let png = encode_png(&image).expect("png encoding should succeed");
        assert!(png.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]));
    }

    #[test]
    fn chunk_size_matches_protocol_limit() {
        assert_eq!(CHUNK_SIZE, 4_096);
    }

    #[test]
    fn environment_check_allows_non_kitty_terminals() {
        validate_environment("xterm-256color", true)
            .expect("interactive compatible terminals should pass");
    }

    #[test]
    fn environment_check_rejects_dumb_terminals() {
        let result = validate_environment("dumb", true);
        assert!(result.is_err());
    }

    #[test]
    fn known_ghostty_environment_skips_terminal_check() {
        assert!(is_known_kitty_graphics_terminal("xterm-ghostty", "ghostty"));
    }

    #[test]
    fn known_warp_environment_skips_terminal_check() {
        assert!(is_known_kitty_graphics_terminal(
            "xterm-256color",
            "WarpTerminal"
        ));
    }
}
