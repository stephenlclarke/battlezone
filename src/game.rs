use std::f32::consts::{PI, TAU};

use crate::math::{Vec3, forward, rotate_y};

const PLAYER_RADIUS: f32 = 1.5;
const PLAYER_EYE_HEIGHT: f32 = 2.1;
const PLAYER_MOVE_SPEED: f32 = 14.0;
const PLAYER_TURN_SPEED: f32 = 2.5;
const PLAYER_RELOAD_TIME: f32 = 0.45;
const SHELL_SPEED: f32 = 48.0;
const SHELL_LIFETIME: f32 = 2.2;
const ENEMY_RADIUS: f32 = 2.0;
const ENEMY_RESPAWN_TIME: f32 = 2.5;
const WORLD_LIMIT: f32 = 70.0;

const CUBE_VERTICES: [Vec3; 8] = [
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(1.0, -1.0, 1.0),
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(-1.0, 1.0, 1.0),
];

const CUBE_EDGES: [(usize, usize); 12] = [
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

const PYRAMID_VERTICES: [Vec3; 5] = [
    Vec3::new(-1.2, -1.0, -1.2),
    Vec3::new(1.2, -1.0, -1.2),
    Vec3::new(1.2, -1.0, 1.2),
    Vec3::new(-1.2, -1.0, 1.2),
    Vec3::new(0.0, 1.4, 0.0),
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

const TANK_VERTICES: [Vec3; 12] = [
    Vec3::new(-2.0, -1.0, -3.0),
    Vec3::new(2.0, -1.0, -3.0),
    Vec3::new(2.0, 0.0, -1.0),
    Vec3::new(-2.0, 0.0, -1.0),
    Vec3::new(-2.0, -1.0, 3.0),
    Vec3::new(2.0, -1.0, 3.0),
    Vec3::new(2.0, 0.0, 3.0),
    Vec3::new(-2.0, 0.0, 3.0),
    Vec3::new(-0.9, 0.7, 0.2),
    Vec3::new(0.9, 0.7, 0.2),
    Vec3::new(0.0, 0.7, 4.8),
    Vec3::new(0.0, 1.5, 0.5),
];

const TANK_EDGES: [(usize, usize); 16] = [
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
    (8, 11),
    (9, 11),
    (11, 10),
];

#[derive(Clone, Copy, Debug, Default)]
pub struct InputState {
    pub forward: bool,
    pub backward: bool,
    pub turn_left: bool,
    pub turn_right: bool,
    pub fire: bool,
    pub quit: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub position: Vec3,
    pub heading: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct WorldLine {
    pub start: Vec3,
    pub end: Vec3,
}

#[derive(Clone, Copy, Debug)]
struct Obstacle {
    position: Vec3,
    heading: f32,
    scale: Vec3,
    radius: f32,
    kind: ShapeKind,
}

#[derive(Clone, Copy, Debug)]
enum ShapeKind {
    Cube,
    Pyramid,
    Tank,
}

#[derive(Clone, Copy, Debug)]
struct Enemy {
    position: Vec3,
    heading: f32,
    alive: bool,
    pulse: f32,
}

#[derive(Clone, Copy, Debug)]
struct Projectile {
    position: Vec3,
    velocity: Vec3,
    ttl: f32,
}

pub struct FrameData {
    pub lines: Vec<WorldLine>,
    pub hud: Vec<String>,
    pub camera: Camera,
}

pub struct Game {
    player_position: Vec3,
    player_heading: f32,
    cooldown: f32,
    score: u32,
    enemy_respawn_timer: f32,
    shell_count: u32,
    time: f32,
    enemy: Enemy,
    projectiles: Vec<Projectile>,
    obstacles: Vec<Obstacle>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            player_position: Vec3::new(0.0, 0.0, -30.0),
            player_heading: 0.0,
            cooldown: 0.0,
            score: 0,
            enemy_respawn_timer: 0.0,
            shell_count: 0,
            time: 0.0,
            enemy: Enemy {
                position: Vec3::new(0.0, 0.0, 30.0),
                heading: PI,
                alive: true,
                pulse: 0.0,
            },
            projectiles: Vec::new(),
            obstacles: vec![
                Obstacle {
                    position: Vec3::new(-14.0, 0.0, 8.0),
                    heading: 0.3,
                    scale: Vec3::new(4.5, 4.0, 4.5),
                    radius: 5.5,
                    kind: ShapeKind::Cube,
                },
                Obstacle {
                    position: Vec3::new(16.0, 0.0, 20.0),
                    heading: 0.9,
                    scale: Vec3::new(4.0, 5.5, 4.0),
                    radius: 5.0,
                    kind: ShapeKind::Pyramid,
                },
                Obstacle {
                    position: Vec3::new(-18.0, 0.0, 38.0),
                    heading: 0.0,
                    scale: Vec3::new(5.5, 4.0, 5.5),
                    radius: 6.3,
                    kind: ShapeKind::Cube,
                },
                Obstacle {
                    position: Vec3::new(10.0, 0.0, 52.0),
                    heading: 0.6,
                    scale: Vec3::new(5.0, 6.0, 5.0),
                    radius: 5.8,
                    kind: ShapeKind::Pyramid,
                },
            ],
        }
    }

    pub fn update(&mut self, dt: f32, input: InputState) {
        self.time += dt;
        self.cooldown = (self.cooldown - dt).max(0.0);

        if input.turn_left {
            self.player_heading -= PLAYER_TURN_SPEED * dt;
        }
        if input.turn_right {
            self.player_heading += PLAYER_TURN_SPEED * dt;
        }
        self.player_heading = normalize_angle(self.player_heading);

        let mut drive: f32 = 0.0;
        if input.forward {
            drive += 1.0;
        }
        if input.backward {
            drive -= 1.0;
        }
        if drive.abs() > f32::EPSILON {
            let candidate = self.player_position
                + forward(self.player_heading) * (drive * PLAYER_MOVE_SPEED * dt);
            if self.position_is_walkable(candidate) {
                self.player_position = clamp_to_world(candidate);
            }
        }

        if input.fire && self.cooldown <= f32::EPSILON {
            let direction = forward(self.player_heading);
            let origin = self.camera().position + direction * 3.2;
            self.projectiles.push(Projectile {
                position: origin,
                velocity: direction * SHELL_SPEED,
                ttl: SHELL_LIFETIME,
            });
            self.cooldown = PLAYER_RELOAD_TIME;
            self.shell_count += 1;
        }

        self.update_projectiles(dt);
        self.update_enemy(dt);
    }

    pub fn frame(&self) -> FrameData {
        let mut lines = Vec::new();
        self.push_ground_bounds(&mut lines);

        for obstacle in &self.obstacles {
            push_shape_lines(
                lines.as_mut(),
                obstacle.kind,
                obstacle.position,
                obstacle.heading,
                obstacle.scale,
            );
        }

        if self.enemy.alive {
            let pulse = self.enemy.pulse.sin() * 0.2;
            let scale = Vec3::new(1.0 + pulse, 1.0, 1.0 + pulse);
            push_shape_lines(
                lines.as_mut(),
                ShapeKind::Tank,
                self.enemy.position,
                self.enemy.heading,
                scale,
            );
        }

        for projectile in &self.projectiles {
            let tail = projectile.position - projectile.velocity.normalized() * 0.6;
            lines.push(WorldLine {
                start: tail,
                end: projectile.position,
            });
        }

        let enemy_state = if self.enemy.alive { "LOCK" } else { "CLEAR" };
        let hud = vec![
            format!(
                "SCORE {:05}  SHELLS {:03}  TARGET {}",
                self.score, self.shell_count, enemy_state
            ),
            format!(
                "POS {:+05.1} {:+05.1}  HEADING {:03.0}  RANGE {:04.1}",
                self.player_position.x,
                self.player_position.z,
                self.player_heading.to_degrees().rem_euclid(360.0),
                self.enemy_distance(),
            ),
            String::from("W/S move  A/D turn  Space fire  Q quit"),
        ];

        FrameData {
            lines,
            hud,
            camera: self.camera(),
        }
    }

    pub fn enemy_distance(&self) -> f32 {
        if self.enemy.alive {
            let delta = self.enemy.position - self.player_position;
            (delta.x * delta.x + delta.z * delta.z).sqrt()
        } else {
            0.0
        }
    }

    pub fn camera(&self) -> Camera {
        Camera {
            position: self.player_position + Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0),
            heading: self.player_heading,
        }
    }

    fn update_projectiles(&mut self, dt: f32) {
        for projectile in &mut self.projectiles {
            projectile.position += projectile.velocity * dt;
            projectile.ttl -= dt;
        }

        self.projectiles.retain(|projectile| {
            projectile.ttl > 0.0
                && projectile.position.x.abs() <= WORLD_LIMIT
                && projectile.position.z.abs() <= WORLD_LIMIT
        });

        if self.enemy.alive
            && self.projectiles.iter().any(|projectile| {
                horizontal_distance(projectile.position, self.enemy.position) <= ENEMY_RADIUS
            })
        {
            self.enemy.alive = false;
            self.enemy_respawn_timer = ENEMY_RESPAWN_TIME;
            self.score += 1000;
        }
    }

    fn update_enemy(&mut self, dt: f32) {
        self.enemy.pulse += dt * 4.0;

        if !self.enemy.alive {
            self.enemy_respawn_timer -= dt;
            if self.enemy_respawn_timer <= 0.0 {
                self.enemy = Enemy {
                    position: spawn_enemy_position(self.time),
                    heading: PI,
                    alive: true,
                    pulse: 0.0,
                };
            }
            return;
        }

        let target = self.player_position;
        let to_player = target - self.enemy.position;
        self.enemy.heading = to_player.x.atan2(to_player.z);

        let strafe = Vec3::new((self.time * 0.8).sin(), 0.0, (self.time * 0.45).cos()) * (6.0 * dt);
        let forward_step = forward(self.enemy.heading) * (2.5 * dt);
        let candidate = clamp_to_world(self.enemy.position + strafe + forward_step);
        if self.position_is_enemy_safe(candidate) {
            self.enemy.position = candidate;
        }
    }

    fn position_is_walkable(&self, candidate: Vec3) -> bool {
        if self.obstacles.iter().any(|obstacle| {
            horizontal_distance(candidate, obstacle.position) <= obstacle.radius + PLAYER_RADIUS
        }) {
            return false;
        }

        !(self.enemy.alive
            && horizontal_distance(candidate, self.enemy.position) <= ENEMY_RADIUS + PLAYER_RADIUS)
    }

    fn position_is_enemy_safe(&self, candidate: Vec3) -> bool {
        if horizontal_distance(candidate, self.player_position)
            <= ENEMY_RADIUS + PLAYER_RADIUS + 1.5
        {
            return false;
        }

        !self.obstacles.iter().any(|obstacle| {
            horizontal_distance(candidate, obstacle.position) <= obstacle.radius + ENEMY_RADIUS
        })
    }

    fn push_ground_bounds(&self, lines: &mut Vec<WorldLine>) {
        let y = 0.0;
        let min = -WORLD_LIMIT;
        let max = WORLD_LIMIT;
        let corners = [
            Vec3::new(min, y, min),
            Vec3::new(max, y, min),
            Vec3::new(max, y, max),
            Vec3::new(min, y, max),
        ];

        for index in 0..corners.len() {
            lines.push(WorldLine {
                start: corners[index],
                end: corners[(index + 1) % corners.len()],
            });
        }

        for offset in [-40.0_f32, -20.0, 0.0, 20.0, 40.0] {
            lines.push(WorldLine {
                start: Vec3::new(min, y, offset),
                end: Vec3::new(max, y, offset),
            });
            lines.push(WorldLine {
                start: Vec3::new(offset, y, min),
                end: Vec3::new(offset, y, max),
            });
        }
    }
}

fn push_shape_lines(
    lines: &mut Vec<WorldLine>,
    kind: ShapeKind,
    position: Vec3,
    heading: f32,
    scale: Vec3,
) {
    let (vertices, edges) = match kind {
        ShapeKind::Cube => (&CUBE_VERTICES[..], &CUBE_EDGES[..]),
        ShapeKind::Pyramid => (&PYRAMID_VERTICES[..], &PYRAMID_EDGES[..]),
        ShapeKind::Tank => (&TANK_VERTICES[..], &TANK_EDGES[..]),
    };

    for (start_index, end_index) in edges {
        let start = transform_vertex(vertices[*start_index], position, heading, scale);
        let end = transform_vertex(vertices[*end_index], position, heading, scale);
        lines.push(WorldLine { start, end });
    }
}

fn transform_vertex(vertex: Vec3, position: Vec3, heading: f32, scale: Vec3) -> Vec3 {
    let scaled = Vec3::new(vertex.x * scale.x, vertex.y * scale.y, vertex.z * scale.z);
    rotate_y(scaled, heading) + position
}

fn spawn_enemy_position(time: f32) -> Vec3 {
    let orbit = time * 0.7;
    Vec3::new(orbit.sin() * 28.0, 0.0, 24.0 + orbit.cos() * 22.0)
}

fn horizontal_distance(a: Vec3, b: Vec3) -> f32 {
    let dx = a.x - b.x;
    let dz = a.z - b.z;
    (dx * dx + dz * dz).sqrt()
}

fn clamp_to_world(position: Vec3) -> Vec3 {
    Vec3::new(
        position.x.clamp(-WORLD_LIMIT, WORLD_LIMIT),
        0.0,
        position.z.clamp(-WORLD_LIMIT, WORLD_LIMIT),
    )
}

fn normalize_angle(angle: f32) -> f32 {
    let mut value = angle % TAU;
    if value < -PI {
        value += TAU;
    } else if value > PI {
        value -= TAU;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::{
        ENEMY_RADIUS, Game, InputState, Obstacle, PLAYER_RADIUS, ShapeKind, Vec3,
        horizontal_distance,
    };

    #[test]
    fn projectile_hit_disables_enemy_and_awards_score() {
        let mut game = Game::new();
        game.projectiles.push(super::Projectile {
            position: game.enemy.position,
            velocity: Vec3::new(0.0, 0.0, 1.0),
            ttl: 1.0,
        });

        game.update(0.016, InputState::default());

        assert!(!game.enemy.alive);
        assert_eq!(game.score, 1000);
    }

    #[test]
    fn player_collision_rejects_obstacle_overlap() {
        let game = Game {
            obstacles: vec![Obstacle {
                position: Vec3::new(0.0, 0.0, 0.0),
                heading: 0.0,
                scale: Vec3::new(1.0, 1.0, 1.0),
                radius: 3.0,
                kind: ShapeKind::Cube,
            }],
            ..Game::new()
        };

        assert!(!game.position_is_walkable(Vec3::new(0.0, 0.0, 3.0)));
        assert!(
            horizontal_distance(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, 0.0))
                <= 3.0 + PLAYER_RADIUS
        );
    }

    #[test]
    fn enemy_hit_radius_matches_expectation() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 0.0, ENEMY_RADIUS);
        assert!((horizontal_distance(a, b) - ENEMY_RADIUS).abs() < 0.001);
    }
}
