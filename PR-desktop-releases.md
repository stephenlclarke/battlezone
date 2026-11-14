# ci(release): publish tested macOS ARM64 and Windows x86-64 downloads

## Motivation

See [ISSUE-desktop-releases.md](ISSUE-desktop-releases.md). Users need downloadable native window builds that are distinct from the historical Kitty release.

## Implementation

Confirm the active rendering path is GameRuntime -> Scene -> vector mesh -> wgpu render pass -> native window surface. No Kitty or crossterm backend remains. Set version 1.1.1 for the first published wgpu release.

Build only aarch64-apple-darwin and x86_64-pc-windows-msvc release assets on native runners. Test locked source, require tag/version agreement, check help identifies wgpu, package the executable with README.md and RELEASING.md, extract and check the packaged executable, and publish both archives with a verified checksum manifest. Allow manual retries of an existing tag and serialize runs for the same tag.

Test CI on macOS, Windows, and Linux. Add real subprocess tests for help, version, and invalid options, plus a Makefile workflow-lint target. Update README.md, AGENTS.md, and RELEASING.md with downloads, architecture names, installation, signing limits, and tag/retry steps.

## Validation

Local make ci passes with 105 tests, formatting, and warnings-as-errors Clippy on the integrated vector renderer. actionlint validates all workflows. Markdown lint uses MD013 disabled to retain single-line prose. The native release workflow must pass both platform builds, tests, and extracted-executable checks before it publishes downloads.

## Compatibility and remaining risks

Windows means 64-bit Intel/AMD, not 32-bit x86 or ARM64. New release assets are a macOS tar.gz and Windows zip; the earlier Windows release used a bare exe. Linux is still tested in CI and available from source/Homebrew but is no longer a new-release binary target. Historical release assets remain untouched.

Integrate the newer main-branch vector renderer rather than release the older develop-branch raster-upload implementation. Renderer source is retained from main; the CPU raster renderer remains for media generation and tests. Windows graphical play remains separate from CLI/build validation. Downloads are not Developer ID notarized or Authenticode signed. The separate Homebrew pin is unchanged.

## Windows validation follow-up

The first Windows run crashed with an access violation while runtime tests opened real audio devices concurrently. Inject AudioManager into RuntimeWorker so production opens one device on its worker thread and unit tests use the existing silent backend. Keep the actual audio-device smoke test separate. Tag v1.1.0 was not published because the Windows tests failed; the corrected source uses a new v1.1.1 tag. SonarCloud separately rejects the repository token with HTTP 403.
