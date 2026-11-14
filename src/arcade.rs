//! Holds the extracted Battlezone arcade rules and battlefield tables used by the native Rust implementation.

use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail};

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
    try_arcade_tables().expect("arcade tables should load from embedded defaults")
}

pub fn try_arcade_tables() -> Result<&'static ArcadeTables> {
    static TABLES: OnceLock<Result<ArcadeTables, String>> = OnceLock::new();
    TABLES
        .get_or_init(|| load_arcade_tables().map_err(|error| format!("{error:#}")))
        .as_ref()
        .map_err(|message| anyhow!(message.clone()))
}

pub fn bonus_tank_label() -> String {
    let tables = arcade_tables();
    bonus_tank_label_for(tables)
}

pub fn missile_nastier_threshold() -> u32 {
    let tables = arcade_tables();
    missile_nastier_threshold_for(tables)
}

pub(crate) fn bonus_tank_label_for(tables: &ArcadeTables) -> String {
    format!(
        "BONUS TANK AT {} AND {}",
        tables.bonus_tank_thresholds[0], tables.bonus_tank_thresholds[1]
    )
}

pub(crate) fn missile_nastier_threshold_for(tables: &ArcadeTables) -> u32 {
    tables.missile_score_threshold + tables.missile_nastier_delta
}

fn load_arcade_tables() -> Result<ArcadeTables> {
    let rules = customization::load_arcade_text("arcade-rules.txt", ARCADE_RULES)
        .context("loading arcade rules")?;
    let battlefield = customization::load_arcade_text("battlefield.txt", BATTLEFIELD_LAYOUT)
        .context("loading battlefield layout")?;
    parse_arcade_tables(&rules, &battlefield)
}

fn parse_arcade_tables(rules: &str, battlefield: &str) -> Result<ArcadeTables> {
    let mut starting_lives = None;
    let mut missile_score_threshold = None;
    let mut missile_nastier_delta = None;
    let mut bonus_tank_thresholds = None;
    let mut saucer_score_threshold = None;
    let mut near_spawn_distance = None;
    let mut far_spawn_distance = None;
    let mut strings: Option<Vec<String>> = None;

    for (line_number, line) in rules.lines().map(str::trim).enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| anyhow!("arcade rule line {} should use key=value", line_number + 1))?;
        match key {
            "starting_lives" => {
                starting_lives = Some(parse_u32(value, key, line_number + 1)?);
            }
            "missile_score_threshold" => {
                missile_score_threshold = Some(parse_u32(value, key, line_number + 1)?);
            }
            "missile_nastier_delta" => {
                missile_nastier_delta = Some(parse_u32(value, key, line_number + 1)?);
            }
            "bonus_tank_thresholds" => {
                bonus_tank_thresholds = Some(parse_two_u32(value, key, line_number + 1)?);
            }
            "saucer_score_threshold" => {
                saucer_score_threshold = Some(parse_u32(value, key, line_number + 1)?);
            }
            "near_spawn_distance" => {
                near_spawn_distance = Some(parse_f32(value, key, line_number + 1)?);
            }
            "far_spawn_distance" => {
                far_spawn_distance = Some(parse_f32(value, key, line_number + 1)?);
            }
            "strings" => strings = Some(value.split('|').map(ToString::to_string).collect()),
            _ => bail!("unknown arcade rule key {key} on line {}", line_number + 1),
        }
    }

    let strings = strings.ok_or_else(|| anyhow!("strings should be defined"))?;
    if strings.len() < 11 {
        bail!("strings should define at least 11 arcade labels");
    }

    Ok(ArcadeTables {
        starting_lives: starting_lives
            .ok_or_else(|| anyhow!("starting_lives should be defined"))?,
        missile_score_threshold: missile_score_threshold
            .ok_or_else(|| anyhow!("missile_score_threshold should be defined"))?,
        missile_nastier_delta: missile_nastier_delta
            .ok_or_else(|| anyhow!("missile_nastier_delta should be defined"))?,
        bonus_tank_thresholds: bonus_tank_thresholds
            .ok_or_else(|| anyhow!("bonus_tank_thresholds should be defined"))?,
        saucer_score_threshold: saucer_score_threshold
            .ok_or_else(|| anyhow!("saucer_score_threshold should be defined"))?,
        near_spawn_distance: near_spawn_distance
            .ok_or_else(|| anyhow!("near_spawn_distance should be defined"))?,
        far_spawn_distance: far_spawn_distance
            .ok_or_else(|| anyhow!("far_spawn_distance should be defined"))?,
        strings,
        obstacles: parse_battlefield(battlefield)?,
    })
}

fn parse_battlefield(text: &str) -> Result<Vec<ObstacleSpec>> {
    let mut obstacles = Vec::new();
    for (line_number, line) in text.lines().map(str::trim).enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.len() != 5 {
            bail!(
                "battlefield line {} should be: kind x z heading_deg radius",
                line_number + 1
            );
        }
        obstacles.push(ObstacleSpec {
            kind: parse_obstacle_kind(parts[0], line_number + 1)?,
            x: parse_f32(parts[1], "x", line_number + 1)?,
            z: parse_f32(parts[2], "z", line_number + 1)?,
            heading: parse_f32(parts[3], "heading_deg", line_number + 1)?.to_radians(),
            radius: parse_f32(parts[4], "radius", line_number + 1)?,
        });
    }

    if obstacles.len() != 21 {
        bail!("battlefield should contain 21 obstacles");
    }
    Ok(obstacles)
}

fn parse_obstacle_kind(value: &str, line_number: usize) -> Result<ObstacleKind> {
    Ok(match value {
        "narrow_pyramid" => ObstacleKind::NarrowPyramid,
        "tall_box" => ObstacleKind::TallBox,
        "wide_pyramid" => ObstacleKind::WidePyramid,
        "short_box" => ObstacleKind::ShortBox,
        _ => bail!("unknown obstacle kind {value} on battlefield line {line_number}"),
    })
}

fn parse_u32(value: &str, key: &str, line_number: usize) -> Result<u32> {
    value
        .parse()
        .with_context(|| format!("parsing integer arcade value {key} on line {line_number}"))
}

fn parse_f32(value: &str, key: &str, line_number: usize) -> Result<f32> {
    value
        .parse()
        .with_context(|| format!("parsing floating-point arcade value {key} on line {line_number}"))
}

fn parse_two_u32(value: &str, key: &str, line_number: usize) -> Result<[u32; 2]> {
    let values = value
        .split(',')
        .map(|part| parse_u32(part, key, line_number))
        .collect::<Result<Vec<_>>>()?;
    let [first, second] = values.try_into().map_err(|_| {
        anyhow!("{key} on line {line_number} should contain exactly two integer values")
    })?;
    Ok([first, second])
}

#[cfg(test)]
mod tests {
    use super::{arcade_tables, bonus_tank_label, missile_nastier_threshold, parse_arcade_tables};

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

    #[test]
    fn malformed_arcade_rules_return_errors() {
        let error = parse_arcade_tables("starting_lives 3\n", "").expect_err("rules should fail");

        assert!(error.to_string().contains("key=value"));
    }
}
