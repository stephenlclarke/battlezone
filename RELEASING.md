# Releases

Battlezone uses winit and wgpu for native window presentation. Interactive play builds vector meshes and draws them directly with wgpu. A separate CPU raster renderer supports headless media generation and tests. There is no Kitty backend. Version 1.1.0 is the first tagged wgpu release; v1.0.0 is the historical terminal release.

## Published downloads

| Runner | Rust target | Asset |
| --- | --- | --- |
| macos-15 (ARM64) | aarch64-apple-darwin | battlezone-vX.Y.Z-aarch64-apple-darwin.tar.gz |
| windows-2025 (x86-64) | x86_64-pc-windows-msvc | battlezone-vX.Y.Z-x86_64-pc-windows-msvc.zip |

Both archives contain the executable, README.md, and RELEASING.md. The release also includes `battlezone-vX.Y.Z-SHA256SUMS.txt`, with SHA-256 entries for both archives. Downloads are published on [GitHub Releases](https://github.com/stephenlclarke/battlezone/releases), not just as temporary Actions artifacts. Linux, Intel Mac, 32-bit Windows, and Windows ARM64 are not current release targets.

The macOS executable is not a .app bundle. The workflow does not apply Developer ID notarization or Windows Authenticode signing. Do not describe these downloads as signed or notarized.

## Prepare a release

1. Finish the change on `develop`. Update the package version in Cargo.toml and refresh Cargo.lock with `cargo check --offline`. The tag must be exactly `v` followed by the package version, using three numeric components.
2. Update README.md, this guide, and any affected runtime or installation docs. Check the Homebrew formula separately if its source pin or test expectations need changing.
3. Run `make ci`, `make workflows`, and Markdown lint. CI also tests macOS ARM64, Windows x86-64, and Linux. Complete live window checks for rendering/input changes; CLI checks do not prove GPU presentation.
4. Commit and push the tested source. Tag the exact commit that should be released, then push that tag. For example, when Cargo.toml contains version 1.1.0:

```sh
git tag -a v1.1.0 -m "Battlezone 1.1.0"
git push origin v1.1.0
```

The workflow builds the tagged commit, even when the tag was made on develop. It does not merge develop into main. Do not move or reuse an existing release tag for different source.

## Build and publication checks

The Release workflow tests both targets, builds locked release binaries, and executes their display-free help/version commands. The version must match the tag and help must identify the wgpu window. It packages each binary with the README and release guide, extracts the archive, and executes the extracted binary's version command.

Only after both native jobs pass does the publication job download their artifacts, require both archives, generate and verify checksums, and create the GitHub Release with generated notes. When rerun for the same tag, it updates those named assets. Existing historical assets on an older release are not deleted.

## Retry an existing tag

Once the workflow is integrated into the default branch, use **Actions > Release > Run workflow**, select a branch containing the current workflow, and enter the existing tag (for example `v1.1.0`). Alternatively:

```sh
gh workflow run release.yml --ref develop -f tag=v1.1.0
```

Before default-branch integration, use the existing tag-triggered run's **Re-run jobs** action (or `gh run rerun <run-id>`).

Manual runs check out and verify the existing tag; they do not release the selected branch's untagged source. The workflow rejects a missing tag, a tag whose checked-out commit differs, or a version mismatch.

## Verify the published downloads

Download both archives and the checksum manifest from the same release. Check the hashes before extracting. On macOS use `shasum -a 256`; on Windows use PowerShell `Get-FileHash -Algorithm SHA256`. Run the extracted executable with `--help` and `--version`. Confirm the expected architecture and perform interactive smoke tests on a graphical desktop when available.

The application embeds its arcade data and synthesizes audio. No ROMs, external assets, or Rust toolchain are required to run a release download. Interactive play still needs a compatible GPU/driver and desktop. If no audio output is available, the game continues silently.

## Homebrew

The tap at [stephenlclarke/homebrew-tap](https://github.com/stephenlclarke/homebrew-tap) is maintained separately. A release does not automatically update its source archive, checksum, version, or tests. The currently documented stable formula is the earlier pinned wgpu build, version 1.0.0_1; its optional HEAD build follows develop. Keep installation instructions explicit about this distinction until the tap is updated.
