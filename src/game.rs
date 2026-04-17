//! Owns the Battlezone game state, arcade rules, attract mode, and frame generation.

use std::f32::consts::{PI, TAU};

use crate::{
    arcade::{self, ArcadeTables, ObstacleKind},
    attract::{self, TextSize},
    constants::{INFO_COLOR, SCREEN_COLOR, SCREEN_COLOR_DIM, WARNING_COLOR},
    input::UpdateInput,
    math::{Vec3, forward, rotate_y},
    render::{BackgroundStyle, Camera, Scene, ScreenDot, ScreenLine, ScreenText, WorldLine},
};

const PLAYER_RADIUS: f32 = 2.4;
const PLAYER_EYE_HEIGHT: f32 = 2.8;
const PLAYER_MOVE_SPEED: f32 = 24.0;
const PLAYER_REVERSE_SPEED: f32 = 14.0;
const PLAYER_TREAD_TURN_RATE: f32 = 0.102;
const PLAYER_SHELL_SPEED: f32 = 92.0;
const ENEMY_SHELL_SPEED: f32 = 74.0;
const PLAYER_SHELL_LIFETIME: f32 = 2.2;
const ENEMY_SHELL_LIFETIME: f32 = 2.8;
const PLAYER_RESPAWN_DELAY: f32 = 2.0;
const PLAYER_DYING_DELAY: f32 = 2.4;
const WORLD_LIMIT: f32 = 124.0;
const RADAR_RADIUS: i32 = 54;
const HUD_MARGIN: i32 = 18;
const MESSAGE_DURATION: f32 = 1.6;
const TITLE_SCREEN_DURATION: f32 = 6.5;
const HIGH_SCORE_SCREEN_DURATION: f32 = 5.0;
const ATTRACT_CYCLE_DURATION: f32 = TITLE_SCREEN_DURATION + HIGH_SCORE_SCREEN_DURATION;
const TITLE_LOGO_SCALE: f32 = 1.0;
const TITLE_LOGO_START_Y: f32 = -2.5;
const TITLE_LOGO_END_Y: f32 = 2.0;
const TITLE_LOGO_START_Z: f32 = 32.0;
const TITLE_LOGO_END_Z: f32 = 96.0;
const ARCADE_BLACK: [u8; 4] = [0, 0, 0, 255];
const ENEMY_SCORE_FULL_AGGRESSION_DELTA: f32 = 7_000.0;
const ENEMY_MAX_PATIENCE_SECONDS: f32 = 17.0;
const ENEMY_SPAWN_FIRE_DELAY: f32 = 2.0;
const ARCADE_OBJECT_TIMER_RATE: f32 = 15.0;
const OBJECT_SPIN_RATE: f32 = TAU / 32.0 * ARCADE_OBJECT_TIMER_RATE;
const SLOW_TANK_TREAD_ADVANCE_RATE: f32 = 0.9;
const SLOW_TANK_MIN_ADVANCE_DISTANCE: f32 = 18.0;
const SUPER_TANK_MIN_ADVANCE_DISTANCE: f32 = 28.0;
const SAUCER_HEIGHT: f32 = 0.0;
const SAUCER_HIT_RADIUS: f32 = 4.2;
const SAUCER_DIRECTION_CHANGE_MAX: f32 = 128.0 / ARCADE_OBJECT_TIMER_RATE;
const SAUCER_RESPAWN_DELAY_MAX: f32 = 256.0 / ARCADE_OBJECT_TIMER_RATE;
const ENEMY_OUT_OF_RANGE_DISTANCE: f32 = WORLD_LIMIT - 4.0;
const TANK_PRESSURE_TIMEOUT_MIN: f32 = 48.0;
const TANK_PRESSURE_TIMEOUT_MAX: f32 = 64.0;
const MISSILE_CHAIN_TIMEOUT_MIN: f32 = 16.0;
const MISSILE_CHAIN_TIMEOUT_MAX: f32 = 32.0;
const RESPAWN_ENEMY_SLEEP_SECONDS: f32 = 3.0;
const EASTER_EGG_CODE: [char; 5] = ['x', 'y', 'z', 'z', 'y'];
const SECRET_FIRE_LEVEL_MAX: u8 = 5;
const SECRET_FIRE_COOLDOWNS: [f32; 6] = [0.0, 0.18, 0.12, 0.08, 0.05, 0.035];
const AUTOPILOT_SHOT_CONE: f32 = 0.11;
const AUTOPILOT_PREDICTION_SECONDS: f32 = 0.6;
const GOD_ENEMY_COLOR: [u8; 4] = [255, 72, 72, 255];
const GOD_ENEMY_PROJECTILE_COLOR: [u8; 4] = [255, 168, 72, 255];
const GOD_PLAYER_PROJECTILE_COLOR: [u8; 4] = [255, 232, 72, 255];

const BOX_VERTICES: [Vec3; 8] = [
    Vec3::new(-1.0, 0.0, -1.0),
    Vec3::new(1.0, 0.0, -1.0),
    Vec3::new(1.0, 2.0, -1.0),
    Vec3::new(-1.0, 2.0, -1.0),
    Vec3::new(-1.0, 0.0, 1.0),
    Vec3::new(1.0, 0.0, 1.0),
    Vec3::new(1.0, 2.0, 1.0),
    Vec3::new(-1.0, 2.0, 1.0),
];

const BOX_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

const NARROW_PYRAMID_VERTICES: [Vec3; 5] = [
    Vec3::new(-1.2, 0.0, -1.4),
    Vec3::new(1.2, 0.0, -1.4),
    Vec3::new(1.2, 0.0, 1.4),
    Vec3::new(-1.2, 0.0, 1.4),
    Vec3::new(0.0, 3.0, 0.0),
];

const WIDE_PYRAMID_VERTICES: [Vec3; 5] = [
    Vec3::new(-1.8, 0.0, -1.8),
    Vec3::new(1.8, 0.0, -1.8),
    Vec3::new(1.8, 0.0, 1.8),
    Vec3::new(-1.8, 0.0, 1.8),
    Vec3::new(0.0, 2.4, 0.0),
];

const PYRAMID_EDGES: [(usize, usize); 8] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (0, 4),
    (1, 4),
    (2, 4),
    (3, 4),
];

const SLOW_TANK_VERTICES: [Vec3; 24] = [
    Vec3::new(1.0, -1.25, -1.438),
    Vec3::new(-1.0, -1.25, -1.438),
    Vec3::new(-1.0, -1.25, 1.891),
    Vec3::new(1.0, -1.25, 1.891),
    Vec3::new(1.109, -0.8125, -2.0),
    Vec3::new(-1.109, -0.8125, -2.0),
    Vec3::new(-1.109, -0.8125, 2.438),
    Vec3::new(1.109, -0.8125, 2.438),
    Vec3::new(0.6719, -0.4688, -1.328),
    Vec3::new(-0.6719, -0.4688, -1.328),
    Vec3::new(-0.6719, -0.4688, 1.328),
    Vec3::new(0.6719, -0.4688, 1.328),
    Vec3::new(0.3281, 0.1875, -1.0),
    Vec3::new(-0.3281, 0.1875, -1.0),
    Vec3::new(0.07812, -0.03125, -0.25),
    Vec3::new(-0.07812, -0.03125, -0.25),
    Vec3::new(-0.07812, -0.1875, 0.25),
    Vec3::new(0.07812, -0.1875, 0.25),
    Vec3::new(-0.07812, -0.03125, 2.188),
    Vec3::new(-0.07812, -0.1875, 2.188),
    Vec3::new(0.07812, -0.03125, 2.188),
    Vec3::new(0.07812, -0.1875, 2.188),
    Vec3::new(0.0, 0.1875, -1.0),
    Vec3::new(0.0, 0.3125, -1.0),
];

const SLOW_TANK_EDGES: [(usize, usize); 38] = [
    (23, 22),
    (12, 13),
    (14, 20),
    (20, 18),
    (18, 15),
    (15, 14),
    (14, 17),
    (17, 16),
    (16, 19),
    (19, 21),
    (21, 17),
    (15, 16),
    (19, 18),
    (20, 21),
    (3, 0),
    (0, 4),
    (4, 7),
    (7, 6),
    (6, 2),
    (2, 3),
    (3, 7),
    (7, 11),
    (11, 10),
    (10, 6),
    (6, 5),
    (5, 9),
    (9, 10),
    (10, 13),
    (13, 9),
    (9, 8),
    (8, 11),
    (11, 12),
    (12, 8),
    (8, 4),
    (4, 5),
    (5, 1),
    (1, 2),
    (1, 0),
];

const RADAR_VERTICES: [Vec3; 8] = [
    Vec3::new(0.1562, 0.3125, 0.0),
    Vec3::new(0.3125, 0.3906, 0.1562),
    Vec3::new(0.3125, 0.4688, 0.1562),
    Vec3::new(0.1562, 0.5469, 0.0),
    Vec3::new(-0.1562, 0.3125, 0.0),
    Vec3::new(-0.3125, 0.3906, 0.1562),
    Vec3::new(-0.3125, 0.4688, 0.1562),
    Vec3::new(-0.1562, 0.5469, 0.0),
];

const RADAR_EDGES: [(usize, usize); 11] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (0, 4),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (7, 3),
];

const SUPER_TANK_VERTICES: [Vec3; 25] = [
    Vec3::new(-0.7188, -1.25, 2.844),
    Vec3::new(-1.078, -1.25, -0.8906),
    Vec3::new(1.078, -1.25, -0.8906),
    Vec3::new(0.7188, -1.25, 2.844),
    Vec3::new(-0.8906, -0.3594, -0.8906),
    Vec3::new(0.8906, -0.3594, -0.8906),
    Vec3::new(0.0, -1.078, 2.141),
    Vec3::new(-0.5312, -0.4531, -0.5312),
    Vec3::new(-0.5312, -0.3594, -0.8906),
    Vec3::new(0.5312, -0.3594, -0.8906),
    Vec3::new(0.5312, -0.4531, -0.5312),
    Vec3::new(-0.3594, 0.1719, -0.5312),
    Vec3::new(-0.3594, 0.1719, -0.8906),
    Vec3::new(0.3594, 0.1719, -0.8906),
    Vec3::new(0.3594, 0.1719, -0.5312),
    Vec3::new(-0.1719, -0.1719, 2.5),
    Vec3::new(-0.1719, -0.1719, 0.1719),
    Vec3::new(0.1719, -0.1719, 0.1719),
    Vec3::new(0.1719, -0.1719, 2.5),
    Vec3::new(-0.1719, 0.0, 2.5),
    Vec3::new(-0.1719, 0.0, -0.1719),
    Vec3::new(0.1719, 0.0, -0.1719),
    Vec3::new(0.1719, 0.0, 2.5),
    Vec3::new(0.0, 0.1719, -0.8906),
    Vec3::new(0.0, 0.0, 0.0),
];

const SUPER_TANK_EDGES: [(usize, usize); 34] = [
    (0, 1),
    (1, 4),
    (4, 0),
    (0, 3),
    (3, 2),
    (2, 5),
    (5, 3),
    (2, 1),
    (4, 5),
    (9, 10),
    (10, 6),
    (6, 14),
    (14, 13),
    (13, 9),
    (9, 8),
    (8, 7),
    (7, 6),
    (6, 11),
    (11, 12),
    (12, 8),
    (12, 13),
    (14, 11),
    (19, 22),
    (22, 21),
    (21, 20),
    (20, 16),
    (16, 15),
    (15, 18),
    (18, 17),
    (17, 16),
    (15, 19),
    (22, 18),
    (17, 21),
    (23, 24),
];

const MISSILE_VERTICES: [Vec3; 7] = [
    Vec3::new(0.0, 0.0, -1.4),
    Vec3::new(0.0, 0.0, 1.8),
    Vec3::new(0.0, 3.0, -0.2),
    Vec3::new(-0.8, 0.6, -0.1),
    Vec3::new(0.8, 0.6, -0.1),
    Vec3::new(-0.8, 1.4, 0.6),
    Vec3::new(0.8, 1.4, 0.6),
];

const MISSILE_EDGES: [(usize, usize); 8] = [
    (0, 1),
    (0, 2),
    (1, 2),
    (3, 4),
    (3, 5),
    (4, 6),
    (5, 6),
    (3, 6),
];

const REAR_TREAD_0_VERTICES: [Vec3; 6] = [
    Vec3::new(1.078, -0.9219, -1.852),
    Vec3::new(-1.078, -0.9219, -1.852),
    Vec3::new(1.047, -1.078, -1.648),
    Vec3::new(-1.047, -1.078, -1.648),
    Vec3::new(1.008, -1.234, -1.438),
    Vec3::new(-1.008, -1.234, -1.438),
];

const REAR_TREAD_1_VERTICES: [Vec3; 6] = [
    Vec3::new(1.086, -0.8906, -1.898),
    Vec3::new(-1.086, -0.8906, -1.898),
    Vec3::new(1.055, -1.047, -1.695),
    Vec3::new(-1.055, -1.047, -1.695),
    Vec3::new(1.016, -1.203, -1.492),
    Vec3::new(-1.016, -1.203, -1.492),
];

const REAR_TREAD_2_VERTICES: [Vec3; 6] = [
    Vec3::new(1.102, -0.8438, -1.953),
    Vec3::new(-1.102, -0.8438, -1.953),
    Vec3::new(1.062, -1.0, -1.75),
    Vec3::new(-1.062, -1.0, -1.75),
    Vec3::new(1.031, -1.156, -1.547),
    Vec3::new(-1.031, -1.156, -1.547),
];

const REAR_TREAD_3_VERTICES: [Vec3; 6] = [
    Vec3::new(1.109, -0.8125, -2.0),
    Vec3::new(-1.109, -0.8125, -2.0),
    Vec3::new(1.07, -0.9688, -1.797),
    Vec3::new(-1.07, -0.9688, -1.797),
    Vec3::new(1.039, -1.125, -1.594),
    Vec3::new(-1.039, -1.125, -1.594),
];

const FRONT_TREAD_0_VERTICES: [Vec3; 6] = [
    Vec3::new(1.109, -0.8125, 2.438),
    Vec3::new(-1.109, -0.8125, 2.438),
    Vec3::new(1.07, -0.9688, 2.25),
    Vec3::new(-1.07, -0.9688, 2.25),
    Vec3::new(1.039, -1.125, 2.062),
    Vec3::new(-1.039, -1.125, 2.062),
];

const FRONT_TREAD_1_VERTICES: [Vec3; 6] = [
    Vec3::new(1.102, -0.8438, 2.391),
    Vec3::new(-1.102, -0.8438, 2.391),
    Vec3::new(1.062, -1.0, 2.203),
    Vec3::new(-1.062, -1.0, 2.203),
    Vec3::new(1.031, -1.156, 2.016),
    Vec3::new(-1.031, -1.156, 2.016),
];

const FRONT_TREAD_2_VERTICES: [Vec3; 6] = [
    Vec3::new(1.086, -0.8906, 2.344),
    Vec3::new(-1.086, -0.8906, 2.344),
    Vec3::new(1.055, -1.047, 2.156),
    Vec3::new(-1.055, -1.047, 2.156),
    Vec3::new(1.016, -1.203, 1.969),
    Vec3::new(-1.016, -1.203, 1.969),
];

const FRONT_TREAD_3_VERTICES: [Vec3; 6] = [
    Vec3::new(1.078, -0.9219, 2.297),
    Vec3::new(-1.078, -0.9219, 2.297),
    Vec3::new(1.047, -1.078, 2.109),
    Vec3::new(-1.047, -1.078, 2.109),
    Vec3::new(1.008, -1.234, 1.922),
    Vec3::new(-1.008, -1.234, 1.922),
];

const TREAD_EDGES: [(usize, usize); 3] = [(0, 1), (2, 3), (4, 5)];

const SAUCER_VERTICES: [Vec3; 17] = [
    Vec3::new(0.0, -0.1562, -0.4688),
    Vec3::new(-0.3125, -0.1562, -0.3125),
    Vec3::new(-0.4688, -0.1562, 0.0),
    Vec3::new(-0.3125, -0.1562, 0.3125),
    Vec3::new(0.0, -0.1562, 0.4688),
    Vec3::new(0.3125, -0.1562, 0.3125),
    Vec3::new(0.4688, -0.1562, 0.0),
    Vec3::new(0.3125, -0.1562, -0.3125),
    Vec3::new(0.0, 0.3125, -1.875),
    Vec3::new(-1.328, 0.3125, -1.328),
    Vec3::new(-1.875, 0.3125, 0.0),
    Vec3::new(-1.328, 0.3125, 1.328),
    Vec3::new(0.0, 0.3125, 1.875),
    Vec3::new(1.328, 0.3125, 1.328),
    Vec3::new(1.875, 0.3125, 0.0),
    Vec3::new(1.328, 0.3125, -1.328),
    Vec3::new(0.0, 1.094, 0.0),
];

const SAUCER_EDGES: [(usize, usize); 32] = [
    (16, 8),
    (8, 9),
    (9, 16),
    (16, 10),
    (10, 11),
    (11, 16),
    (16, 12),
    (12, 13),
    (13, 16),
    (16, 14),
    (14, 15),
    (15, 16),
    (0, 7),
    (7, 15),
    (15, 8),
    (8, 0),
    (0, 1),
    (1, 9),
    (9, 10),
    (10, 2),
    (2, 3),
    (3, 11),
    (11, 12),
    (12, 4),
    (4, 5),
    (5, 13),
    (13, 14),
    (14, 6),
    (6, 7),
    (6, 5),
    (4, 3),
    (2, 1),
];

const LANDSCAPE_POINTS: [Vec3; 12] = [
    Vec3::new(-260.0, 18.0, 200.0),
    Vec3::new(-220.0, 34.0, 214.0),
    Vec3::new(-180.0, 26.0, 228.0),
    Vec3::new(-120.0, 58.0, 235.0),
    Vec3::new(-70.0, 44.0, 228.0),
    Vec3::new(-12.0, 68.0, 240.0),
    Vec3::new(40.0, 52.0, 234.0),
    Vec3::new(86.0, 42.0, 225.0),
    Vec3::new(140.0, 54.0, 214.0),
    Vec3::new(196.0, 32.0, 208.0),
    Vec3::new(240.0, 18.0, 204.0),
    Vec3::new(284.0, 28.0, 198.0),
];

const VOLCANO_BASE: Vec3 = Vec3::new(-210.0, 0.0, -150.0);
const VOLCANO_TOP: Vec3 = Vec3::new(-210.0, 58.0, -150.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameEvent {
    TitleScreenEntered,
    GameStarted,
    PlayerShot,
    EnemyShot,
    EnemyDestroyed,
    PlayerDestroyed,
    SaucerDestroyed,
    RadarPing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Title,
    Playing,
    EnteringInitials,
    GameOver,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AttractScene {
    Title,
    HighScores,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayerState {
    Alive,
    Respawning,
    Dying,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnemyKind {
    SlowTank,
    SuperTank,
    Missile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SaucerState {
    Inactive,
    Alive,
    Dying,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProjectileOwner {
    Player,
    Enemy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HighScoreEntry {
    pub initials: String,
    pub score: u32,
}

#[derive(Clone, Copy, Debug)]
struct Player {
    position: Vec3,
    heading: f32,
    state: PlayerState,
    timer: f32,
    spawn_grace_timer: f32,
}

#[derive(Clone, Copy, Debug)]
struct Enemy {
    kind: EnemyKind,
    position: Vec3,
    heading: f32,
    desired_heading: f32,
    radar_heading: f32,
    tread_phase: f32,
    sleep_timer: f32,
    state_timer: f32,
    decision_timer: f32,
    shot_cooldown: f32,
    missile_height: f32,
    missile_vertical_velocity: f32,
    alive: bool,
}

#[derive(Clone, Copy, Debug)]
struct Saucer {
    position: Vec3,
    velocity: Vec3,
    heading: f32,
    timer: f32,
    state: SaucerState,
}

#[derive(Clone, Copy, Debug)]
struct Projectile {
    owner: ProjectileOwner,
    position: Vec3,
    velocity: Vec3,
    ttl: f32,
}

#[derive(Clone, Debug)]
struct InitialsEntry {
    letters: [u8; 3],
    cursor: usize,
    blink_timer: f32,
    blink_visible: bool,
    score: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct EasterEggState {
    active: bool,
    sequence_index: usize,
    invincible: bool,
    fire_level: u8,
}

impl EasterEggState {
    fn toggle(&mut self) {
        self.active = !self.active;
        self.sequence_index = 0;
        self.reset_runtime_flags();
    }

    fn reset_runtime_flags(&mut self) {
        self.invincible = false;
        self.fire_level = 0;
    }
}

pub struct Game {
    arcade: &'static ArcadeTables,
    rng: fastrand::Rng,
    mode: Mode,
    player: Player,
    score: u32,
    enemy_score: u32,
    lives: u32,
    next_bonus_tank_index: usize,
    missile_launch_counter: u8,
    high_scores: Vec<HighScoreEntry>,
    enemy: Option<Enemy>,
    player_projectiles: Vec<Projectile>,
    player_shot_cooldown: f32,
    enemy_projectile: Option<Projectile>,
    tank_pressure_timer: f32,
    missile_chain_timer: f32,
    saucer: Saucer,
    title_timer: f32,
    game_over_timer: f32,
    prompt_timer: f32,
    prompt_visible: bool,
    blocked_timer: f32,
    radar_sweep_angle: f32,
    radar_ping_brightness: f32,
    viewport_width: u32,
    viewport_height: u32,
    initials: Option<InitialsEntry>,
    easter_egg: EasterEggState,
    autopilot: bool,
    events: Vec<GameEvent>,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        Self::with_seed(fastrand::u64(..))
    }

    pub fn with_seed(seed: u64) -> Self {
        let arcade = arcade::arcade_tables();
        let mut game = Self {
            arcade,
            rng: fastrand::Rng::with_seed(seed),
            mode: Mode::Title,
            player: Player {
                position: Vec3::new(0.0, 0.0, 0.0),
                heading: 0.0,
                state: PlayerState::Respawning,
                timer: PLAYER_RESPAWN_DELAY,
                spawn_grace_timer: PLAYER_RESPAWN_DELAY,
            },
            score: 0,
            enemy_score: 0,
            lives: arcade.starting_lives,
            next_bonus_tank_index: 0,
            missile_launch_counter: u8::MAX,
            high_scores: default_high_scores(),
            enemy: None,
            player_projectiles: Vec::new(),
            player_shot_cooldown: 0.0,
            enemy_projectile: None,
            tank_pressure_timer: 0.0,
            missile_chain_timer: 0.0,
            saucer: Saucer {
                position: Vec3::new(0.0, SAUCER_HEIGHT, 0.0),
                velocity: Vec3::new(0.0, 0.0, 0.0),
                heading: 0.0,
                timer: 4.0,
                state: SaucerState::Inactive,
            },
            title_timer: 0.0,
            game_over_timer: 0.0,
            prompt_timer: 0.0,
            prompt_visible: true,
            blocked_timer: 0.0,
            radar_sweep_angle: 0.0,
            radar_ping_brightness: 0.0,
            viewport_width: 640,
            viewport_height: 360,
            initials: None,
            easter_egg: EasterEggState::default(),
            autopilot: false,
            events: vec![GameEvent::TitleScreenEntered],
        };
        game.reset_title_world();
        game
    }

    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.viewport_width = width.max(320);
        self.viewport_height = height.max(180);
    }

    pub fn drain_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn update_with_input(&mut self, dt: f32, input: UpdateInput) {
        if self.mode == Mode::Playing {
            self.handle_easter_egg_input(&input.typed_chars);
            if self.easter_egg.active && input.autopilot_toggle_requested {
                self.autopilot = !self.autopilot;
            }
        }
        self.prompt_timer += dt;
        if self.prompt_timer >= 0.35 {
            self.prompt_timer -= 0.35;
            self.prompt_visible = !self.prompt_visible;
        }
        self.blocked_timer = (self.blocked_timer - dt).max(0.0);
        self.radar_ping_brightness = (self.radar_ping_brightness - dt * 1.8).max(0.0);
        self.player_shot_cooldown = (self.player_shot_cooldown - dt).max(0.0);

        match self.mode {
            Mode::Title => self.update_title(dt, input),
            Mode::Playing => self.update_playing(dt, input),
            Mode::EnteringInitials => self.update_initials(dt, input),
            Mode::GameOver => self.update_game_over(dt),
        }
    }

    pub fn frame(&self) -> Scene {
        match self.mode {
            Mode::Title => self.title_scene(),
            Mode::Playing => self.play_scene(),
            Mode::EnteringInitials => self.initials_scene(),
            Mode::GameOver => self.game_over_scene(),
        }
    }

    fn update_title(&mut self, dt: f32, input: UpdateInput) {
        self.title_timer += dt;
        self.radar_sweep_angle = wrap_angle(self.radar_sweep_angle + dt * 1.2);
        if input.start_requested {
            self.start_game();
        }
    }

    fn update_game_over(&mut self, dt: f32) {
        self.game_over_timer -= dt;
        if self.game_over_timer <= 0.0 {
            self.enter_title_mode();
        }
    }

    fn update_initials(&mut self, dt: f32, input: UpdateInput) {
        let Some(initials) = self.initials.as_mut() else {
            self.enter_title_mode();
            return;
        };

        initials.blink_timer += dt;
        if initials.blink_timer >= 0.25 {
            initials.blink_timer -= 0.25;
            initials.blink_visible = !initials.blink_visible;
        }

        if input.initials_previous {
            rotate_letter(&mut initials.letters[initials.cursor], -1);
        }
        if input.initials_next {
            rotate_letter(&mut initials.letters[initials.cursor], 1);
        }
        if input.initials_confirm {
            initials.cursor += 1;
            if initials.cursor >= initials.letters.len() {
                self.commit_initials();
            } else if initials.letters[initials.cursor] == b'-' {
                initials.letters[initials.cursor] = b'A';
            }
        }
    }

    fn update_playing(&mut self, dt: f32, input: UpdateInput) {
        let input = self.effective_play_input(input);
        self.title_timer += dt;
        self.radar_sweep_angle = wrap_angle(self.radar_sweep_angle + dt * 2.1);

        self.update_player_state(dt);
        self.update_saucer(dt);
        self.update_enemy(dt);
        self.update_projectiles(dt);

        if self.player.state == PlayerState::Alive {
            self.update_player_movement(dt, &input);
            self.handle_player_fire(&input);
        }

        if self.enemy.is_none() && self.player.state != PlayerState::Dying {
            self.spawn_enemy(None);
        }
    }

    fn update_player_state(&mut self, dt: f32) {
        match self.player.state {
            PlayerState::Alive => {
                self.player.spawn_grace_timer = (self.player.spawn_grace_timer - dt).max(0.0);
            }
            PlayerState::Respawning => {
                self.player.timer -= dt;
                self.player.spawn_grace_timer = (self.player.spawn_grace_timer - dt).max(0.0);
                if self.player.timer <= 0.0 {
                    self.player.state = PlayerState::Alive;
                }
            }
            PlayerState::Dying => {
                self.player.timer -= dt;
                if self.player.timer <= 0.0 {
                    if self.lives == 0 {
                        if self.qualifies_for_high_score(self.score) {
                            self.begin_initials_entry();
                        } else {
                            self.mode = Mode::GameOver;
                            self.game_over_timer = 3.0;
                        }
                    } else {
                        self.respawn_player();
                    }
                }
            }
        }
    }

    fn update_player_movement(&mut self, dt: f32, input: &UpdateInput) {
        let left_speed = tread_speed(input.left_tread_axis());
        let right_speed = tread_speed(input.right_tread_axis());
        if left_speed.abs() <= f32::EPSILON && right_speed.abs() <= f32::EPSILON {
            return;
        }

        let angular_velocity = (right_speed - left_speed) * PLAYER_TREAD_TURN_RATE;
        self.player.heading = wrap_angle(self.player.heading + angular_velocity * dt);

        let speed = (left_speed + right_speed) * 0.5;
        let candidate = self.player.position + forward(self.player.heading) * (speed * dt);
        if self.position_is_walkable(candidate, PLAYER_RADIUS) {
            self.player.position = clamp_to_world(candidate);
        } else if speed.abs() > 0.5 {
            self.blocked_timer = MESSAGE_DURATION;
        }
    }

    fn handle_player_fire(&mut self, input: &UpdateInput) {
        if !input.fire
            || self.player.state != PlayerState::Alive
            || self.player_projectiles.len() >= self.player_projectile_capacity()
            || self.player_shot_cooldown > 0.0
        {
            return;
        }

        let velocity = forward(self.player.heading) * PLAYER_SHELL_SPEED;
        self.player_projectiles.push(Projectile {
            owner: ProjectileOwner::Player,
            position: self.player.position
                + Vec3::new(0.0, PLAYER_EYE_HEIGHT - 0.2, 0.0)
                + forward(self.player.heading) * 3.6,
            velocity,
            ttl: PLAYER_SHELL_LIFETIME,
        });
        self.player_shot_cooldown = self.player_fire_cooldown();
        self.events.push(GameEvent::PlayerShot);
    }

    fn update_enemy(&mut self, dt: f32) {
        let Some(mut enemy) = self.enemy else {
            return;
        };

        if !enemy.alive {
            enemy.state_timer -= dt;
            if enemy.state_timer <= 0.0 {
                self.enemy = None;
            } else {
                self.enemy = Some(enemy);
            }
            return;
        }

        enemy.state_timer += dt;
        enemy.sleep_timer = (enemy.sleep_timer - dt).max(0.0);
        enemy.decision_timer -= dt;
        enemy.shot_cooldown = (enemy.shot_cooldown - dt).max(0.0);
        match enemy.kind {
            EnemyKind::SlowTank | EnemyKind::SuperTank => {
                self.tank_pressure_timer = (self.tank_pressure_timer - dt).max(0.0);
            }
            EnemyKind::Missile => {
                self.missile_chain_timer = (self.missile_chain_timer - dt).max(0.0);
            }
        }

        match enemy.kind {
            EnemyKind::SlowTank | EnemyKind::SuperTank => {
                let enemy_sleeping = enemy.sleep_timer > 0.0;
                if enemy.kind == EnemyKind::SlowTank {
                    enemy.radar_heading = wrap_angle(enemy.radar_heading + OBJECT_SPIN_RATE * dt);
                }

                if !enemy_sleeping && enemy.decision_timer <= 0.0 {
                    let aggression = self.enemy_aggression(enemy);
                    let aim = angle_to(enemy.position, self.player.position);
                    let inaccuracy = if enemy.kind == EnemyKind::SlowTank {
                        let spread = lerp_f32(0.42, 0.14, aggression);
                        self.rng.f32() * spread - spread * 0.5
                    } else {
                        let spread = lerp_f32(0.22, 0.08, aggression);
                        self.rng.f32() * spread - spread * 0.5
                    };
                    enemy.desired_heading = wrap_angle(aim + inaccuracy);
                    enemy.decision_timer = if enemy.kind == EnemyKind::SlowTank {
                        lerp_f32(2.4, 1.1, aggression)
                            + self.rng.f32() * lerp_f32(1.2, 0.5, aggression)
                    } else {
                        lerp_f32(1.0, 0.35, aggression)
                            + self.rng.f32() * lerp_f32(0.45, 0.2, aggression)
                    };
                }

                let aggression = self.enemy_aggression(enemy);
                let turn_rate = match enemy.kind {
                    EnemyKind::SlowTank => lerp_f32(0.85, 1.4, aggression),
                    EnemyKind::SuperTank => lerp_f32(1.3, 2.4, aggression),
                    EnemyKind::Missile => 0.0,
                };
                enemy.heading = rotate_toward(enemy.heading, enemy.desired_heading, turn_rate * dt);

                let distance = distance_flat(enemy.position, self.player.position);
                let move_speed = match enemy.kind {
                    EnemyKind::SlowTank => lerp_f32(7.5, 10.5, aggression),
                    EnemyKind::SuperTank => lerp_f32(12.5, 16.5, aggression),
                    EnemyKind::Missile => 0.0,
                };
                let min_advance_distance = if enemy.kind == EnemyKind::SlowTank {
                    SLOW_TANK_MIN_ADVANCE_DISTANCE
                } else {
                    SUPER_TANK_MIN_ADVANCE_DISTANCE
                };
                if distance >= min_advance_distance {
                    let candidate = enemy.position + forward(enemy.heading) * (move_speed * dt);
                    if self.enemy_position_is_walkable(candidate, 3.4) {
                        enemy.position = clamp_to_world(candidate);
                        if enemy.kind == EnemyKind::SlowTank {
                            enemy.tread_phase = (enemy.tread_phase
                                + move_speed * dt * SLOW_TANK_TREAD_ADVANCE_RATE)
                                .rem_euclid(4.0);
                        }
                    } else {
                        enemy.desired_heading = wrap_angle(enemy.desired_heading + PI * 0.5);
                        enemy.decision_timer = 0.05;
                    }
                }

                let aim_error = angle_delta(
                    enemy.heading,
                    angle_to(enemy.position, self.player.position),
                )
                .abs();
                if self.enemy_projectile.is_none()
                    && enemy.shot_cooldown <= 0.0
                    && !enemy_sleeping
                    && self.player.spawn_grace_timer <= 0.0
                    && distance <= 84.0
                    && aim_error < lerp_f32(0.24, 0.08, aggression)
                {
                    self.enemy_projectile = Some(Projectile {
                        owner: ProjectileOwner::Enemy,
                        position: enemy.position
                            + Vec3::new(0.0, 1.6, 0.0)
                            + forward(enemy.heading) * 4.0,
                        velocity: forward(enemy.heading) * ENEMY_SHELL_SPEED,
                        ttl: ENEMY_SHELL_LIFETIME,
                    });
                    enemy.shot_cooldown = if enemy.kind == EnemyKind::SlowTank {
                        lerp_f32(2.7, 2.0, aggression)
                    } else {
                        lerp_f32(1.9, 1.15, aggression)
                    };
                    self.events.push(GameEvent::EnemyShot);
                }
            }
            EnemyKind::Missile => {
                let to_player = angle_to(enemy.position, self.player.position);
                let nastier = self.score >= arcade::missile_nastier_threshold();
                let sway = if nastier {
                    (self.title_timer * 9.0).sin() * 0.12
                } else {
                    (self.title_timer * 7.0).sin() * 0.38
                };
                enemy.desired_heading = wrap_angle(to_player + sway);
                enemy.heading = rotate_toward(enemy.heading, enemy.desired_heading, 3.8 * dt);

                let candidate = enemy.position + forward(enemy.heading) * (26.0 * dt);
                if !self.enemy_position_is_walkable(candidate, 2.4) && enemy.missile_height <= 0.15
                {
                    enemy.missile_vertical_velocity = 18.0;
                }
                enemy.position = clamp_to_world(candidate);
                enemy.missile_height =
                    (enemy.missile_height + enemy.missile_vertical_velocity * dt).max(0.0);
                enemy.missile_vertical_velocity -= 26.0 * dt;
                if enemy.missile_height <= 0.0 {
                    enemy.missile_height = 0.0;
                    enemy.missile_vertical_velocity = enemy.missile_vertical_velocity.max(0.0);
                }
            }
        }

        if distance_flat(enemy.position, self.player.position) <= PLAYER_RADIUS + 2.6
            && self.player.state == PlayerState::Alive
        {
            self.kill_player();
        }

        let distance_from_player = distance_flat(enemy.position, self.player.position);
        if matches!(enemy.kind, EnemyKind::SlowTank | EnemyKind::SuperTank)
            && self.tank_pressure_timer <= 0.0
        {
            self.enemy = Some(enemy);
            self.enemy_projectile = None;
            self.spawn_enemy(Some(EnemyKind::Missile));
            return;
        }
        if enemy.kind == EnemyKind::Missile && distance_from_player >= ENEMY_OUT_OF_RANGE_DISTANCE {
            let replacement = if self.missile_chain_timer > 0.0 {
                EnemyKind::Missile
            } else {
                tank_kind(self.missile_launch_counter)
            };
            self.enemy = Some(enemy);
            self.enemy_projectile = None;
            self.spawn_enemy(Some(replacement));
            return;
        }

        self.update_radar_ping(enemy.position);
        self.enemy = Some(enemy);
    }

    fn update_projectiles(&mut self, dt: f32) {
        let player_projectiles = std::mem::take(&mut self.player_projectiles);
        for projectile in player_projectiles {
            if let Some(projectile) = self.advance_projectile(projectile, dt) {
                self.player_projectiles.push(projectile);
            }
        }
        self.enemy_projectile = self
            .enemy_projectile
            .take()
            .and_then(|projectile| self.advance_projectile(projectile, dt));
    }

    fn advance_projectile(&mut self, mut projectile: Projectile, dt: f32) -> Option<Projectile> {
        projectile.ttl -= dt;
        if projectile.ttl <= 0.0 {
            return None;
        }
        projectile.position += projectile.velocity * dt;
        projectile.position = clamp_to_world(projectile.position);

        if self.projectile_hits_obstacle(projectile.position) {
            return None;
        }

        match projectile.owner {
            ProjectileOwner::Player => {
                if let Some(enemy) = self.enemy
                    && enemy.alive
                    && distance(projectile.position, enemy_position(enemy))
                        <= enemy_radius(enemy.kind)
                {
                    self.destroy_enemy();
                    return None;
                }

                if self.saucer.state == SaucerState::Alive
                    && distance(projectile.position, self.saucer.position) <= SAUCER_HIT_RADIUS
                {
                    self.destroy_saucer(true);
                    return None;
                }
            }
            ProjectileOwner::Enemy => {
                if self.saucer.state == SaucerState::Alive
                    && distance(projectile.position, self.saucer.position) <= SAUCER_HIT_RADIUS
                {
                    self.destroy_saucer(false);
                    return None;
                }

                if self.player.state == PlayerState::Alive
                    && !self.easter_egg.invincible
                    && distance(
                        projectile.position,
                        self.player.position + Vec3::new(0.0, 1.6, 0.0),
                    ) <= PLAYER_RADIUS
                {
                    self.kill_player();
                    return None;
                }
            }
        }

        Some(projectile)
    }

    fn update_saucer(&mut self, dt: f32) {
        match self.saucer.state {
            SaucerState::Inactive => {
                if self.score < self.arcade.saucer_score_threshold {
                    return;
                }
                self.saucer.timer -= dt;
                if self.saucer.timer <= 0.0 {
                    self.spawn_saucer();
                }
            }
            SaucerState::Alive => {
                self.saucer.heading = wrap_angle(self.saucer.heading + OBJECT_SPIN_RATE * dt);
                self.saucer.position += self.saucer.velocity * dt;
                self.saucer.timer -= dt;
                if self.saucer.timer <= 0.0 {
                    self.randomize_saucer_motion();
                }
            }
            SaucerState::Dying => {
                self.saucer.heading = wrap_angle(self.saucer.heading + OBJECT_SPIN_RATE * dt);
                self.saucer.timer -= dt;
                if self.saucer.timer <= 0.0 {
                    self.saucer.state = SaucerState::Inactive;
                    self.saucer.timer = self.random_saucer_respawn_delay();
                }
            }
        }
    }

    fn destroy_enemy(&mut self) {
        let Some(mut enemy) = self.enemy else {
            return;
        };
        enemy.alive = false;
        enemy.state_timer = 1.4;
        self.score += match enemy.kind {
            EnemyKind::SlowTank => 1_000,
            EnemyKind::Missile => 2_000,
            EnemyKind::SuperTank => 3_000,
        };
        self.check_bonus_tank_award();
        self.events.push(GameEvent::EnemyDestroyed);
        self.enemy = Some(enemy);
        self.enemy_projectile = None;
    }

    fn destroy_saucer(&mut self, award_points: bool) {
        if self.saucer.state != SaucerState::Alive {
            return;
        }
        if award_points {
            self.score += 5_000;
            self.check_bonus_tank_award();
        }
        self.saucer.state = SaucerState::Dying;
        self.saucer.timer = 1.2;
        self.events.push(GameEvent::SaucerDestroyed);
    }

    fn kill_player(&mut self) {
        if self.player.state != PlayerState::Alive {
            return;
        }

        self.player.state = PlayerState::Dying;
        self.player.timer = PLAYER_DYING_DELAY;
        self.player.spawn_grace_timer = 0.0;
        self.player_projectiles.clear();
        self.player_shot_cooldown = 0.0;
        self.enemy_projectile = None;
        self.lives = self.lives.saturating_sub(1);
        self.enemy_score = self.enemy_score.saturating_add(1_000);
        self.events.push(GameEvent::PlayerDestroyed);
    }

    fn check_bonus_tank_award(&mut self) {
        if self.next_bonus_tank_index >= self.arcade.bonus_tank_thresholds.len() {
            return;
        }

        while self.next_bonus_tank_index < self.arcade.bonus_tank_thresholds.len()
            && self.score >= self.arcade.bonus_tank_thresholds[self.next_bonus_tank_index]
        {
            self.lives += 1;
            self.next_bonus_tank_index += 1;
        }
    }

    fn respawn_player(&mut self) {
        self.place_player_randomly();
        self.player.state = PlayerState::Respawning;
        self.player.timer = PLAYER_RESPAWN_DELAY;
        self.player.spawn_grace_timer = PLAYER_RESPAWN_DELAY;
        self.player_projectiles.clear();
        self.player_shot_cooldown = 0.0;
        self.enemy_projectile = None;

        match self.enemy {
            Some(enemy) if enemy.alive && enemy.kind == EnemyKind::Missile => {
                self.spawn_enemy(Some(tank_kind(self.missile_launch_counter)));
            }
            Some(mut enemy) if enemy.alive => {
                let random_heading = self.rng.f32() * TAU;
                enemy.heading = random_heading;
                enemy.desired_heading = random_heading;
                enemy.sleep_timer = RESPAWN_ENEMY_SLEEP_SECONDS;
                enemy.decision_timer = RESPAWN_ENEMY_SLEEP_SECONDS;
                enemy.shot_cooldown = enemy.shot_cooldown.max(RESPAWN_ENEMY_SLEEP_SECONDS);
                self.enemy = Some(enemy);
            }
            _ => self.spawn_enemy(None),
        }
    }

    fn start_game(&mut self) {
        self.mode = Mode::Playing;
        self.autopilot = false;
        self.score = 0;
        self.enemy_score = 0;
        self.lives = self.arcade.starting_lives;
        self.next_bonus_tank_index = 0;
        self.missile_launch_counter = u8::MAX;
        self.enemy = None;
        self.player_projectiles.clear();
        self.player_shot_cooldown = 0.0;
        self.enemy_projectile = None;
        self.tank_pressure_timer = 0.0;
        self.missile_chain_timer = 0.0;
        self.place_player_randomly();
        self.player.state = PlayerState::Respawning;
        self.player.timer = PLAYER_RESPAWN_DELAY;
        self.player.spawn_grace_timer = PLAYER_RESPAWN_DELAY;
        self.saucer.state = SaucerState::Inactive;
        self.saucer.timer = self.random_saucer_respawn_delay();
        self.spawn_enemy(Some(EnemyKind::SlowTank));
        self.events.push(GameEvent::GameStarted);
    }

    fn enter_title_mode(&mut self) {
        self.mode = Mode::Title;
        self.initials = None;
        self.autopilot = false;
        self.easter_egg = EasterEggState::default();
        self.reset_title_world();
        self.events.push(GameEvent::TitleScreenEntered);
    }

    fn reset_title_world(&mut self) {
        self.title_timer = 0.0;
        self.prompt_timer = 0.0;
        self.prompt_visible = true;
        self.enemy = Some(Enemy {
            kind: EnemyKind::SlowTank,
            position: Vec3::new(18.0, 0.0, 72.0),
            heading: PI,
            desired_heading: PI,
            radar_heading: 0.0,
            tread_phase: 0.0,
            sleep_timer: 0.0,
            state_timer: 0.0,
            decision_timer: 1.0,
            shot_cooldown: 0.0,
            missile_height: 0.0,
            missile_vertical_velocity: 0.0,
            alive: true,
        });
        self.saucer = Saucer {
            position: Vec3::new(-36.0, SAUCER_HEIGHT, 86.0),
            velocity: Vec3::new(8.0, 0.0, -3.0),
            heading: 0.0,
            timer: SAUCER_DIRECTION_CHANGE_MAX,
            state: SaucerState::Alive,
        };
    }

    fn begin_initials_entry(&mut self) {
        self.mode = Mode::EnteringInitials;
        self.initials = Some(InitialsEntry {
            letters: [b'A', b'-', b'-'],
            cursor: 0,
            blink_timer: 0.0,
            blink_visible: true,
            score: self.score,
        });
    }

    fn commit_initials(&mut self) {
        let Some(entry) = self.initials.take() else {
            return;
        };
        let initials = String::from_utf8(entry.letters.to_vec()).expect("letters should be ASCII");
        self.high_scores.push(HighScoreEntry {
            initials,
            score: entry.score,
        });
        self.high_scores.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.initials.cmp(&right.initials))
        });
        self.high_scores.truncate(10);
        self.enter_title_mode();
    }

    fn qualifies_for_high_score(&self, score: u32) -> bool {
        self.high_scores.len() < 10
            || self
                .high_scores
                .last()
                .is_some_and(|entry| score > entry.score)
    }

    fn spawn_enemy(&mut self, preferred: Option<EnemyKind>) {
        let previous_kind = self.enemy.map(|enemy| enemy.kind);
        let kind = preferred.unwrap_or_else(|| self.choose_enemy_kind());
        if kind == EnemyKind::Missile {
            self.missile_launch_counter = self.missile_launch_counter.wrapping_add(1);
        }
        let aggression = self.enemy_spawn_aggression();

        for _ in 0..64 {
            let (spawn_heading, spawn_distance) = if kind == EnemyKind::Missile {
                (
                    wrap_angle(self.player.heading + self.rng.f32() * 0.45 - 0.225),
                    self.arcade.far_spawn_distance,
                )
            } else {
                let spread = lerp_f32(0.3, PI, aggression);
                (
                    wrap_angle(self.player.heading + self.rng.f32() * spread * 2.0 - spread),
                    if self.rng.bool() {
                        self.arcade.near_spawn_distance
                    } else {
                        self.arcade.far_spawn_distance
                    },
                )
            };
            let position = self.player.position + forward(spawn_heading) * spawn_distance;
            if !self.enemy_position_is_walkable(position, 3.5) {
                continue;
            }
            let heading = if self.player.state == PlayerState::Alive {
                angle_to(position, self.player.position)
            } else {
                self.rng.f32() * TAU
            };
            self.enemy = Some(Enemy {
                kind,
                position: clamp_to_world(position),
                heading,
                desired_heading: heading,
                radar_heading: 0.0,
                tread_phase: 0.0,
                sleep_timer: if self.player.state == PlayerState::Alive {
                    0.0
                } else {
                    RESPAWN_ENEMY_SLEEP_SECONDS
                },
                state_timer: 0.0,
                decision_timer: if self.player.state == PlayerState::Alive {
                    0.45
                } else {
                    3.0
                },
                shot_cooldown: ENEMY_SPAWN_FIRE_DELAY,
                missile_height: 0.0,
                missile_vertical_velocity: 0.0,
                alive: true,
            });
            self.reset_enemy_cycle_timers(kind, previous_kind);
            return;
        }

        self.enemy = Some(Enemy {
            kind,
            position: Vec3::new(32.0, 0.0, 64.0),
            heading: PI,
            desired_heading: PI,
            radar_heading: 0.0,
            tread_phase: 0.0,
            sleep_timer: if self.player.state == PlayerState::Alive {
                0.0
            } else {
                RESPAWN_ENEMY_SLEEP_SECONDS
            },
            state_timer: 0.0,
            decision_timer: 0.6,
            shot_cooldown: ENEMY_SPAWN_FIRE_DELAY,
            missile_height: 0.0,
            missile_vertical_velocity: 0.0,
            alive: true,
        });
        self.reset_enemy_cycle_timers(kind, previous_kind);
    }

    fn choose_enemy_kind(&mut self) -> EnemyKind {
        if self.score < self.arcade.missile_score_threshold {
            return tank_kind(self.missile_launch_counter);
        }
        if self.rng.bool() {
            EnemyKind::Missile
        } else {
            tank_kind(self.missile_launch_counter)
        }
    }

    fn spawn_saucer(&mut self) {
        self.saucer.state = SaucerState::Alive;
        self.saucer.position = Vec3::new(
            self.rng.f32() * 200.0 - 100.0,
            SAUCER_HEIGHT,
            self.rng.f32() * 200.0 - 100.0,
        );
        self.saucer.heading = 0.0;
        self.randomize_saucer_motion();
    }

    fn randomize_saucer_motion(&mut self) {
        self.saucer.velocity = Vec3::new(
            self.random_saucer_velocity_component(),
            0.0,
            self.random_saucer_velocity_component(),
        );
        self.saucer.timer = self.random_saucer_direction_change_delay();
    }

    fn random_saucer_direction_change_delay(&mut self) -> f32 {
        self.rng.f32() * SAUCER_DIRECTION_CHANGE_MAX
    }

    fn random_saucer_respawn_delay(&mut self) -> f32 {
        self.rng.f32() * SAUCER_RESPAWN_DELAY_MAX
    }

    fn random_saucer_velocity_component(&mut self) -> f32 {
        self.rng.f32() * 24.0 - 12.0
    }

    fn reset_enemy_cycle_timers(&mut self, kind: EnemyKind, previous_kind: Option<EnemyKind>) {
        match kind {
            EnemyKind::SlowTank | EnemyKind::SuperTank => {
                self.tank_pressure_timer = self.random_tank_pressure_timeout();
                self.missile_chain_timer = 0.0;
            }
            EnemyKind::Missile => {
                self.tank_pressure_timer = 0.0;
                if previous_kind != Some(EnemyKind::Missile) || self.missile_chain_timer <= 0.0 {
                    self.missile_chain_timer = self.random_missile_chain_timeout();
                }
            }
        }
    }

    fn random_tank_pressure_timeout(&mut self) -> f32 {
        TANK_PRESSURE_TIMEOUT_MIN
            + self.rng.f32() * (TANK_PRESSURE_TIMEOUT_MAX - TANK_PRESSURE_TIMEOUT_MIN)
    }

    fn random_missile_chain_timeout(&mut self) -> f32 {
        MISSILE_CHAIN_TIMEOUT_MIN
            + self.rng.f32() * (MISSILE_CHAIN_TIMEOUT_MAX - MISSILE_CHAIN_TIMEOUT_MIN)
    }

    fn place_player_randomly(&mut self) {
        self.player.position = self.random_player_spawn_position();
        self.player.heading = self.rng.f32() * TAU;
    }

    fn random_player_spawn_position(&mut self) -> Vec3 {
        for _ in 0..128 {
            let position = Vec3::new(
                self.rng.f32() * WORLD_LIMIT * 2.0 - WORLD_LIMIT,
                0.0,
                self.rng.f32() * WORLD_LIMIT * 2.0 - WORLD_LIMIT,
            );
            let clear_of_enemy = self.enemy.is_none_or(|enemy| {
                !enemy.alive
                    || distance_flat(position, enemy_position(enemy))
                        > PLAYER_RADIUS + enemy_radius(enemy.kind) + 6.0
            });
            if clear_of_enemy && self.position_is_walkable(position, PLAYER_RADIUS) {
                return clamp_to_world(position);
            }
        }

        Vec3::new(0.0, 0.0, 0.0)
    }

    fn enemy_spawn_aggression(&self) -> f32 {
        ((self.score as f32 - self.enemy_score as f32) / ENEMY_SCORE_FULL_AGGRESSION_DELTA)
            .clamp(0.0, 1.0)
    }

    fn enemy_aggression(&self, enemy: Enemy) -> f32 {
        let base = self.enemy_spawn_aggression();
        if enemy.state_timer >= ENEMY_MAX_PATIENCE_SECONDS {
            base.max(0.85)
        } else {
            base
        }
    }

    fn handle_easter_egg_input(&mut self, typed_chars: &[char]) {
        for &character in typed_chars {
            self.update_easter_egg_sequence(character);
            if !self.easter_egg.active {
                continue;
            }

            match character {
                'g' => {
                    self.easter_egg.invincible = !self.easter_egg.invincible;
                }
                'f' => {
                    self.easter_egg.fire_level =
                        (self.easter_egg.fire_level + 1).min(SECRET_FIRE_LEVEL_MAX);
                    self.player_shot_cooldown = self
                        .player_shot_cooldown
                        .min(SECRET_FIRE_COOLDOWNS[self.easter_egg.fire_level as usize]);
                }
                _ => {}
            }
        }
    }

    fn update_easter_egg_sequence(&mut self, character: char) -> bool {
        if character == EASTER_EGG_CODE[self.easter_egg.sequence_index] {
            self.easter_egg.sequence_index += 1;
            if self.easter_egg.sequence_index == EASTER_EGG_CODE.len() {
                self.easter_egg.toggle();
                self.autopilot = false;
                self.player_shot_cooldown = 0.0;
            }
            return true;
        }

        self.easter_egg.sequence_index = usize::from(character == EASTER_EGG_CODE[0]);
        character == EASTER_EGG_CODE[0]
    }

    fn effective_play_input(&self, input: UpdateInput) -> UpdateInput {
        if self.autopilot && self.player.state == PlayerState::Alive {
            return self.autopilot_input();
        }

        input
    }

    fn player_projectile_capacity(&self) -> usize {
        1 + usize::from(self.easter_egg.fire_level)
    }

    fn player_fire_cooldown(&self) -> f32 {
        SECRET_FIRE_COOLDOWNS[self.easter_egg.fire_level as usize]
    }

    fn xyzzy_indicator_rows(&self) -> Option<Vec<(String, [u8; 4])>> {
        if !self.easter_egg.active {
            return None;
        }

        Some(vec![
            (String::from("XYZZY MODE"), WARNING_COLOR),
            (
                format!("FIRE RATE {}", self.easter_egg.fire_level + 1),
                SCREEN_COLOR,
            ),
            (
                format!(
                    "GOD {}",
                    if self.easter_egg.invincible {
                        "ON"
                    } else {
                        "OFF"
                    }
                ),
                if self.easter_egg.invincible {
                    WARNING_COLOR
                } else {
                    SCREEN_COLOR_DIM
                },
            ),
            (
                format!("AUTO {}", if self.autopilot { "ON" } else { "OFF" }),
                if self.autopilot {
                    WARNING_COLOR
                } else {
                    SCREEN_COLOR_DIM
                },
            ),
        ])
    }

    fn autopilot_input(&self) -> UpdateInput {
        let Some(enemy) = self.enemy.filter(|enemy| enemy.alive) else {
            return UpdateInput::default();
        };

        let enemy_position = enemy_position(enemy);
        let enemy_bearing = angle_to(self.player.position, enemy_position);
        let candidate_offsets = [
            0.0,
            0.22,
            -0.22,
            0.5,
            -0.5,
            PI * 0.5,
            -PI * 0.5,
            PI * 0.82,
            -PI * 0.82,
            PI,
        ];
        let throttles = [1, 0, -1];

        let mut best_heading = enemy_bearing;
        let mut best_throttle = 0;
        let mut best_score = f32::MIN;
        for offset in candidate_offsets {
            let heading = wrap_angle(enemy_bearing + offset);
            for throttle in throttles {
                if let Some(score) =
                    self.autopilot_candidate_score(heading, throttle, enemy, enemy_position)
                    && score > best_score
                {
                    best_score = score;
                    best_heading = heading;
                    best_throttle = throttle;
                }
            }
        }

        let mut input = tread_input_for_heading(self.player.heading, best_heading, best_throttle);
        if self.autopilot_should_fire(enemy_position) {
            input.fire = true;
        }
        input
    }

    fn autopilot_candidate_score(
        &self,
        heading: f32,
        throttle: i8,
        enemy: Enemy,
        enemy_position: Vec3,
    ) -> Option<f32> {
        let mut predicted_position = self.player.position;
        if throttle != 0 {
            let speed = tread_speed(throttle) * AUTOPILOT_PREDICTION_SECONDS;
            predicted_position += forward(heading) * speed;
            if !self.position_is_walkable(predicted_position, PLAYER_RADIUS) {
                return None;
            }
        }

        let distance_to_enemy = distance_flat(predicted_position, enemy_position);
        let aim_error = angle_delta(heading, angle_to(predicted_position, enemy_position)).abs();
        let clear_shot = !self.segment_hits_obstacle(predicted_position, enemy_position);
        let projectile_threat = self.autopilot_projectile_threat(predicted_position);
        let fire_threat =
            self.autopilot_enemy_fire_threat(predicted_position, enemy, enemy_position);
        let shielded = self.segment_hits_obstacle(enemy_position, predicted_position);

        let mut score = 12.0 - (distance_to_enemy - 38.0).abs() * 0.1;
        score -= projectile_threat * 28.0;
        score -= fire_threat * 18.0;
        score -= angle_delta(self.player.heading, heading).abs() * 1.5;
        if shielded {
            score += 6.0;
        }
        if clear_shot && aim_error < AUTOPILOT_SHOT_CONE {
            score += 14.0;
        }
        if throttle == 0 && clear_shot && aim_error < 0.18 {
            score += 3.0;
        }
        if enemy.kind == EnemyKind::Missile && distance_to_enemy < 24.0 {
            score -= 12.0;
        }
        Some(score)
    }

    fn autopilot_should_fire(&self, enemy_position: Vec3) -> bool {
        self.player_shot_cooldown <= 0.0
            && self.player_projectiles.len() < self.player_projectile_capacity()
            && distance_flat(self.player.position, enemy_position) <= 90.0
            && !self.segment_hits_obstacle(self.player.position, enemy_position)
            && angle_delta(
                self.player.heading,
                angle_to(self.player.position, enemy_position),
            )
            .abs()
                < AUTOPILOT_SHOT_CONE
            && (self.easter_egg.invincible
                || self.autopilot_projectile_threat(self.player.position) < 0.65)
    }

    fn autopilot_projectile_threat(&self, position: Vec3) -> f32 {
        let Some(projectile) = self.enemy_projectile else {
            return 0.0;
        };

        let velocity = Vec3::new(projectile.velocity.x, 0.0, projectile.velocity.z);
        let speed_sq = velocity.x * velocity.x + velocity.z * velocity.z;
        if speed_sq <= f32::EPSILON {
            return 0.0;
        }

        let to_position = position - projectile.position;
        let t = ((to_position.x * velocity.x + to_position.z * velocity.z) / speed_sq)
            .clamp(0.0, projectile.ttl.min(1.2));
        let closest = projectile.position + projectile.velocity * t;
        let distance = distance_flat(position, closest);
        if distance >= 14.0 {
            0.0
        } else {
            1.0 - distance / 14.0
        }
    }

    fn autopilot_enemy_fire_threat(
        &self,
        position: Vec3,
        enemy: Enemy,
        enemy_position: Vec3,
    ) -> f32 {
        if self.segment_hits_obstacle(enemy_position, position) {
            return 0.0;
        }

        let distance = distance_flat(enemy_position, position);
        match enemy.kind {
            EnemyKind::Missile => {
                if distance >= 28.0 {
                    0.0
                } else {
                    1.0 - distance / 28.0
                }
            }
            EnemyKind::SlowTank | EnemyKind::SuperTank => {
                if distance > 84.0 {
                    return 0.0;
                }
                let aim_error =
                    angle_delta(enemy.heading, angle_to(enemy_position, position)).abs();
                if aim_error > 0.26 {
                    return 0.0;
                }
                let cooldown_factor = if enemy.shot_cooldown <= 0.25 {
                    1.0
                } else {
                    0.45
                };
                (1.0 - distance / 84.0) * (1.0 - aim_error / 0.26) * cooldown_factor
            }
        }
    }

    fn position_is_walkable(&self, position: Vec3, radius: f32) -> bool {
        if position.x.abs() > WORLD_LIMIT || position.z.abs() > WORLD_LIMIT {
            return false;
        }
        !self.arcade.obstacles.iter().any(|obstacle| {
            distance_flat(position, Vec3::new(obstacle.x, 0.0, obstacle.z))
                <= obstacle.radius + radius
        })
    }

    fn enemy_position_is_walkable(&self, position: Vec3, radius: f32) -> bool {
        self.position_is_walkable(position, radius)
            && distance_flat(position, self.player.position) > PLAYER_RADIUS + radius
    }

    fn projectile_hits_obstacle(&self, position: Vec3) -> bool {
        self.arcade.obstacles.iter().any(|obstacle| {
            distance_flat(position, Vec3::new(obstacle.x, 0.0, obstacle.z)) <= obstacle.radius
        })
    }

    fn segment_hits_obstacle(&self, start: Vec3, end: Vec3) -> bool {
        self.arcade.obstacles.iter().any(|obstacle| {
            distance_point_to_segment_flat(start, end, Vec3::new(obstacle.x, 0.0, obstacle.z))
                <= obstacle.radius
        })
    }

    fn update_radar_ping(&mut self, enemy_position: Vec3) {
        let relative = enemy_position - self.player.position;
        let bearing = wrap_angle(relative.z.atan2(relative.x));
        let sweep_bearing = wrap_angle(self.radar_sweep_angle);
        if angle_delta(bearing, sweep_bearing).abs() < 0.08 {
            if self.radar_ping_brightness <= 0.2 {
                self.events.push(GameEvent::RadarPing);
            }
            self.radar_ping_brightness = 1.0;
        }
    }

    fn attract_scene(&self) -> AttractScene {
        if self.title_timer.rem_euclid(ATTRACT_CYCLE_DURATION) < TITLE_SCREEN_DURATION {
            AttractScene::Title
        } else {
            AttractScene::HighScores
        }
    }

    fn title_scene(&self) -> Scene {
        match self.attract_scene() {
            AttractScene::Title => self.title_logo_scene(),
            AttractScene::HighScores => self.high_score_scene(),
        }
    }

    fn title_logo_scene(&self) -> Scene {
        let camera = Camera {
            position: Vec3::new(
                self.title_timer.sin() * 2.5,
                PLAYER_EYE_HEIGHT + 0.15,
                -24.0 + self.title_timer.cos() * 1.25,
            ),
            heading: self.title_timer.sin() * 0.04,
        };
        let mut scene = Scene::empty(camera);
        scene.background = BackgroundStyle::Solid(ARCADE_BLACK);
        scene.show_crosshair = false;
        self.add_landscape(&mut scene);
        self.add_obstacles(&mut scene);
        if let Some(enemy) = self.enemy {
            self.add_enemy(&mut scene, enemy, 0.9);
        }
        if self.saucer.state == SaucerState::Alive {
            self.add_saucer(&mut scene);
        }
        self.add_title_logo(&mut scene);

        self.add_arcade_score_header(&mut scene, 0, self.arcade.starting_lives);

        if self.prompt_visible {
            self.push_arcade_label(&mut scene, attract::PRESS_START_LABEL, WARNING_COLOR);
        }

        self.push_arcade_label(&mut scene, attract::INSERT_COIN_LABEL, SCREEN_COLOR_DIM);
        self.push_arcade_label(&mut scene, attract::COPYRIGHT_LABEL, SCREEN_COLOR_DIM);
        self.push_arcade_text(
            &mut scene,
            attract::BONUS_TANK_LABEL.position,
            arcade::bonus_tank_label(),
            attract::BONUS_TANK_LABEL.size,
            SCREEN_COLOR,
        );
        scene
    }

    fn high_score_scene(&self) -> Scene {
        let mut scene = Scene::empty(Camera {
            position: Vec3::new(0.0, 0.0, 0.0),
            heading: 0.0,
        });
        scene.background = BackgroundStyle::Solid(ARCADE_BLACK);
        self.add_arcade_score_header(&mut scene, 0, self.arcade.starting_lives);
        self.push_arcade_label(&mut scene, attract::HIGH_SCORES_LABEL, SCREEN_COLOR);
        self.add_high_score_rows(&mut scene);
        self.push_arcade_text(
            &mut scene,
            self.high_score_bonus_label_position(),
            arcade::bonus_tank_label(),
            attract::BONUS_TANK_LABEL.size,
            SCREEN_COLOR,
        );
        scene
    }

    fn play_scene(&self) -> Scene {
        let camera = Camera {
            position: self.player.position + Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0),
            heading: self.player.heading,
        };
        let mut scene = Scene::empty(camera);
        scene.show_crosshair = self.player.state != PlayerState::Dying;

        self.add_landscape(&mut scene);
        self.add_obstacles(&mut scene);
        if let Some(enemy) = self.enemy {
            self.add_enemy(&mut scene, enemy, 1.0);
        }
        if self.saucer.state != SaucerState::Inactive {
            self.add_saucer(&mut scene);
        }
        for projectile in &self.player_projectiles {
            self.add_projectile(&mut scene, *projectile, 1.1);
        }
        if let Some(projectile) = self.enemy_projectile {
            self.add_projectile(&mut scene, projectile, 0.9);
        }
        if self.player.state == PlayerState::Dying {
            self.add_player_explosion(&mut scene);
        }

        self.add_play_overlay(&mut scene);
        scene
    }

    fn initials_scene(&self) -> Scene {
        let mut scene = Scene::empty(Camera {
            position: Vec3::new(0.0, 0.0, 0.0),
            heading: 0.0,
        });
        scene.background = BackgroundStyle::Solid(ARCADE_BLACK);
        let entry_score = self
            .initials
            .as_ref()
            .map_or(self.score, |initials| initials.score);
        self.add_arcade_score_header(&mut scene, entry_score, 0);
        self.push_arcade_label(&mut scene, attract::GREAT_SCORE_LABEL, WARNING_COLOR);
        self.push_arcade_label(&mut scene, attract::ENTER_INITIALS_LABEL, SCREEN_COLOR);
        self.push_arcade_label(&mut scene, attract::CHANGE_LETTER_LABEL, SCREEN_COLOR_DIM);
        self.push_arcade_label(&mut scene, attract::SELECT_LETTER_LABEL, SCREEN_COLOR_DIM);

        if let Some(initials) = &self.initials {
            self.push_arcade_text(
                &mut scene,
                (-72, -72),
                self.initials_display_text(initials),
                TextSize::Full,
                SCREEN_COLOR,
            );
        }
        scene
    }

    fn game_over_scene(&self) -> Scene {
        let mut scene = self.play_scene();
        let center_x = self.viewport_width as i32 / 2;
        let center_y = self.viewport_height as i32 / 2;
        scene.overlay_text.push(ScreenText {
            position: (center_x, center_y),
            text: "GAME OVER".to_string(),
            color: WARNING_COLOR,
            scale: 4,
            centered: true,
        });
        scene
    }

    fn add_arcade_score_header(&self, scene: &mut Scene, score: u32, lives: u32) {
        self.push_arcade_text(
            scene,
            attract::SCORE_LABEL.position,
            format!("SCORE {}", format_arcade_score(score)),
            attract::SCORE_LABEL.size,
            SCREEN_COLOR,
        );

        let top_score = self
            .high_scores
            .first()
            .map_or(score, |entry| entry.score.max(score));
        self.push_arcade_text(
            scene,
            attract::HIGH_SCORE_LABEL.position,
            format!("HIGH SCORE {}", format_arcade_score(top_score)),
            attract::HIGH_SCORE_LABEL.size,
            SCREEN_COLOR,
        );

        let lives_origin = self.arcade_to_screen((128, 360));
        let icon_spacing = 22 * i32::from(self.arcade_text_scale(TextSize::Full));
        for index in 0..lives as i32 {
            push_tank_icon(
                scene,
                (lives_origin.0 + index * icon_spacing, lives_origin.1),
                i32::from(self.arcade_text_scale(TextSize::Half)),
                INFO_COLOR,
            );
        }
    }

    fn add_high_score_rows(&self, scene: &mut Scene) {
        for (index, entry) in self.high_scores.iter().enumerate() {
            let position = (
                attract::HIGH_SCORE_LIST_START.0 + index as i16 * attract::HIGH_SCORE_ROW_DELTA.0,
                attract::HIGH_SCORE_LIST_START.1 + index as i16 * attract::HIGH_SCORE_ROW_DELTA.1,
            );
            let row_text = format!("{} {}", format_arcade_score(entry.score), entry.initials);
            self.push_arcade_text(
                scene,
                position,
                row_text.clone(),
                TextSize::Full,
                INFO_COLOR,
            );
            if entry.score >= attract::TANK_ICON_SCORE_THRESHOLD {
                let pixel_position = self.arcade_to_screen(position);
                let row_scale = i32::from(self.arcade_text_scale(TextSize::Full));
                let scale = i32::from(self.arcade_text_scale(TextSize::Half));
                let icon_x = pixel_position.0
                    + ((296.0 / attract::ARCADE_SCREEN_WIDTH as f32) * self.viewport_width as f32)
                        .round() as i32;
                push_tank_icon(
                    scene,
                    (icon_x, pixel_position.1 + 6 * row_scale),
                    scale,
                    INFO_COLOR,
                );
            }
        }
    }

    fn high_score_bonus_label_position(&self) -> (i16, i16) {
        let last_row_y =
            self.high_scores
                .last()
                .map_or(attract::BONUS_TANK_LABEL.position.1, |_| {
                    attract::HIGH_SCORE_LIST_START.1
                        + (self.high_scores.len() as i16 - 1) * attract::HIGH_SCORE_ROW_DELTA.1
                });
        (
            attract::BONUS_TANK_LABEL.position.0,
            (last_row_y - 64).min(attract::BONUS_TANK_LABEL.position.1),
        )
    }

    fn add_title_logo(&self, scene: &mut Scene) {
        let progress = (self.title_timer.rem_euclid(ATTRACT_CYCLE_DURATION)
            / TITLE_SCREEN_DURATION)
            .clamp(0.0, 1.0);
        let logo_position = Vec3::new(
            0.0,
            TITLE_LOGO_START_Y + (TITLE_LOGO_END_Y - TITLE_LOGO_START_Y) * progress,
            TITLE_LOGO_START_Z + (TITLE_LOGO_END_Z - TITLE_LOGO_START_Z) * progress,
        );
        let logo_scale = Vec3::new(TITLE_LOGO_SCALE, TITLE_LOGO_SCALE, TITLE_LOGO_SCALE);
        for mesh in attract::TITLE_LOGO_MESHES {
            push_shape(
                scene,
                mesh.vertices,
                mesh.edges,
                logo_position,
                0.0,
                logo_scale,
                1.25,
            );
        }
    }

    fn push_arcade_label(&self, scene: &mut Scene, label: attract::ScreenLabel, color: [u8; 4]) {
        self.push_arcade_text(scene, label.position, label.text, label.size, color);
    }

    fn push_arcade_text(
        &self,
        scene: &mut Scene,
        position: (i16, i16),
        text: impl Into<String>,
        size: TextSize,
        color: [u8; 4],
    ) {
        scene.overlay_text.push(ScreenText {
            position: self.arcade_to_screen(position),
            text: text.into(),
            color,
            scale: self.arcade_text_scale(size),
            centered: false,
        });
    }

    fn arcade_to_screen(&self, position: (i16, i16)) -> (i32, i32) {
        let x =
            self.viewport_width as f32 * (position.0 as f32 / attract::ARCADE_SCREEN_WIDTH as f32);
        let y = self.viewport_height as f32
            * (position.1 as f32 / attract::ARCADE_SCREEN_HEIGHT as f32);
        (
            (self.viewport_width as f32 * 0.5 + x).round() as i32,
            (self.viewport_height as f32 * 0.5 - y).round() as i32,
        )
    }

    fn arcade_text_scale(&self, size: TextSize) -> u8 {
        let full = if self.viewport_height >= 540 {
            3
        } else if self.viewport_height >= 360 {
            2
        } else {
            1
        };
        match size {
            TextSize::Full => full,
            TextSize::Half => full.saturating_sub(1).max(1),
        }
    }

    fn initials_display_text(&self, initials: &InitialsEntry) -> String {
        let mut text = String::new();
        for (index, letter) in initials.letters.iter().enumerate() {
            if index == initials.cursor && !initials.blink_visible {
                text.push(' ');
            } else {
                text.push(char::from(*letter));
            }
            if index + 1 < initials.letters.len() {
                text.push(' ');
                text.push(' ');
            }
        }
        text
    }

    fn add_play_overlay(&self, scene: &mut Scene) {
        let center_x = self.viewport_width as i32 / 2;
        let top_y = HUD_MARGIN;
        scene.overlay_text.push(ScreenText {
            position: (HUD_MARGIN, top_y),
            text: format!("SCORE {:06}", self.score),
            color: SCREEN_COLOR,
            scale: 1,
            centered: false,
        });
        let high_score = self
            .high_scores
            .first()
            .map_or(self.score, |entry| entry.score.max(self.score));
        scene.overlay_text.push(ScreenText {
            position: (self.viewport_width as i32 - HUD_MARGIN, top_y),
            text: format!("HIGH {:06}", high_score),
            color: SCREEN_COLOR,
            scale: 1,
            centered: true,
        });
        scene.overlay_text.push(ScreenText {
            position: (HUD_MARGIN, self.viewport_height as i32 - 28),
            text: format!("TANKS {:02}", self.lives),
            color: INFO_COLOR,
            scale: 1,
            centered: false,
        });
        self.add_xyzzy_indicator(scene);

        if let Some(message) = self.status_message() {
            scene.overlay_text.push(ScreenText {
                position: (center_x, 44),
                text: message.to_string(),
                color: WARNING_COLOR,
                scale: 2,
                centered: true,
            });
        }

        self.add_radar(scene);
    }

    fn add_xyzzy_indicator(&self, scene: &mut Scene) {
        let Some(rows) = self.xyzzy_indicator_rows() else {
            return;
        };

        let scale = 1u8;
        let line_height = 10 * i32::from(scale);
        let bottom_y = self.viewport_height as i32 - 28;
        let start_y = bottom_y - (rows.len() as i32 - 1) * line_height;
        for (index, (text, color)) in rows.into_iter().enumerate() {
            scene.overlay_text.push(ScreenText {
                position: (
                    self.viewport_width as i32 - HUD_MARGIN - hud_text_width(&text, scale),
                    start_y + index as i32 * line_height,
                ),
                text,
                color,
                scale,
                centered: false,
            });
        }
    }

    fn add_radar(&self, scene: &mut Scene) {
        let center = (self.viewport_width as i32 / 2, 74);
        push_ring(scene, center, RADAR_RADIUS, 24, SCREEN_COLOR_DIM);
        scene.overlay_text.push(ScreenText {
            position: (center.0, center.1 - RADAR_RADIUS - 10),
            text: "N".to_string(),
            color: SCREEN_COLOR_DIM,
            scale: 1,
            centered: true,
        });
        scene.overlay_text.push(ScreenText {
            position: (center.0, center.1 + RADAR_RADIUS + 4),
            text: "S".to_string(),
            color: SCREEN_COLOR_DIM,
            scale: 1,
            centered: true,
        });
        scene.overlay_text.push(ScreenText {
            position: (center.0 + RADAR_RADIUS + 10, center.1 - 4),
            text: "E".to_string(),
            color: SCREEN_COLOR_DIM,
            scale: 1,
            centered: true,
        });
        scene.overlay_text.push(ScreenText {
            position: (center.0 - RADAR_RADIUS - 10, center.1 - 4),
            text: "W".to_string(),
            color: SCREEN_COLOR_DIM,
            scale: 1,
            centered: true,
        });

        let sweep_end = (
            center.0 + (self.radar_sweep_angle.cos() * RADAR_RADIUS as f32) as i32,
            center.1 - (self.radar_sweep_angle.sin() * RADAR_RADIUS as f32) as i32,
        );
        scene.overlay_lines.push(ScreenLine {
            start: center,
            end: sweep_end,
            color: [
                SCREEN_COLOR_DIM[0],
                SCREEN_COLOR_DIM[1],
                SCREEN_COLOR_DIM[2],
                255,
            ],
            thickness: 1,
        });

        if let Some(enemy) = self.enemy.filter(|enemy| enemy.alive) {
            let relative = enemy.position - self.player.position;
            let scale = 0.38;
            let blip = (
                center.0 + (relative.x * scale) as i32,
                center.1 - (relative.z * scale) as i32,
            );
            let color = if self.radar_ping_brightness > 0.2 {
                [WARNING_COLOR[0], WARNING_COLOR[1], WARNING_COLOR[2], 255]
            } else {
                [SCREEN_COLOR[0], SCREEN_COLOR[1], SCREEN_COLOR[2], 255]
            };
            scene.overlay_dots.push(ScreenDot {
                center: blip,
                color,
                radius: if self.radar_ping_brightness > 0.2 {
                    3
                } else {
                    2
                },
            });
        }
    }

    fn status_message(&self) -> Option<&str> {
        if self.blocked_timer > 0.0 {
            return Some(&self.arcade.strings[10]);
        }
        let enemy = self.enemy?;
        if !enemy.alive {
            return None;
        }

        let to_enemy = enemy.position - self.player.position;
        let relative_angle = angle_delta(
            self.player.heading,
            angle_to(self.player.position, enemy.position),
        );
        let distance = distance_flat(self.player.position, enemy.position);
        if distance <= 48.0 && relative_angle.abs() < 0.24 {
            return Some(&self.arcade.strings[6]);
        }
        if relative_angle.abs() > PI * 0.68 {
            return Some(&self.arcade.strings[9]);
        }
        if relative_angle < 0.0 {
            return Some(&self.arcade.strings[7]);
        }
        if relative_angle > 0.0 || to_enemy.x >= 0.0 {
            return Some(&self.arcade.strings[8]);
        }
        None
    }

    fn add_landscape(&self, scene: &mut Scene) {
        for pair in LANDSCAPE_POINTS.windows(2) {
            scene.world_lines.push(WorldLine {
                start: pair[0],
                end: pair[1],
                brightness: 0.65,
                color: None,
            });
        }
        scene.world_lines.push(WorldLine {
            start: Vec3::new(-240.0, 0.0, 180.0),
            end: Vec3::new(300.0, 0.0, 180.0),
            brightness: 0.45,
            color: None,
        });
        scene.world_lines.push(WorldLine {
            start: VOLCANO_BASE + Vec3::new(-18.0, 0.0, -12.0),
            end: VOLCANO_TOP,
            brightness: 0.78,
            color: None,
        });
        scene.world_lines.push(WorldLine {
            start: VOLCANO_BASE + Vec3::new(18.0, 0.0, -10.0),
            end: VOLCANO_TOP,
            brightness: 0.78,
            color: None,
        });

        for index in 0..5 {
            let phase = self.title_timer * 0.9 + index as f32 * 0.7;
            let start = VOLCANO_TOP + Vec3::new(index as f32 * 1.5 - 3.0, 0.0, 0.0);
            let end = start
                + Vec3::new(
                    phase.sin() * 4.0,
                    12.0 + phase.cos().abs() * 10.0,
                    phase.cos() * 3.0,
                );
            scene.world_lines.push(WorldLine {
                start,
                end,
                brightness: 0.85,
                color: None,
            });
        }

        let moon_center = Vec3::new(48.0, 66.0, 230.0);
        let radius = 8.0;
        let mut previous = moon_center + Vec3::new(radius, 0.0, 0.0);
        for step in 1..=12 {
            let angle = step as f32 / 12.0 * TAU;
            let next = moon_center + Vec3::new(angle.cos() * radius, angle.sin() * radius, 0.0);
            scene.world_lines.push(WorldLine {
                start: previous,
                end: next,
                brightness: 0.6,
                color: None,
            });
            previous = next;
        }
    }

    fn add_obstacles(&self, scene: &mut Scene) {
        for obstacle in &self.arcade.obstacles {
            match obstacle.kind {
                ObstacleKind::NarrowPyramid => {
                    push_shape(
                        scene,
                        &NARROW_PYRAMID_VERTICES,
                        &PYRAMID_EDGES,
                        Vec3::new(obstacle.x, 0.0, obstacle.z),
                        obstacle.heading,
                        Vec3::new(1.8, 2.0, 1.8),
                        0.95,
                    );
                }
                ObstacleKind::WidePyramid => {
                    push_shape(
                        scene,
                        &WIDE_PYRAMID_VERTICES,
                        &PYRAMID_EDGES,
                        Vec3::new(obstacle.x, 0.0, obstacle.z),
                        obstacle.heading,
                        Vec3::new(2.2, 1.9, 2.2),
                        0.95,
                    );
                }
                ObstacleKind::TallBox => {
                    push_shape(
                        scene,
                        &BOX_VERTICES,
                        &BOX_EDGES,
                        Vec3::new(obstacle.x, 0.0, obstacle.z),
                        obstacle.heading,
                        Vec3::new(1.8, 4.6, 1.8),
                        0.9,
                    );
                }
                ObstacleKind::ShortBox => {
                    push_shape(
                        scene,
                        &BOX_VERTICES,
                        &BOX_EDGES,
                        Vec3::new(obstacle.x, 0.0, obstacle.z),
                        obstacle.heading,
                        Vec3::new(2.4, 2.8, 2.4),
                        0.9,
                    );
                }
            }
        }
    }

    fn add_enemy(&self, scene: &mut Scene, enemy: Enemy, brightness: f32) {
        let position = enemy_position(enemy);
        let enemy_color = self.easter_egg.invincible.then_some(GOD_ENEMY_COLOR);
        if enemy.alive {
            match enemy.kind {
                EnemyKind::SlowTank => {
                    let front_visible =
                        slow_tank_front_faces_player(position, enemy.heading, self.player.position);
                    let tread_vertices = slow_tank_tread_vertices(front_visible, enemy.tread_phase);
                    push_shape_with_color(
                        scene,
                        &SLOW_TANK_VERTICES,
                        &SLOW_TANK_EDGES,
                        position,
                        enemy.heading,
                        Vec3::new(1.25, 1.25, 1.25),
                        brightness,
                        enemy_color,
                    );
                    push_shape_with_color(
                        scene,
                        &RADAR_VERTICES,
                        &RADAR_EDGES,
                        position,
                        wrap_angle(enemy.heading + enemy.radar_heading),
                        Vec3::new(1.25, 1.25, 1.25),
                        brightness * 1.05,
                        enemy_color,
                    );
                    push_shape_with_color(
                        scene,
                        tread_vertices,
                        &TREAD_EDGES,
                        position,
                        enemy.heading,
                        Vec3::new(1.25, 1.25, 1.25),
                        brightness,
                        enemy_color,
                    );
                }
                EnemyKind::SuperTank => push_shape_with_color(
                    scene,
                    &SUPER_TANK_VERTICES,
                    &SUPER_TANK_EDGES,
                    position,
                    enemy.heading,
                    Vec3::new(1.45, 1.45, 1.45),
                    brightness * 1.08,
                    enemy_color,
                ),
                EnemyKind::Missile => push_shape_with_color(
                    scene,
                    &MISSILE_VERTICES,
                    &MISSILE_EDGES,
                    position,
                    enemy.heading,
                    Vec3::new(1.5, 1.5, 1.5),
                    brightness * 1.1,
                    enemy_color,
                ),
            }
        } else {
            add_explosion(
                scene,
                position + Vec3::new(0.0, 1.4, 0.0),
                enemy.state_timer,
            );
        }
    }

    fn add_saucer(&self, scene: &mut Scene) {
        if self.saucer.state == SaucerState::Dying {
            add_explosion(scene, self.saucer.position, self.saucer.timer);
            return;
        }
        push_shape(
            scene,
            &SAUCER_VERTICES,
            &SAUCER_EDGES,
            self.saucer.position,
            self.saucer.heading,
            Vec3::new(1.2, 1.2, 1.2),
            1.05,
        );
    }

    fn add_projectile(&self, scene: &mut Scene, projectile: Projectile, brightness: f32) {
        let direction = projectile.velocity.normalized() * 1.1;
        let color = if self.easter_egg.invincible {
            Some(match projectile.owner {
                ProjectileOwner::Player => GOD_PLAYER_PROJECTILE_COLOR,
                ProjectileOwner::Enemy => GOD_ENEMY_PROJECTILE_COLOR,
            })
        } else {
            None
        };
        scene.world_lines.push(WorldLine {
            start: projectile.position - direction,
            end: projectile.position + direction,
            brightness,
            color,
        });
    }

    fn add_player_explosion(&self, scene: &mut Scene) {
        let center = self.player.position + Vec3::new(0.0, 1.4, 0.0);
        add_explosion(scene, center, self.player.timer);
    }
}

fn push_shape(
    scene: &mut Scene,
    vertices: &[Vec3],
    edges: &[(usize, usize)],
    position: Vec3,
    heading: f32,
    scale: Vec3,
    brightness: f32,
) {
    push_shape_with_color(
        scene, vertices, edges, position, heading, scale, brightness, None,
    );
}

#[allow(clippy::too_many_arguments)]
fn push_shape_with_color(
    scene: &mut Scene,
    vertices: &[Vec3],
    edges: &[(usize, usize)],
    position: Vec3,
    heading: f32,
    scale: Vec3,
    brightness: f32,
    color: Option<[u8; 4]>,
) {
    for &(start, end) in edges {
        let start = transform_vertex(vertices[start], position, heading, scale);
        let end = transform_vertex(vertices[end], position, heading, scale);
        scene.world_lines.push(WorldLine {
            start,
            end,
            brightness,
            color,
        });
    }
}

fn push_ring(scene: &mut Scene, center: (i32, i32), radius: i32, segments: usize, color: [u8; 4]) {
    let mut previous = (center.0 + radius, center.1);
    for step in 1..=segments {
        let angle = step as f32 / segments as f32 * TAU;
        let next = (
            center.0 + (angle.cos() * radius as f32) as i32,
            center.1 + (angle.sin() * radius as f32) as i32,
        );
        scene.overlay_lines.push(ScreenLine {
            start: previous,
            end: next,
            color,
            thickness: 1,
        });
        previous = next;
    }
}

fn add_explosion(scene: &mut Scene, center: Vec3, timer: f32) {
    let intensity = 1.0 + timer * 0.35;
    for step in 0..8 {
        let angle = step as f32 / 8.0 * TAU + timer * 0.7;
        let direction = Vec3::new(angle.cos(), (angle * 1.8).sin() * 0.4 + 0.2, angle.sin());
        scene.world_lines.push(WorldLine {
            start: center,
            end: center + direction * (4.0 + timer * 5.0),
            brightness: intensity,
            color: None,
        });
    }
}

fn push_tank_icon(scene: &mut Scene, origin: (i32, i32), scale: i32, color: [u8; 4]) {
    let segments = [
        ((-6, 5), (-3, -1)),
        ((-3, -1), (3, -1)),
        ((3, -1), (6, 5)),
        ((6, 5), (3, 5)),
        ((3, 5), (3, 8)),
        ((3, 8), (-3, 8)),
        ((-3, 8), (-3, 5)),
        ((-3, 5), (-6, 5)),
        ((-1, -3), (1, -3)),
        ((-1, -3), (0, -7)),
        ((1, -3), (0, -7)),
    ];

    for (start, end) in segments {
        scene.overlay_lines.push(ScreenLine {
            start: (origin.0 + start.0 * scale, origin.1 + start.1 * scale),
            end: (origin.0 + end.0 * scale, origin.1 + end.1 * scale),
            color,
            thickness: scale.max(1),
        });
    }
}

fn format_arcade_score(score: u32) -> String {
    format!("{:04}000", (score / 1_000).min(9_999))
}

fn transform_vertex(vertex: Vec3, position: Vec3, heading: f32, scale: Vec3) -> Vec3 {
    let scaled = Vec3::new(vertex.x * scale.x, vertex.y * scale.y, vertex.z * scale.z);
    rotate_y(scaled, -heading) + position
}

fn slow_tank_front_faces_player(position: Vec3, heading: f32, player_position: Vec3) -> bool {
    let local_to_player = rotate_y(player_position - position, heading);
    local_to_player.z >= 0.0
}

fn slow_tank_tread_vertices(front_visible: bool, tread_phase: f32) -> &'static [Vec3] {
    let frame = tread_phase.floor() as usize % 4;
    match (front_visible, frame) {
        (true, 0) => &FRONT_TREAD_0_VERTICES,
        (true, 1) => &FRONT_TREAD_1_VERTICES,
        (true, 2) => &FRONT_TREAD_2_VERTICES,
        (true, _) => &FRONT_TREAD_3_VERTICES,
        (false, 0) => &REAR_TREAD_0_VERTICES,
        (false, 1) => &REAR_TREAD_1_VERTICES,
        (false, 2) => &REAR_TREAD_2_VERTICES,
        (false, _) => &REAR_TREAD_3_VERTICES,
    }
}

fn default_high_scores() -> Vec<HighScoreEntry> {
    vec![
        HighScoreEntry {
            initials: String::from("ACE"),
            score: 120_000,
        },
        HighScoreEntry {
            initials: String::from("DVG"),
            score: 105_000,
        },
        HighScoreEntry {
            initials: String::from("AVG"),
            score: 70_000,
        },
        HighScoreEntry {
            initials: String::from("ROM"),
            score: 60_000,
        },
        HighScoreEntry {
            initials: String::from("CPU"),
            score: 50_000,
        },
        HighScoreEntry {
            initials: String::from("TNK"),
            score: 40_000,
        },
        HighScoreEntry {
            initials: String::from("RAD"),
            score: 30_000,
        },
        HighScoreEntry {
            initials: String::from("GUN"),
            score: 20_000,
        },
        HighScoreEntry {
            initials: String::from("VEC"),
            score: 10_000,
        },
        HighScoreEntry {
            initials: String::from("COM"),
            score: 5_000,
        },
    ]
}

fn rotate_letter(letter: &mut u8, delta: i32) {
    const LETTERS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ ";

    let current = LETTERS
        .iter()
        .position(|candidate| candidate == letter)
        .unwrap_or(0) as i32;
    let next = (current + delta).rem_euclid(LETTERS.len() as i32) as usize;
    *letter = LETTERS[next];
}

fn tank_kind(missile_launch_counter: u8) -> EnemyKind {
    if (5..=127).contains(&missile_launch_counter) {
        EnemyKind::SuperTank
    } else {
        EnemyKind::SlowTank
    }
}

fn tread_speed(axis: i8) -> f32 {
    match axis.cmp(&0) {
        std::cmp::Ordering::Greater => PLAYER_MOVE_SPEED,
        std::cmp::Ordering::Less => -PLAYER_REVERSE_SPEED,
        std::cmp::Ordering::Equal => 0.0,
    }
}

fn tread_input_for_heading(current_heading: f32, target_heading: f32, throttle: i8) -> UpdateInput {
    let delta = angle_delta(current_heading, target_heading);
    let (left_axis, right_axis) = if delta.abs() > 0.55 {
        if delta > 0.0 { (-1, 1) } else { (1, -1) }
    } else if throttle == 0 {
        if delta.abs() < 0.12 {
            (0, 0)
        } else if delta > 0.0 {
            (0, 1)
        } else {
            (1, 0)
        }
    } else if delta.abs() > 0.18 {
        if delta > 0.0 {
            (throttle, 0)
        } else {
            (0, throttle)
        }
    } else {
        (throttle, throttle)
    };

    let mut input = UpdateInput::default();
    set_tread_axis(&mut input, left_axis, true);
    set_tread_axis(&mut input, right_axis, false);
    input
}

fn set_tread_axis(input: &mut UpdateInput, axis: i8, left: bool) {
    match (left, axis.cmp(&0)) {
        (true, std::cmp::Ordering::Greater) => input.left_tread_forward = true,
        (true, std::cmp::Ordering::Less) => input.left_tread_backward = true,
        (false, std::cmp::Ordering::Greater) => input.right_tread_forward = true,
        (false, std::cmp::Ordering::Less) => input.right_tread_backward = true,
        _ => {}
    }
}

fn hud_text_width(text: &str, scale: u8) -> i32 {
    let scale = i32::from(scale.max(1));
    (text.chars().count() as i32 * 6 * scale).saturating_sub(scale)
}

fn enemy_radius(kind: EnemyKind) -> f32 {
    match kind {
        EnemyKind::SlowTank => 4.0,
        EnemyKind::SuperTank => 4.3,
        EnemyKind::Missile => 2.8,
    }
}

fn enemy_position(enemy: Enemy) -> Vec3 {
    enemy.position + Vec3::new(0.0, enemy.missile_height, 0.0)
}

fn angle_to(from: Vec3, to: Vec3) -> f32 {
    let delta = to - from;
    wrap_angle(delta.x.atan2(delta.z))
}

fn angle_delta(from: f32, to: f32) -> f32 {
    let mut delta = wrap_angle(to) - wrap_angle(from);
    if delta > PI {
        delta -= TAU;
    } else if delta < -PI {
        delta += TAU;
    }
    delta
}

fn rotate_toward(current: f32, target: f32, max_step: f32) -> f32 {
    let delta = angle_delta(current, target);
    if delta.abs() <= max_step {
        wrap_angle(target)
    } else if delta > 0.0 {
        wrap_angle(current + max_step)
    } else {
        wrap_angle(current - max_step)
    }
}

fn distance_flat(left: Vec3, right: Vec3) -> f32 {
    let dx = left.x - right.x;
    let dz = left.z - right.z;
    (dx * dx + dz * dz).sqrt()
}

fn distance(left: Vec3, right: Vec3) -> f32 {
    (left - right).length()
}

fn distance_point_to_segment_flat(start: Vec3, end: Vec3, point: Vec3) -> f32 {
    let segment = end - start;
    let length_sq = segment.x * segment.x + segment.z * segment.z;
    if length_sq <= f32::EPSILON {
        return distance_flat(start, point);
    }

    let offset = point - start;
    let t = ((offset.x * segment.x + offset.z * segment.z) / length_sq).clamp(0.0, 1.0);
    let nearest = Vec3::new(start.x + segment.x * t, 0.0, start.z + segment.z * t);
    distance_flat(nearest, point)
}

fn wrap_angle(angle: f32) -> f32 {
    angle.rem_euclid(TAU)
}

fn lerp_f32(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t.clamp(0.0, 1.0)
}

fn clamp_to_world(position: Vec3) -> Vec3 {
    Vec3::new(
        position.x.clamp(-WORLD_LIMIT, WORLD_LIMIT),
        position.y,
        position.z.clamp(-WORLD_LIMIT, WORLD_LIMIT),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        EnemyKind, Game, PlayerState, Projectile, ProjectileOwner, SaucerState, enemy_radius,
        tank_kind,
    };
    use crate::input::UpdateInput;
    use crate::math::Vec3;

    #[test]
    fn tank_progression_tracks_missile_counter() {
        assert_eq!(tank_kind(u8::MAX), EnemyKind::SlowTank);
        assert_eq!(tank_kind(4), EnemyKind::SlowTank);
        assert_eq!(tank_kind(5), EnemyKind::SuperTank);
        assert_eq!(tank_kind(127), EnemyKind::SuperTank);
        assert_eq!(tank_kind(128), EnemyKind::SlowTank);
    }

    #[test]
    fn player_collision_rejects_obstacle_overlap() {
        let game = Game::new();
        assert!(!game.position_is_walkable(crate::math::Vec3::new(32.0, 0.0, 32.0), 2.0));
    }

    #[test]
    fn destroying_enemy_awards_score() {
        let mut game = Game::new();
        game.start_game();
        game.enemy = Some(super::Enemy {
            kind: EnemyKind::Missile,
            position: crate::math::Vec3::new(0.0, 0.0, 12.0),
            heading: 0.0,
            desired_heading: 0.0,
            radar_heading: 0.0,
            tread_phase: 0.0,
            sleep_timer: 0.0,
            state_timer: 0.0,
            decision_timer: 0.0,
            shot_cooldown: 0.0,
            missile_height: 0.0,
            missile_vertical_velocity: 0.0,
            alive: true,
        });
        game.destroy_enemy();
        assert_eq!(game.score, 2_000);
    }

    #[test]
    fn player_death_transitions_to_dying_state() {
        let mut game = Game::new();
        game.start_game();
        game.player.state = PlayerState::Alive;
        game.kill_player();
        assert_eq!(game.player.state, PlayerState::Dying);
        assert_eq!(game.enemy_score, 1_000);
    }

    #[test]
    fn enemy_hit_radius_matches_expectation() {
        assert!(enemy_radius(EnemyKind::SuperTank) > enemy_radius(EnemyKind::Missile));
    }

    #[test]
    fn title_start_request_enters_play_mode() {
        let mut game = Game::new();
        game.update_with_input(
            0.016,
            UpdateInput {
                start_requested: true,
                ..UpdateInput::default()
            },
        );
        assert!(matches!(game.mode, super::Mode::Playing));
    }

    #[test]
    fn xyzzy_toggles_secret_mode_and_resets_runtime_flags() {
        let mut game = Game::new();
        game.start_game();

        game.update_with_input(
            0.0,
            UpdateInput {
                typed_chars: vec!['x', 'y', 'z', 'z', 'y'],
                ..UpdateInput::default()
            },
        );
        assert!(game.easter_egg.active);
        assert!(!game.easter_egg.invincible);
        assert!(!game.autopilot);
        assert_eq!(game.easter_egg.fire_level, 0);

        game.update_with_input(
            0.0,
            UpdateInput {
                typed_chars: vec!['g', 'f'],
                autopilot_toggle_requested: true,
                ..UpdateInput::default()
            },
        );
        assert!(game.easter_egg.invincible);
        assert!(game.autopilot);
        assert_eq!(game.easter_egg.fire_level, 1);

        game.update_with_input(
            0.0,
            UpdateInput {
                typed_chars: vec!['x', 'y', 'z', 'z', 'y'],
                ..UpdateInput::default()
            },
        );
        assert!(!game.easter_egg.active);
        assert!(!game.easter_egg.invincible);
        assert!(!game.autopilot);
        assert_eq!(game.easter_egg.fire_level, 0);

        game.update_with_input(
            0.0,
            UpdateInput {
                typed_chars: vec!['x', 'y', 'z', 'z', 'y'],
                ..UpdateInput::default()
            },
        );
        assert!(game.easter_egg.active);
        assert!(!game.easter_egg.invincible);
        assert!(!game.autopilot);
        assert_eq!(game.easter_egg.fire_level, 0);
    }

    #[test]
    fn god_mode_blocks_enemy_projectile_hits() {
        let mut game = Game::new();
        game.start_game();
        game.player.state = PlayerState::Alive;
        game.player.spawn_grace_timer = 0.0;
        game.easter_egg.active = true;
        game.easter_egg.invincible = true;

        let projectile = Projectile {
            owner: ProjectileOwner::Enemy,
            position: game.player.position + Vec3::new(0.0, 1.6, 0.0),
            velocity: Vec3::new(0.0, 0.0, 12.0),
            ttl: 1.0,
        };

        let result = game.advance_projectile(projectile, 0.0);
        assert!(result.is_some());
        assert_eq!(game.player.state, PlayerState::Alive);
    }

    #[test]
    fn enemy_projectile_can_destroy_saucer_without_awarding_points() {
        let mut game = Game::new();
        game.start_game();
        game.saucer.state = SaucerState::Alive;
        game.saucer.position = Vec3::new(0.0, super::SAUCER_HEIGHT, 8.0);
        game.score = 0;

        let projectile = Projectile {
            owner: ProjectileOwner::Enemy,
            position: game.saucer.position,
            velocity: Vec3::new(0.0, 0.0, 0.0),
            ttl: 1.0,
        };

        let result = game.advance_projectile(projectile, 0.0);
        assert!(result.is_none());
        assert_eq!(game.saucer.state, SaucerState::Dying);
        assert_eq!(game.score, 0);
    }

    #[test]
    fn saucer_stays_active_until_destroyed() {
        let mut game = Game::new();
        game.start_game();
        game.score = game.arcade.saucer_score_threshold;
        game.saucer.state = SaucerState::Inactive;
        game.saucer.timer = 0.0;

        game.update_saucer(0.1);
        assert_eq!(game.saucer.state, SaucerState::Alive);

        let initial_position = game.saucer.position;
        game.saucer.timer = 0.01;
        game.update_saucer(0.02);
        assert_eq!(game.saucer.state, SaucerState::Alive);
        assert_ne!(game.saucer.position, initial_position);
    }

    #[test]
    fn saucer_uses_arcade_ground_and_timer_ranges() {
        assert_eq!(super::SAUCER_HEIGHT, 0.0);
        assert!((super::SAUCER_DIRECTION_CHANGE_MAX - (128.0 / 15.0)).abs() < 0.001);
        assert!((super::SAUCER_RESPAWN_DELAY_MAX - (256.0 / 15.0)).abs() < 0.001);
    }

    #[test]
    fn respawn_keeps_alive_tank_but_randomizes_player_spawn() {
        let mut game = Game::with_seed(7);
        game.start_game();
        game.enemy = Some(super::Enemy {
            kind: EnemyKind::SlowTank,
            position: crate::math::Vec3::new(0.0, 0.0, 72.0),
            heading: 0.0,
            desired_heading: 0.0,
            radar_heading: 0.0,
            tread_phase: 0.0,
            sleep_timer: 0.0,
            state_timer: 0.0,
            decision_timer: 0.0,
            shot_cooldown: 0.0,
            missile_height: 0.0,
            missile_vertical_velocity: 0.0,
            alive: true,
        });

        game.respawn_player();

        assert_eq!(
            game.enemy.expect("enemy should remain active").kind,
            EnemyKind::SlowTank
        );
        assert_eq!(
            game.enemy
                .expect("enemy should remain active")
                .decision_timer,
            super::RESPAWN_ENEMY_SLEEP_SECONDS
        );
        assert_eq!(
            game.enemy.expect("enemy should remain active").heading,
            game.enemy
                .expect("enemy should remain active")
                .desired_heading
        );
        assert!(
            game.enemy
                .expect("enemy should remain active")
                .shot_cooldown
                >= super::RESPAWN_ENEMY_SLEEP_SECONDS
        );
        assert!(game.position_is_walkable(game.player.position, super::PLAYER_RADIUS));
        assert_ne!(game.player.position, Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn respawn_replaces_active_missile_with_tank() {
        let mut game = Game::with_seed(11);
        game.start_game();
        game.enemy = Some(super::Enemy {
            kind: EnemyKind::Missile,
            position: crate::math::Vec3::new(0.0, 0.0, 96.0),
            heading: 0.0,
            desired_heading: 0.0,
            radar_heading: 0.0,
            tread_phase: 0.0,
            sleep_timer: 0.0,
            state_timer: 0.0,
            decision_timer: 0.0,
            shot_cooldown: 0.0,
            missile_height: 0.0,
            missile_vertical_velocity: 0.0,
            alive: true,
        });

        game.respawn_player();

        assert_ne!(
            game.enemy.expect("respawn should create a tank").kind,
            EnemyKind::Missile
        );
    }

    #[test]
    fn existing_tank_cannot_fire_during_respawn_sleep_window() {
        let mut game = Game::with_seed(19);
        game.start_game();
        game.player.state = PlayerState::Respawning;
        game.player.spawn_grace_timer = 0.0;
        game.enemy_projectile = None;
        game.enemy = Some(super::Enemy {
            kind: EnemyKind::SlowTank,
            position: crate::math::Vec3::new(0.0, 0.0, 48.0),
            heading: super::angle_to(crate::math::Vec3::new(0.0, 0.0, 48.0), game.player.position),
            desired_heading: 1.25,
            radar_heading: 0.0,
            tread_phase: 0.0,
            sleep_timer: 1.0,
            state_timer: 0.0,
            decision_timer: 1.0,
            shot_cooldown: 0.0,
            missile_height: 0.0,
            missile_vertical_velocity: 0.0,
            alive: true,
        });

        game.update_enemy(0.0);

        assert!(game.enemy_projectile.is_none());
    }

    #[test]
    fn tank_pressure_replaces_tank_with_missile() {
        let mut game = Game::with_seed(13);
        game.start_game();
        game.player.state = PlayerState::Alive;
        game.player.spawn_grace_timer = 0.0;
        game.enemy = Some(super::Enemy {
            kind: EnemyKind::SlowTank,
            position: crate::math::Vec3::new(0.0, 0.0, 72.0),
            heading: 0.0,
            desired_heading: 0.0,
            radar_heading: 0.0,
            tread_phase: 0.0,
            sleep_timer: 0.0,
            state_timer: 0.0,
            decision_timer: 1.0,
            shot_cooldown: 2.0,
            missile_height: 0.0,
            missile_vertical_velocity: 0.0,
            alive: true,
        });
        game.tank_pressure_timer = 0.0;

        game.update_enemy(0.0);

        assert_eq!(
            game.enemy.expect("pressure should replace tank").kind,
            EnemyKind::Missile
        );
    }

    #[test]
    fn evaded_missile_chains_until_timer_expires() {
        let mut game = Game::with_seed(17);
        game.start_game();
        game.player.state = PlayerState::Alive;
        game.player.position = Vec3::new(0.0, 0.0, 0.0);
        game.enemy = Some(super::Enemy {
            kind: EnemyKind::Missile,
            position: crate::math::Vec3::new(0.0, 0.0, 140.0),
            heading: 0.0,
            desired_heading: 0.0,
            radar_heading: 0.0,
            tread_phase: 0.0,
            sleep_timer: 0.0,
            state_timer: 0.0,
            decision_timer: 0.0,
            shot_cooldown: 0.0,
            missile_height: 0.0,
            missile_vertical_velocity: 0.0,
            alive: true,
        });
        game.missile_chain_timer = 8.0;

        game.update_enemy(0.0);
        assert_eq!(
            game.enemy.expect("evaded missile should chain").kind,
            EnemyKind::Missile
        );

        game.enemy = Some(super::Enemy {
            kind: EnemyKind::Missile,
            position: crate::math::Vec3::new(0.0, 0.0, 140.0),
            heading: 0.0,
            desired_heading: 0.0,
            radar_heading: 0.0,
            tread_phase: 0.0,
            sleep_timer: 0.0,
            state_timer: 0.0,
            decision_timer: 0.0,
            shot_cooldown: 0.0,
            missile_height: 0.0,
            missile_vertical_velocity: 0.0,
            alive: true,
        });
        game.player.position = Vec3::new(0.0, 0.0, 0.0);
        game.missile_chain_timer = 0.0;

        game.update_enemy(0.0);
        assert_ne!(
            game.enemy
                .expect("expired chain should fall back to tank")
                .kind,
            EnemyKind::Missile
        );
    }

    #[test]
    fn high_score_bonus_label_sits_below_the_table() {
        let game = Game::new();
        let position = game.high_score_bonus_label_position();
        let last_row_y = super::attract::HIGH_SCORE_LIST_START.1
            + (game.high_scores.len() as i16 - 1) * super::attract::HIGH_SCORE_ROW_DELTA.1;
        assert!(position.1 < last_row_y);
    }

    #[test]
    fn autopilot_toggle_requires_xyzzy_mode() {
        let mut game = Game::new();
        game.start_game();

        game.update_with_input(
            0.0,
            UpdateInput {
                autopilot_toggle_requested: true,
                ..UpdateInput::default()
            },
        );
        assert!(!game.autopilot);

        game.update_with_input(
            0.0,
            UpdateInput {
                typed_chars: vec!['x', 'y', 'z', 'z', 'y'],
                ..UpdateInput::default()
            },
        );
        assert!(game.easter_egg.active);

        game.update_with_input(
            0.0,
            UpdateInput {
                autopilot_toggle_requested: true,
                ..UpdateInput::default()
            },
        );
        assert!(game.autopilot);
    }

    #[test]
    fn xyzzy_indicator_shows_fire_god_and_auto_states() {
        let mut game = Game::new();
        assert!(game.xyzzy_indicator_rows().is_none());

        game.easter_egg.active = true;
        game.easter_egg.fire_level = 2;
        game.easter_egg.invincible = true;
        game.autopilot = true;

        let rows = game
            .xyzzy_indicator_rows()
            .expect("xyzzy rows should be present");
        let labels = rows.into_iter().map(|(text, _)| text).collect::<Vec<_>>();
        assert_eq!(
            labels,
            vec![
                String::from("XYZZY MODE"),
                String::from("FIRE RATE 3"),
                String::from("GOD ON"),
                String::from("AUTO ON"),
            ]
        );
    }
}
