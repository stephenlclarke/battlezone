# Publish native desktop downloads for the wgpu app

## Problem

The application uses wgpu, but its release setup and documentation do not clearly distinguish the current native window build from the v1.0.0 Kitty release. The existing workflow lacks native tests, packaged-executable checks, version/tag verification, and an explicit two-platform download contract.

## Expected result

Build and publish macOS ARM64 and Windows x86-64 archives on GitHub Releases, together with SHA-256 checksums. Test the tagged source and extracted executables on their native runners. Keep repository metadata and installation/release documentation aligned.
