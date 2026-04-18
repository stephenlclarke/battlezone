# Battlezone

[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_battlezone&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_battlezone)
[![Bugs](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_battlezone&metric=bugs)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_battlezone)
[![Code Smells](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_battlezone&metric=code_smells)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_battlezone)
[![Coverage](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_battlezone&metric=coverage)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_battlezone)
[![Duplicated Lines (%)](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_battlezone&metric=duplicated_lines_density)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_battlezone)
[![Lines of Code](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_battlezone&metric=ncloc)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_battlezone)
[![Reliability Rating](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_battlezone&metric=reliability_rating)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_battlezone)
[![Security Rating](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_battlezone&metric=security_rating)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_battlezone)
[![Technical Debt](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_battlezone&metric=sqale_index)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_battlezone)
[![Maintainability Rating](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_battlezone&metric=sqale_rating)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_battlezone)
[![Vulnerabilities](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_battlezone&metric=vulnerabilities)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_battlezone)
![Repo Visitors](https://visitor-badge.laobi.icu/badge?page_id=stephenlclarke.battlezone)

---

This is a self-contained Rust implementation of Atari's original Battlezone,
rendered through the Kitty graphics protocol.

The current game uses a native Rust state machine for twin-stick tread
movement, title and high-score screens, battlefield layout, radar, enemy
behaviors, saucer timings, and scoring/lives flow. During development the
obstacle layout, attract-screen data, and rule tables were extracted from the
original arcade ROM set, but the shipped application no longer depends on any
ROM files at runtime.

![Battlezone](docs/battlezone.png)

<!-- markdownlint-disable MD033 -->
<p align="center">
  <img
    src="docs/start-sequence.gif"
    alt="Battlezone attract, title, high-score, and gameplay sequence"
  />
</p>
<!-- markdownlint-enable MD033 -->

Run targets:

- `cargo run`
- `make run`
- `cargo test`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `make ci`
- `make coverage`
- `make sq-ci`
- `make sq`
- `make readme-media`

Run this inside `kitty`, `ghostty`, `warp` or another terminal that supports the
Kitty graphics protocol.

## Install

Install directly from git with Cargo:

- `cargo install --git https://github.com/stephenlclarke/battlezone battlezone`

`cargo install` builds with Cargo's release profile by default. Do not pass
`--debug` unless you explicitly want a slower debug build.

After installation, run the game with:

- `battlezone`

## Controls

Arcade play controls:

- `1` or `Enter`: start from the title screen
- `Q` / `A`: left tread forward / reverse
- `P` / `L`: right tread forward / reverse
- `Up`: both treads forward
- `Down`: both treads reverse
- `Left`: pivot left
- `Right`: pivot right
- `Space`: fire
- `Esc`: quit

On terminals with key press/release reporting, holding `Q` + `P` together drives
both treads forward like `Up`, and holding `A` + `L` together drives both
treads backward like `Down`. On simpler terminals, use the arrow keys for
reliable combined-tread movement.

## XYZZY Mode

After starting a game, type `X`, `Y`, `Z`, `Z`, `Y` to toggle hidden
`XYZZY` mode on or off.

Typing `XYZZY` a second time turns the mode off and resets every secret option
back to its default state. If you activate `xyzzy` again later, it starts clean
with all hidden options disabled and the fire-rate boost reset to its base
level.

Letter-key controls accept either upper- or lower-case input.

Extra keys while `xyzzy` mode is active:

- `F`: increase the hidden fire-rate level with each press, up to a capped
  maximum.
- `G`: toggle god mode. While active, enemy units are drawn in red, enemy
  projectiles are orange, your projectiles are yellow, and enemy fire cannot
  kill the player tank.
- `H`: toggle hidden autopilot. While active, the tank will steer toward the
  current enemy, try to maintain a firing solution, and bias away from
  incoming shots and exposed positions.

## Notes

- This project is a native implementation, not a 6502 emulator.
- The battlefield obstacle coordinates, bonus-tank defaults, missile threshold,
  attract-screen strings, title/high-score layouts, and title-logo meshes were
  extracted from the original arcade data and flattened into native Rust
  modules/assets.
- Audio is synthesized in-process with `rodio`, so no platform-specific helper
  binaries are required.
- High scores now persist between runs in
  `~/.xyzzy/battlezone/high_scores.txt`; set `BATTLEZONE_DATA_DIR` to redirect
  that file for local experiments or tests.
- If `battlezone` is not found after installation, ensure `~/.cargo/bin` is on
  your `PATH`.

## SonarQube

- `make sq-ci` generates the Cobertura coverage report used by the SonarCloud
  workflow in CI.
- `make sq` runs the same coverage step locally and then invokes
  `sonar-scanner`.
- Local SonarQube scans require `cargo-llvm-cov`, `sonar-scanner`, and a
  `SONAR_TOKEN` environment variable.

## Source Materials

These references were used for reverse engineering, rules verification, attract
screen reconstruction, and extraction of historical arcade data while keeping
the final runtime self-contained:

- [Battlezone disassembly project](https://6502disassembly.com/va-battlezone/):
  primary gameplay notes covering scoring, enemy progression, saucer behavior,
  sound behavior, battlefield layout, and arcade quirks.
- [Battlezone strings and characters](https://6502disassembly.com/va-battlezone/strings.html):
  exact title-screen, high-score, and initials-entry strings plus the arcade
  character set used to match the original text layout.
- [Battlezone objects](https://6502disassembly.com/va-battlezone/objects.html):
  object index list and vector-object notes used to recreate obstacles, tanks,
  missile/saucer shapes, and the three-part Battlezone title logo.
- [Battlezone revision notes](https://6502disassembly.com/va-battlezone/rev1.html):
  revision-specific behavior notes, including the high-score tank-icon change
  between ROM revisions.
- [MAME Battlezone driver](https://raw.githubusercontent.com/mamedev/mame/master/src/mame/atari/bzone.cpp):
  machine layout, memory map, and default DIP-switch settings for the arcade
  cabinet.
- [Battle Zone (rev 2) ROM archive](https://www.retrostic.com/roms/mame/battle-zone-41678):
  source page for the historical `bzone.zip` MAME ROM set used during
  extraction and reference work. The shipped game does not bundle or load ROM
  files at runtime.

## Customisation

Arcade defaults ship in `assets/arcade/`. Copy the files you want to change
into `~/.xyzzy/battlezone/` to override them locally. Key/value files are
merged on top of the embedded defaults, so you only need to include the keys
you want to change. Layout files are replaced whole. If `BATTLEZONE_DATA_DIR`
is set, the game reads these files from that directory instead.

### `arcade-rules.txt`

`starting_lives`
Default: `3`
Meaning: number of tanks granted at the start of a credit.

`missile_score_threshold`
Default: `10000`
Meaning: score where guided missiles start appearing.

`missile_nastier_delta`
Default: `25000`
Meaning: additional score needed before missiles switch to the nastier
threshold.

`bonus_tank_thresholds`
Default: `15000,100000`
Meaning: score thresholds for the first and second bonus tanks.

`saucer_score_threshold`
Default: `2000`
Meaning: score where saucers begin to join the battle.

`near_spawn_distance`
Default: `47.996094`
Meaning: minimum spawn radius used for newly created enemies.

`far_spawn_distance`
Default: `95.996094`
Meaning: far spawn radius used for distant enemy placement.

`strings`
Default:

```text
PRESS START|HIGH SCORES|GREAT SCORE|ENTER YOUR INITIALS|
CHANGE LETTER WITH RIGHT HAND CONTROLLER|SELECT LETTER WITH FIRE BUTTON|
ENEMY IN RANGE|ENEMY TO LEFT|ENEMY TO RIGHT|ENEMY TO REAR|
MOTION BLOCKED BY OBJECT
```

Meaning: title, HUD, and initials-entry strings extracted from the arcade
data.

### `battlefield.txt`

This file is a whole-layout override rather than a partial key/value merge. If
`~/.xyzzy/battlezone/battlefield.txt` exists, it fully replaces the embedded
21-row obstacle layout. Each default row in `assets/arcade/battlefield.txt`
uses `kind x z heading_deg radius` to define one obstacle’s type, position,
rotation, and collision radius.

## Platform Support

The game is intended for Unix-like environments with a terminal that speaks the
Kitty graphics protocol.

macOS is the primary target because it has been the main development platform.
Linux should also work, but the audio and terminal stack still need broader
real-world validation.

For automated docs media generation or headless regression work, use the
examples under `examples/` rather than trying to capture an interactive terminal
session directly.
