//! Holds the extracted Battlezone arcade rules and battlefield tables used by the native Rust implementation.

use std::sync::OnceLock;

use crate::customization;

pub const ORIGINAL_FPS: f32 = 41.666_668;
pub const ORIGINAL_FRAME_TIME: f32 = 1.0 / ORIGINAL_FPS;

const ARCADE_RULES: &str = include_str!("../assets/arcade/arcade-rules.txt");
const BATTLEFIELD_LAYOUT: &str = include_str!("../assets/arcade/battlefield.txt");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObstacleKind {
    NarrowPyramid,
    TallBox,
    WidePyramid,
    ShortBox,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObstacleSpec {
    pub kind: ObstacleKind,
    pub x: f32,
    pub z: f32,
    pub heading: f32,
    pub radius: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArcadeTables {
    pub starting_lives: u32,
    pub missile_score_threshold: u32,
    pub missile_nastier_delta: u32,
    pub bonus_tank_thresholds: [u32; 2],
    pub saucer_score_threshold: u32,
    pub near_spawn_distance: f32,
    pub far_spawn_distance: f32,
    pub strings: Vec<String>,
    pub obstacles: Vec<ObstacleSpec>,
}

pub fn arcade_tables() -> &'static ArcadeTables {
    static TABLES: OnceLock<ArcadeTables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let rules = customization::load_arcade_text("arcade-rules.txt", ARCADE_RULES);
        let battlefield = customization::load_arcade_text("battlefield.txt", BATTLEFIELD_LAYOUT);
        parse_arcade_tables(&rules, &battlefield)
    })
}

pub fn bonus_tank_label() -> String {
    let tables = arcade_tables();
    format!(
        "BONUS TANK AT {} AND {}",
        tables.bonus_tank_thresholds[0], tables.bonus_tank_thresholds[1]
    )
}

pub fn missile_nastier_threshold() -> u32 {
    let tables = arcade_tables();
    tables.missile_score_threshold + tables.missile_nastier_delta
}

fn parse_arcade_tables(rules: &str, battlefield: &str) -> ArcadeTables {
    let mut starting_lives = None;
    let mut missile_score_threshold = None;
    let mut missile_nastier_delta = None;
    let mut bonus_tank_thresholds = None;
    let mut saucer_score_threshold = None;
    let mut near_spawn_distance = None;
    let mut far_spawn_distance = None;
    let mut strings = None;

    for line in rules.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let (key, value) = line
            .split_once('=')
            .expect("arcade rules should use key=value lines");
        match key {
            "starting_lives" => starting_lives = Some(parse_u32(value)),
            "missile_score_threshold" => missile_score_threshold = Some(parse_u32(value)),
            "missile_nastier_delta" => missile_nastier_delta = Some(parse_u32(value)),
            "bonus_tank_thresholds" => bonus_tank_thresholds = Some(parse_two_u32(value)),
            "saucer_score_threshold" => saucer_score_threshold = Some(parse_u32(value)),
            "near_spawn_distance" => near_spawn_distance = Some(parse_f32(value)),
            "far_spawn_distance" => far_spawn_distance = Some(parse_f32(value)),
            "strings" => strings = Some(value.split('|').map(ToString::to_string).collect()),
            _ => panic!("unknown arcade rule key {key}"),
        }
    }

    ArcadeTables {
        starting_lives: starting_lives.expect("starting_lives should be defined"),
        missile_score_threshold: missile_score_threshold
            .expect("missile_score_threshold should be defined"),
        missile_nastier_delta: missile_nastier_delta
            .expect("missile_nastier_delta should be defined"),
        bonus_tank_thresholds: bonus_tank_thresholds
            .expect("bonus_tank_thresholds should be defined"),
        saucer_score_threshold: saucer_score_threshold
            .expect("saucer_score_threshold should be defined"),
        near_spawn_distance: near_spawn_distance.expect("near_spawn_distance should be defined"),
        far_spawn_distance: far_spawn_distance.expect("far_spawn_distance should be defined"),
        strings: strings.expect("strings should be defined"),
        obstacles: parse_battlefield(battlefield),
    }
}

fn parse_battlefield(text: &str) -> Vec<ObstacleSpec> {
    let mut obstacles = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts = line.split_whitespace().collect::<Vec<_>>();
        assert_eq!(
            parts.len(),
            5,
            "battlefield rows should be: kind x z heading_deg radius"
        );
        obstacles.push(ObstacleSpec {
            kind: parse_obstacle_kind(parts[0]),
            x: parse_f32(parts[1]),
            z: parse_f32(parts[2]),
            heading: parse_f32(parts[3]).to_radians(),
            radius: parse_f32(parts[4]),
        });
    }

    assert_eq!(
        obstacles.len(),
        21,
        "battlefield should contain 21 obstacles"
    );
    obstacles
}

fn parse_obstacle_kind(value: &str) -> ObstacleKind {
    match value {
        "narrow_pyramid" => ObstacleKind::NarrowPyramid,
        "tall_box" => ObstacleKind::TallBox,
        "wide_pyramid" => ObstacleKind::WidePyramid,
        "short_box" => ObstacleKind::ShortBox,
        _ => panic!("unknown obstacle kind {value}"),
    }
}

fn parse_u32(value: &str) -> u32 {
    value.parse().expect("expected integer arcade value")
}

fn parse_f32(value: &str) -> f32 {
    value.parse().expect("expected floating-point arcade value")
}

fn parse_two_u32(value: &str) -> [u32; 2] {
    let values = value.split(',').map(parse_u32).collect::<Vec<_>>();
    [values[0], values[1]]
}

#[cfg(test)]
mod tests {
    use super::{arcade_tables, bonus_tank_label, missile_nastier_threshold};

    #[test]
    fn obstacle_tables_match_expected_layout() {
        let tables = arcade_tables();
        assert_eq!(tables.obstacles.len(), 21);
        assert!((tables.obstacles[0].x - 32.0).abs() < 0.01);
        assert!((tables.obstacles[0].z - 32.0).abs() < 0.01);
        assert!((tables.obstacles[20].x + 12.0).abs() < 0.01);
        assert!((tables.obstacles[20].z - 44.0).abs() < 0.01);
    }

    #[test]
    fn spawn_distances_match_arcade_values() {
        let tables = arcade_tables();
        assert!((tables.near_spawn_distance - 47.996_094).abs() < 0.001);
        assert!((tables.far_spawn_distance - 95.996_09).abs() < 0.001);
    }

    #[test]
    fn default_labels_match_arcade_defaults() {
        assert_eq!(bonus_tank_label(), "BONUS TANK AT 15000 AND 100000");
        assert_eq!(missile_nastier_threshold(), 35_000);
    }
}
