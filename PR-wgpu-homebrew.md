# feat(render): port Battlezone to wgpu with Homebrew support

## Motivation

See ISSUE-wgpu-homebrew.md. Kitty presentation and terminal input prevent standalone window play, and the existing Homebrew release pin predates the port.

## Implementation

Use winit lifecycle and held keyboard controls with fixed game steps, clear input on focus loss, resize the game viewport, and present RGBA rasters using a persistent wgpu texture and full-screen shader. Remove the Kitty protocol and terminal dependencies. Preserve CPU scene rasterization and the headless documentation generator. Add --help and --version commands that do not initialize audio, a display, or a GPU. Update installation instructions.

## Validation

make ci passes: 74 tests, formatting, and warnings-as-errors Clippy. Markdown lint passes with MD013 disabled to retain single-line prose. Live macOS smoke testing visibly confirmed attract rendering, Enter starting gameplay, rendering after window zoom, and Escape terminating the process. The companion tap change tests the installed help/version commands.

## Compatibility and remaining risks

Interactive play now needs a graphical desktop and compatible GPU. Linux and Windows runtime validation remains outstanding. GPU/window lifecycle is validated by the live smoke test rather than unit coverage; no coverage percentage is claimed. CPU rasterization remains capped at 960 x 720. Checked-in media predates the window port. Two existing Clippy style findings were corrected to pass the normal checks.
