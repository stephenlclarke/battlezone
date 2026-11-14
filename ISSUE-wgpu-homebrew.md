# Battlezone needs a native window and an installable Homebrew build

## Problem

Before this change, the application rendered through Kitty and read terminal keyboard events. The Homebrew tap pinned the Kitty-based v1.0.0 release, so installing that formula could not deliver the requested native wgpu port.

## Resolution

The application now presents through wgpu. The published Homebrew formula pins port commit `457237526a80061a9469f227b98cad090616c44d` with revision 1; a source installation, installed help/version tests, and linkage checks passed on macOS.

## Expected result

Preserve the game and raster visuals in a winit/wgpu window using the Defender lifecycle pattern. Homebrew must build an immutable published source archive and check the installed executable without opening a window.
