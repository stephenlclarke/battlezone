# Battlezone needs a native window and an installable Homebrew build

## Problem

The application renders through Kitty and reads terminal keyboard events. The Homebrew tap pins the Kitty-based v1.0.0 release, so installing that formula cannot deliver the requested native wgpu port.

## Expected result

Preserve the game and raster visuals in a winit/wgpu window using the Defender lifecycle pattern. Homebrew must build an immutable published source archive and check the installed executable without opening a window.
