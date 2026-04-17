//! Holds ROM-derived title-screen layout and logo data used by the native Battlezone attract screens.

use crate::math::Vec3;

pub const ARCADE_SCREEN_WIDTH: i32 = 1024;
pub const ARCADE_SCREEN_HEIGHT: i32 = 768;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextSize {
    Full,
    Half,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenLabel {
    pub position: (i16, i16),
    pub text: &'static str,
    pub size: TextSize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LogoMesh {
    pub vertices: &'static [Vec3],
    pub edges: &'static [(usize, usize)],
}

pub const SCORE_LABEL: ScreenLabel = ScreenLabel {
    position: (128, 320),
    text: "SCORE     000",
    size: TextSize::Full,
};

pub const HIGH_SCORE_LABEL: ScreenLabel = ScreenLabel {
    position: (128, 280),
    text: "HIGH SCORE      000",
    size: TextSize::Half,
};

pub const PRESS_START_LABEL: ScreenLabel = ScreenLabel {
    position: (-136, 0),
    text: "PRESS START",
    size: TextSize::Full,
};

pub const HIGH_SCORES_LABEL: ScreenLabel = ScreenLabel {
    position: (-112, 160),
    text: "HIGH SCORES",
    size: TextSize::Full,
};

pub const GREAT_SCORE_LABEL: ScreenLabel = ScreenLabel {
    position: (-256, 96),
    text: "GREAT SCORE",
    size: TextSize::Full,
};

pub const ENTER_INITIALS_LABEL: ScreenLabel = ScreenLabel {
    position: (28, 104),
    text: "ENTER YOUR INITIALS",
    size: TextSize::Half,
};

pub const CHANGE_LETTER_LABEL: ScreenLabel = ScreenLabel {
    position: (-240, 64),
    text: "CHANGE LETTER WITH RIGHT HAND CONTROLLER",
    size: TextSize::Half,
};

pub const SELECT_LETTER_LABEL: ScreenLabel = ScreenLabel {
    position: (-180, 32),
    text: "SELECT LETTER WITH FIRE BUTTON",
    size: TextSize::Half,
};

pub const INSERT_COIN_LABEL: ScreenLabel = ScreenLabel {
    position: (-144, -88),
    text: "INSERT COIN",
    size: TextSize::Full,
};

pub const BONUS_TANK_LABEL: ScreenLabel = ScreenLabel {
    position: (-348, -280),
    text: "BONUS TANK AT ",
    size: TextSize::Full,
};

pub const COPYRIGHT_LABEL: ScreenLabel = ScreenLabel {
    position: (-168, -240),
    text: "(C)(P)  ATARI 1980",
    size: TextSize::Full,
};

pub const HIGH_SCORE_LIST_START: (i16, i16) = (-128, 104);
pub const HIGH_SCORE_ROW_DELTA: (i16, i16) = (0, -40);
pub const TANK_ICON_SCORE_THRESHOLD: u32 = 100_000;

pub const TITLE_LOGO_MESHES: [LogoMesh; 3] = [
    LogoMesh {
        vertices: &BA_VERTICES,
        edges: &BA_EDGES,
    },
    LogoMesh {
        vertices: &TTLE_VERTICES,
        edges: &TTLE_EDGES,
    },
    LogoMesh {
        vertices: &ZONE_VERTICES,
        edges: &ZONE_EDGES,
    },
];

const BA_VERTICES: [Vec3; 20] = [
    Vec3::new(20.0, 0.125, 0.875),
    Vec3::new(15.0, 0.125, 0.875),
    Vec3::new(12.5, 0.344, 2.625),
    Vec3::new(13.75, 0.562, 4.375),
    Vec3::new(12.5, 0.781, 6.25),
    Vec3::new(15.0, 1.0, 8.0),
    Vec3::new(20.0, 1.0, 8.0),
    Vec3::new(17.5, 0.344, 2.625),
    Vec3::new(16.25, 0.344, 2.625),
    Vec3::new(17.5, 0.562, 4.375),
    Vec3::new(16.25, 0.781, 6.25),
    Vec3::new(17.5, 0.781, 6.25),
    Vec3::new(12.5, 0.125, 0.875),
    Vec3::new(8.75, 0.344, 0.125),
    Vec3::new(5.0, 0.125, 0.875),
    Vec3::new(8.75, 1.0, 8.0),
    Vec3::new(10.0, 0.438, 3.5),
    Vec3::new(8.75, 0.5, 4.0),
    Vec3::new(7.5, 0.438, 3.5),
    Vec3::new(8.75, 0.656, 5.25),
];

const BA_EDGES: [(usize, usize); 20] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 4),
    (4, 5),
    (5, 6),
    (6, 0),
    (7, 8),
    (8, 9),
    (9, 10),
    (10, 11),
    (11, 7),
    (12, 13),
    (13, 14),
    (14, 15),
    (15, 12),
    (16, 17),
    (17, 18),
    (18, 19),
    (19, 16),
];

const TTLE_VERTICES: [Vec3; 21] = [
    Vec3::new(2.5, 0.125, 0.875),
    Vec3::new(1.25, 0.781, 6.25),
    Vec3::new(-2.5, 0.781, 6.25),
    Vec3::new(-3.75, 0.125, 0.875),
    Vec3::new(-5.0, 0.781, 6.25),
    Vec3::new(-8.75, 0.781, 6.25),
    Vec3::new(-8.75, 0.125, 0.875),
    Vec3::new(-15.0, 0.125, 0.875),
    Vec3::new(-21.25, 0.219, 1.75),
    Vec3::new(-17.5, 0.344, 2.625),
    Vec3::new(-17.5, 0.438, 3.5),
    Vec3::new(-20.0, 0.562, 4.375),
    Vec3::new(-17.5, 0.656, 5.25),
    Vec3::new(-17.5, 0.781, 6.25),
    Vec3::new(-21.25, 0.875, 7.125),
    Vec3::new(-15.0, 1.0, 8.0),
    Vec3::new(-11.25, 0.344, 2.625),
    Vec3::new(-11.25, 1.0, 8.0),
    Vec3::new(7.5, 1.0, 8.0),
    Vec3::new(7.5, 0.781, 6.25),
    Vec3::new(3.75, 0.781, 6.25),
];

const TTLE_EDGES: [(usize, usize); 22] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 4),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 8),
    (8, 9),
    (9, 10),
    (10, 11),
    (11, 12),
    (12, 13),
    (13, 14),
    (14, 15),
    (15, 7),
    (7, 16),
    (16, 17),
    (17, 18),
    (18, 19),
    (19, 20),
    (20, 0),
];

const ZONE_VERTICES: [Vec3; 25] = [
    Vec3::new(18.75, -1.0, -8.0),
    Vec3::new(8.75, -1.0, -8.0),
    Vec3::new(13.75, -0.781, -6.25),
    Vec3::new(8.75, -0.125, -0.875),
    Vec3::new(18.75, -0.125, -0.875),
    Vec3::new(13.75, -0.344, -2.625),
    Vec3::new(1.25, -1.0, -8.0),
    Vec3::new(1.25, -0.125, -0.875),
    Vec3::new(6.25, -0.781, -6.25),
    Vec3::new(3.75, -0.781, -6.25),
    Vec3::new(3.75, -0.344, -2.625),
    Vec3::new(6.25, -0.344, -2.625),
    Vec3::new(0.0, -1.0, -8.0),
    Vec3::new(-2.5, -0.562, -4.375),
    Vec3::new(-10.0, -1.0, -8.0),
    Vec3::new(-16.25, -0.875, -7.125),
    Vec3::new(-12.5, -0.781, -6.25),
    Vec3::new(-12.5, -0.656, -5.25),
    Vec3::new(-15.0, -0.562, -4.375),
    Vec3::new(-12.5, -0.438, -3.5),
    Vec3::new(-12.5, -0.344, -2.625),
    Vec3::new(-16.25, -0.219, -1.75),
    Vec3::new(-10.0, -0.125, -0.875),
    Vec3::new(-7.5, -0.562, -4.375),
    Vec3::new(0.0, -0.125, -0.875),
];

const ZONE_EDGES: [(usize, usize); 28] = [
    (1, 0),
    (0, 5),
    (5, 4),
    (4, 3),
    (3, 2),
    (2, 1),
    (1, 3),
    (3, 7),
    (7, 6),
    (6, 1),
    (9, 8),
    (8, 11),
    (11, 10),
    (10, 9),
    (14, 22),
    (22, 23),
    (23, 24),
    (24, 12),
    (12, 13),
    (13, 14),
    (14, 15),
    (15, 16),
    (16, 17),
    (17, 18),
    (18, 19),
    (19, 20),
    (20, 21),
    (21, 22),
];

#[cfg(test)]
mod tests {
    use super::{
        ARCADE_SCREEN_HEIGHT, ARCADE_SCREEN_WIDTH, HIGH_SCORE_LIST_START, PRESS_START_LABEL,
        TITLE_LOGO_MESHES,
    };

    #[test]
    fn arcade_screen_bounds_match_vector_monitor_coordinates() {
        assert_eq!(ARCADE_SCREEN_WIDTH, 1024);
        assert_eq!(ARCADE_SCREEN_HEIGHT, 768);
        assert_eq!(PRESS_START_LABEL.position, (-136, 0));
        assert_eq!(HIGH_SCORE_LIST_START, (-128, 104));
    }

    #[test]
    fn title_logo_meshes_match_rom_extract_sizes() {
        assert_eq!(TITLE_LOGO_MESHES[0].vertices.len(), 20);
        assert_eq!(TITLE_LOGO_MESHES[0].edges.len(), 20);
        assert_eq!(TITLE_LOGO_MESHES[1].vertices.len(), 21);
        assert_eq!(TITLE_LOGO_MESHES[1].edges.len(), 22);
        assert_eq!(TITLE_LOGO_MESHES[2].vertices.len(), 25);
        assert_eq!(TITLE_LOGO_MESHES[2].edges.len(), 28);
    }
}
