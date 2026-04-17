//! Owns the Battlezone game state, arcade rules, attract mode, and frame generation.

use std::f32::consts::{PI, TAU};

use crate::{
    arcade::{self, ArcadeTables, ObstacleKind},
    constants::{GAME_TITLE, INFO_COLOR, SCREEN_COLOR, SCREEN_COLOR_DIM, WARNING_COLOR},
    input::UpdateInput,
    math::{Vec3, forward, rotate_y},
    render::{Camera, Scene, ScreenDot, ScreenLine, ScreenText, WorldLine},
};

const PLAYER_RADIUS: f32 = 2.4;
const PLAYER_EYE_HEIGHT: f32 = 2.8;
const PLAYER_MOVE_SPEED: f32 = 24.0;
const PLAYER_REVERSE_SPEED: f32 = 14.0;
const PLAYER_TURN_SPEED: f32 = 2.45;
const PLAYER_SHELL_SPEED: f32 = 92.0;
const ENEMY_SHELL_SPEED: f32 = 74.0;
const PLAYER_SHELL_LIFETIME: f32 = 2.2;
const ENEMY_SHELL_LIFETIME: f32 = 2.8;
const PLAYER_RESPAWN_DELAY: f32 = 1.8;
const PLAYER_DYING_DELAY: f32 = 2.4;
const WORLD_LIMIT: f32 = 124.0;
const TITLE_CAMERA_RADIUS: f32 = 24.0;
const RADAR_RADIUS: i32 = 54;
const HUD_MARGIN: i32 = 18;
const MESSAGE_DURATION: f32 = 1.6;

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

const SLOW_TANK_VERTICES: [Vec3; 14] = [
    Vec3::new(-1.8, 0.0, -3.2),
    Vec3::new(1.8, 0.0, -3.2),
    Vec3::new(1.8, 0.8, -1.0),
    Vec3::new(-1.8, 0.8, -1.0),
    Vec3::new(-1.8, 0.0, 3.4),
    Vec3::new(1.8, 0.0, 3.4),
    Vec3::new(1.8, 0.8, 3.4),
    Vec3::new(-1.8, 0.8, 3.4),
    Vec3::new(-0.8, 1.4, 0.2),
    Vec3::new(0.8, 1.4, 0.2),
    Vec3::new(0.0, 1.4, 4.6),
    Vec3::new(-1.1, 1.6, -0.4),
    Vec3::new(1.1, 1.6, -0.4),
    Vec3::new(0.0, 2.2, -1.5),
];

const SLOW_TANK_EDGES: [(usize, usize); 20] = [
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
    (8, 9),
    (8, 10),
    (9, 10),
    (11, 12),
    (11, 13),
    (12, 13),
    (11, 8),
    (12, 9),
];

const SUPER_TANK_VERTICES: [Vec3; 12] = [
    Vec3::new(-2.4, 0.0, -3.6),
    Vec3::new(2.4, 0.0, -3.6),
    Vec3::new(1.9, 1.4, -0.6),
    Vec3::new(-1.9, 1.4, -0.6),
    Vec3::new(-2.4, 0.0, 3.4),
    Vec3::new(2.4, 0.0, 3.4),
    Vec3::new(2.0, 1.1, 3.4),
    Vec3::new(-2.0, 1.1, 3.4),
    Vec3::new(-0.8, 1.9, 1.0),
    Vec3::new(0.8, 1.9, 1.0),
    Vec3::new(-0.6, 1.4, 5.0),
    Vec3::new(0.6, 1.4, 5.0),
];

const SUPER_TANK_EDGES: [(usize, usize); 18] = [
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
    (8, 9),
    (8, 10),
    (9, 11),
    (8, 3),
    (9, 2),
    (10, 11),
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

const SAUCER_VERTICES: [Vec3; 10] = [
    Vec3::new(-2.4, 0.0, 0.0),
    Vec3::new(-1.4, 0.8, -1.8),
    Vec3::new(0.0, 1.1, -2.4),
    Vec3::new(1.4, 0.8, -1.8),
    Vec3::new(2.4, 0.0, 0.0),
    Vec3::new(1.4, -0.5, 1.8),
    Vec3::new(0.0, -0.7, 2.6),
    Vec3::new(-1.4, -0.5, 1.8),
    Vec3::new(-0.7, 0.9, 0.0),
    Vec3::new(0.7, 0.9, 0.0),
];

const SAUCER_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 4),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 0),
    (1, 8),
    (8, 2),
    (3, 9),
    (9, 4),
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

pub struct Game {
    arcade: &'static ArcadeTables,
    rng: fastrand::Rng,
    mode: Mode,
    player: Player,
    score: u32,
    lives: u32,
    next_bonus_tank_index: usize,
    high_scores: Vec<HighScoreEntry>,
    enemy: Option<Enemy>,
    player_projectile: Option<Projectile>,
    enemy_projectile: Option<Projectile>,
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
            lives: arcade.starting_lives,
            next_bonus_tank_index: 0,
            high_scores: default_high_scores(),
            enemy: None,
            player_projectile: None,
            enemy_projectile: None,
            saucer: Saucer {
                position: Vec3::new(0.0, 12.0, 0.0),
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
        self.prompt_timer += dt;
        if self.prompt_timer >= 0.35 {
            self.prompt_timer -= 0.35;
            self.prompt_visible = !self.prompt_visible;
        }
        self.blocked_timer = (self.blocked_timer - dt).max(0.0);
        self.radar_ping_brightness = (self.radar_ping_brightness - dt * 1.8).max(0.0);

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
            }
        }
    }

    fn update_playing(&mut self, dt: f32, input: UpdateInput) {
        self.title_timer += dt;
        self.radar_sweep_angle = wrap_angle(self.radar_sweep_angle + dt * 2.1);

        self.update_player_state(dt);
        self.update_saucer(dt);
        self.update_enemy(dt);
        self.update_projectiles(dt);

        if self.player.state == PlayerState::Alive {
            self.update_player_movement(dt, input);
            self.handle_player_fire(input);
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

    fn update_player_movement(&mut self, dt: f32, input: UpdateInput) {
        if input.turn_left {
            self.player.heading = wrap_angle(self.player.heading - PLAYER_TURN_SPEED * dt);
        }
        if input.turn_right {
            self.player.heading = wrap_angle(self.player.heading + PLAYER_TURN_SPEED * dt);
        }

        let mut speed = 0.0;
        if input.forward {
            speed += PLAYER_MOVE_SPEED;
        }
        if input.backward {
            speed -= PLAYER_REVERSE_SPEED;
        }
        if speed.abs() <= f32::EPSILON {
            return;
        }

        let candidate = self.player.position + forward(self.player.heading) * (speed * dt);
        if self.position_is_walkable(candidate, PLAYER_RADIUS) {
            self.player.position = clamp_to_world(candidate);
        } else {
            self.blocked_timer = MESSAGE_DURATION;
        }
    }

    fn handle_player_fire(&mut self, input: UpdateInput) {
        if !input.fire
            || self.player_projectile.is_some()
            || self.player.state != PlayerState::Alive
        {
            return;
        }

        let velocity = forward(self.player.heading) * PLAYER_SHELL_SPEED;
        self.player_projectile = Some(Projectile {
            owner: ProjectileOwner::Player,
            position: self.player.position
                + Vec3::new(0.0, PLAYER_EYE_HEIGHT - 0.2, 0.0)
                + forward(self.player.heading) * 3.6,
            velocity,
            ttl: PLAYER_SHELL_LIFETIME,
        });
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
        enemy.decision_timer -= dt;
        enemy.shot_cooldown = (enemy.shot_cooldown - dt).max(0.0);

        match enemy.kind {
            EnemyKind::SlowTank | EnemyKind::SuperTank => {
                if enemy.decision_timer <= 0.0 {
                    let aim = angle_to(enemy.position, self.player.position);
                    let inaccuracy = if enemy.kind == EnemyKind::SlowTank {
                        self.rng.f32() * 0.35 - 0.175
                    } else {
                        self.rng.f32() * 0.12 - 0.06
                    };
                    enemy.desired_heading = wrap_angle(aim + inaccuracy);
                    enemy.decision_timer = if enemy.kind == EnemyKind::SlowTank {
                        1.6 + self.rng.f32() * 1.4
                    } else {
                        0.55 + self.rng.f32() * 0.45
                    };
                }

                let turn_rate = match enemy.kind {
                    EnemyKind::SlowTank => 1.1,
                    EnemyKind::SuperTank => 2.1,
                    EnemyKind::Missile => 0.0,
                };
                enemy.heading = rotate_toward(enemy.heading, enemy.desired_heading, turn_rate * dt);

                let move_speed = match enemy.kind {
                    EnemyKind::SlowTank => 10.0,
                    EnemyKind::SuperTank => 16.0,
                    EnemyKind::Missile => 0.0,
                };
                let candidate = enemy.position + forward(enemy.heading) * (move_speed * dt);
                if self.enemy_position_is_walkable(candidate, 3.4) {
                    enemy.position = clamp_to_world(candidate);
                } else {
                    enemy.desired_heading = wrap_angle(enemy.desired_heading + PI * 0.5);
                    enemy.decision_timer = 0.05;
                }

                let distance = distance_flat(enemy.position, self.player.position);
                let aim_error = angle_delta(
                    enemy.heading,
                    angle_to(enemy.position, self.player.position),
                )
                .abs();
                if self.enemy_projectile.is_none()
                    && enemy.shot_cooldown <= 0.0
                    && self.player.spawn_grace_timer <= 0.0
                    && distance <= 84.0
                    && aim_error < 0.16
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
                        2.4
                    } else {
                        1.3
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

        self.update_radar_ping(enemy.position);
        self.enemy = Some(enemy);
    }

    fn update_projectiles(&mut self, dt: f32) {
        self.player_projectile = self.advance_projectile(self.player_projectile, dt);
        self.enemy_projectile = self.advance_projectile(self.enemy_projectile, dt);
    }

    fn advance_projectile(
        &mut self,
        projectile: Option<Projectile>,
        dt: f32,
    ) -> Option<Projectile> {
        let mut projectile = projectile?;
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
                    && distance(projectile.position, self.saucer.position) <= 4.2
                {
                    self.score += 5_000;
                    self.check_bonus_tank_award();
                    self.saucer.state = SaucerState::Dying;
                    self.saucer.timer = 1.2;
                    self.events.push(GameEvent::SaucerDestroyed);
                    return None;
                }
            }
            ProjectileOwner::Enemy => {
                if self.player.state == PlayerState::Alive
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
                    self.saucer.state = SaucerState::Alive;
                    self.saucer.position = Vec3::new(
                        self.rng.f32() * 200.0 - 100.0,
                        14.0,
                        self.rng.f32() * 200.0 - 100.0,
                    );
                    self.saucer.velocity = Vec3::new(
                        self.rng.f32() * 18.0 - 9.0,
                        0.0,
                        self.rng.f32() * 18.0 - 9.0,
                    );
                    self.saucer.timer = 5.0 + self.rng.f32() * 7.0;
                }
            }
            SaucerState::Alive => {
                self.saucer.position += self.saucer.velocity * dt;
                self.saucer.position = clamp_to_world(self.saucer.position);
                self.saucer.heading = wrap_angle(self.saucer.heading + dt * 2.8);
                self.saucer.timer -= dt;
                if self.saucer.timer <= 0.0 {
                    self.saucer.state = SaucerState::Inactive;
                    self.saucer.timer = self.rng.f32() * 17.0;
                }
            }
            SaucerState::Dying => {
                self.saucer.heading = wrap_angle(self.saucer.heading + dt * 5.4);
                self.saucer.timer -= dt;
                if self.saucer.timer <= 0.0 {
                    self.saucer.state = SaucerState::Inactive;
                    self.saucer.timer = self.rng.f32() * 17.0;
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

    fn kill_player(&mut self) {
        if self.player.state != PlayerState::Alive {
            return;
        }

        self.player.state = PlayerState::Dying;
        self.player.timer = PLAYER_DYING_DELAY;
        self.player.spawn_grace_timer = 0.0;
        self.player_projectile = None;
        self.enemy_projectile = None;
        self.lives = self.lives.saturating_sub(1);
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
        self.player.position = Vec3::new(0.0, 0.0, 0.0);
        self.player.heading = 0.0;
        self.player.state = PlayerState::Respawning;
        self.player.timer = PLAYER_RESPAWN_DELAY;
        self.player.spawn_grace_timer = PLAYER_RESPAWN_DELAY;
        self.player_projectile = None;
        self.enemy_projectile = None;
        self.spawn_enemy(Some(EnemyKind::SlowTank));
    }

    fn start_game(&mut self) {
        self.mode = Mode::Playing;
        self.score = 0;
        self.lives = self.arcade.starting_lives;
        self.next_bonus_tank_index = 0;
        self.enemy = None;
        self.player_projectile = None;
        self.enemy_projectile = None;
        self.player.position = Vec3::new(0.0, 0.0, 0.0);
        self.player.heading = 0.0;
        self.player.state = PlayerState::Respawning;
        self.player.timer = PLAYER_RESPAWN_DELAY;
        self.player.spawn_grace_timer = PLAYER_RESPAWN_DELAY;
        self.saucer.state = SaucerState::Inactive;
        self.saucer.timer = 4.0;
        self.spawn_enemy(Some(EnemyKind::SlowTank));
        self.events.push(GameEvent::GameStarted);
    }

    fn enter_title_mode(&mut self) {
        self.mode = Mode::Title;
        self.initials = None;
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
            state_timer: 0.0,
            decision_timer: 1.0,
            shot_cooldown: 0.0,
            missile_height: 0.0,
            missile_vertical_velocity: 0.0,
            alive: true,
        });
        self.saucer = Saucer {
            position: Vec3::new(-36.0, 14.0, 86.0),
            velocity: Vec3::new(8.0, 0.0, -3.0),
            heading: 0.0,
            timer: 6.0,
            state: SaucerState::Alive,
        };
    }

    fn begin_initials_entry(&mut self) {
        self.mode = Mode::EnteringInitials;
        self.initials = Some(InitialsEntry {
            letters: [b'A', b'A', b'A'],
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
        self.high_scores.truncate(5);
        self.enter_title_mode();
    }

    fn qualifies_for_high_score(&self, score: u32) -> bool {
        self.high_scores.len() < 5
            || self
                .high_scores
                .last()
                .is_some_and(|entry| score > entry.score)
    }

    fn spawn_enemy(&mut self, preferred: Option<EnemyKind>) {
        let kind = preferred.unwrap_or_else(|| self.choose_enemy_kind());

        for _ in 0..64 {
            let (spawn_heading, spawn_distance) = if kind == EnemyKind::Missile {
                (
                    wrap_angle(self.player.heading + self.rng.f32() * 0.45 - 0.225),
                    self.arcade.far_spawn_distance,
                )
            } else {
                (
                    self.rng.f32() * TAU,
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
            self.enemy = Some(Enemy {
                kind,
                position: clamp_to_world(position),
                heading: angle_to(position, self.player.position),
                desired_heading: angle_to(position, self.player.position),
                state_timer: 0.0,
                decision_timer: 0.45,
                shot_cooldown: 1.4,
                missile_height: 0.0,
                missile_vertical_velocity: 0.0,
                alive: true,
            });
            return;
        }

        self.enemy = Some(Enemy {
            kind,
            position: Vec3::new(32.0, 0.0, 64.0),
            heading: PI,
            desired_heading: PI,
            state_timer: 0.0,
            decision_timer: 0.6,
            shot_cooldown: 1.2,
            missile_height: 0.0,
            missile_vertical_velocity: 0.0,
            alive: true,
        });
    }

    fn choose_enemy_kind(&mut self) -> EnemyKind {
        if self.score < self.arcade.missile_score_threshold {
            return tank_kind(self.score);
        }
        if self.rng.bool() {
            EnemyKind::Missile
        } else {
            tank_kind(self.score)
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

    fn title_scene(&self) -> Scene {
        let camera = Camera {
            position: Vec3::new(
                self.title_timer.sin() * TITLE_CAMERA_RADIUS,
                PLAYER_EYE_HEIGHT,
                -96.0 + self.title_timer.cos() * TITLE_CAMERA_RADIUS,
            ),
            heading: self.title_timer * 0.28,
        };
        let mut scene = Scene::empty(camera);
        scene.show_crosshair = false;
        self.add_landscape(&mut scene);
        self.add_obstacles(&mut scene);
        if let Some(enemy) = self.enemy {
            self.add_enemy(&mut scene, enemy, 0.9);
        }
        if self.saucer.state == SaucerState::Alive {
            self.add_saucer(&mut scene);
        }

        let center_x = self.viewport_width as i32 / 2;
        scene.overlay_text.push(ScreenText {
            position: (center_x, 32),
            text: GAME_TITLE.to_string(),
            color: SCREEN_COLOR,
            scale: 4,
            centered: true,
        });
        scene.overlay_text.push(ScreenText {
            position: (center_x, 74),
            text: arcade::bonus_tank_label(),
            color: INFO_COLOR,
            scale: 1,
            centered: true,
        });
        scene.overlay_text.push(ScreenText {
            position: (center_x, 96),
            text: "FAITHFUL RUST IMPLEMENTATION OF THE ORIGINAL ARCADE GAME".to_string(),
            color: SCREEN_COLOR_DIM,
            scale: 1,
            centered: true,
        });

        if self.prompt_visible {
            scene.overlay_text.push(ScreenText {
                position: (center_x, self.viewport_height as i32 - 52),
                text: self.arcade.strings[0].clone(),
                color: WARNING_COLOR,
                scale: 2,
                centered: true,
            });
        }

        scene.overlay_text.push(ScreenText {
            position: (center_x, 128),
            text: self.arcade.strings[1].clone(),
            color: SCREEN_COLOR,
            scale: 2,
            centered: true,
        });

        for (index, entry) in self.high_scores.iter().enumerate() {
            scene.overlay_text.push(ScreenText {
                position: (center_x, 160 + index as i32 * 18),
                text: format!("{:02} {} {:06}", index + 1, entry.initials, entry.score),
                color: INFO_COLOR,
                scale: 1,
                centered: true,
            });
        }

        scene.overlay_text.push(ScreenText {
            position: (center_x, self.viewport_height as i32 - 26),
            text: "W/S MOVE  A/D TURN  SPACE FIRE  ENTER START  Q QUIT".to_string(),
            color: SCREEN_COLOR_DIM,
            scale: 1,
            centered: true,
        });

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
        if let Some(projectile) = self.player_projectile {
            self.add_projectile(&mut scene, projectile, 1.1);
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
        let mut scene = self.title_scene();
        let center_x = self.viewport_width as i32 / 2;
        let center_y = self.viewport_height as i32 / 2;
        scene.overlay_text.push(ScreenText {
            position: (center_x, center_y - 42),
            text: self.arcade.strings[2].clone(),
            color: WARNING_COLOR,
            scale: 3,
            centered: true,
        });
        scene.overlay_text.push(ScreenText {
            position: (center_x, center_y - 8),
            text: self.arcade.strings[3].clone(),
            color: SCREEN_COLOR,
            scale: 2,
            centered: true,
        });

        if let Some(initials) = &self.initials {
            scene.overlay_text.push(ScreenText {
                position: (center_x, center_y + 28),
                text: format!("{:06}", initials.score),
                color: INFO_COLOR,
                scale: 2,
                centered: true,
            });
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
            scene.overlay_text.push(ScreenText {
                position: (center_x, center_y + 62),
                text,
                color: SCREEN_COLOR,
                scale: 4,
                centered: true,
            });
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
            });
        }
        scene.world_lines.push(WorldLine {
            start: Vec3::new(-240.0, 0.0, 180.0),
            end: Vec3::new(300.0, 0.0, 180.0),
            brightness: 0.45,
        });
        scene.world_lines.push(WorldLine {
            start: VOLCANO_BASE + Vec3::new(-18.0, 0.0, -12.0),
            end: VOLCANO_TOP,
            brightness: 0.78,
        });
        scene.world_lines.push(WorldLine {
            start: VOLCANO_BASE + Vec3::new(18.0, 0.0, -10.0),
            end: VOLCANO_TOP,
            brightness: 0.78,
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
        if enemy.alive {
            match enemy.kind {
                EnemyKind::SlowTank => push_shape(
                    scene,
                    &SLOW_TANK_VERTICES,
                    &SLOW_TANK_EDGES,
                    position,
                    enemy.heading,
                    Vec3::new(1.35, 1.35, 1.35),
                    brightness,
                ),
                EnemyKind::SuperTank => push_shape(
                    scene,
                    &SUPER_TANK_VERTICES,
                    &SUPER_TANK_EDGES,
                    position,
                    enemy.heading,
                    Vec3::new(1.25, 1.25, 1.25),
                    brightness * 1.08,
                ),
                EnemyKind::Missile => push_shape(
                    scene,
                    &MISSILE_VERTICES,
                    &MISSILE_EDGES,
                    position,
                    enemy.heading,
                    Vec3::new(1.5, 1.5, 1.5),
                    brightness * 1.1,
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
            Vec3::new(1.6, 1.6, 1.6),
            1.05,
        );
    }

    fn add_projectile(&self, scene: &mut Scene, projectile: Projectile, brightness: f32) {
        let direction = projectile.velocity.normalized() * 1.1;
        scene.world_lines.push(WorldLine {
            start: projectile.position - direction,
            end: projectile.position + direction,
            brightness,
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
    for &(start, end) in edges {
        let start = transform_vertex(vertices[start], position, heading, scale);
        let end = transform_vertex(vertices[end], position, heading, scale);
        scene.world_lines.push(WorldLine {
            start,
            end,
            brightness,
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
        });
    }
}

fn transform_vertex(vertex: Vec3, position: Vec3, heading: f32, scale: Vec3) -> Vec3 {
    let scaled = Vec3::new(vertex.x * scale.x, vertex.y * scale.y, vertex.z * scale.z);
    rotate_y(scaled, -heading) + position
}

fn default_high_scores() -> Vec<HighScoreEntry> {
    vec![
        HighScoreEntry {
            initials: String::from("ACE"),
            score: 60_000,
        },
        HighScoreEntry {
            initials: String::from("DVG"),
            score: 45_000,
        },
        HighScoreEntry {
            initials: String::from("AVG"),
            score: 30_000,
        },
        HighScoreEntry {
            initials: String::from("ROM"),
            score: 20_000,
        },
        HighScoreEntry {
            initials: String::from("CPU"),
            score: 10_000,
        },
    ]
}

fn rotate_letter(letter: &mut u8, delta: i32) {
    let current = i32::from(*letter - b'A');
    let next = (current + delta).rem_euclid(26) as u8;
    *letter = b'A' + next;
}

fn tank_kind(score: u32) -> EnemyKind {
    if score >= 20_000 {
        EnemyKind::SuperTank
    } else {
        EnemyKind::SlowTank
    }
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

fn wrap_angle(angle: f32) -> f32 {
    angle.rem_euclid(TAU)
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
    use super::{EnemyKind, Game, PlayerState, enemy_radius, tank_kind};
    use crate::input::UpdateInput;

    #[test]
    fn tank_progression_reaches_super_tank_scores() {
        assert_eq!(tank_kind(0), EnemyKind::SlowTank);
        assert_eq!(tank_kind(25_000), EnemyKind::SuperTank);
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
}
