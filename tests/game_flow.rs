use battlezone::{
    arcade::ORIGINAL_FRAME_TIME,
    game::{Game, GameEvent},
    input::UpdateInput,
    render::Scene,
};

fn overlay_contains(scene: &Scene, needle: &str) -> bool {
    scene
        .overlay_text
        .iter()
        .any(|text| text.text.contains(needle))
}

fn advance_frames(game: &mut Game, frames: usize) {
    for _ in 0..frames {
        game.update_with_input(ORIGINAL_FRAME_TIME, UpdateInput::default());
    }
}

#[test]
fn attract_mode_cycles_from_title_to_high_scores() {
    let mut game = Game::with_seed(1);
    game.set_viewport(640, 360);

    let title_scene = game.frame();
    assert!(overlay_contains(&title_scene, "PRESS"));
    assert!(!title_scene.world_lines.is_empty());

    advance_frames(&mut game, 300);

    let high_score_scene = game.frame();
    assert!(overlay_contains(&high_score_scene, "HIGH SCORES"));
    assert!(high_score_scene.world_lines.is_empty());
}

#[test]
fn starting_game_reaches_play_hud_after_respawn() {
    let mut game = Game::with_seed(2);
    game.set_viewport(640, 360);
    game.update_with_input(
        ORIGINAL_FRAME_TIME,
        UpdateInput {
            start_requested: true,
            ..UpdateInput::default()
        },
    );

    advance_frames(&mut game, 90);

    let scene = game.frame();
    assert!(scene.show_crosshair);
    assert!(overlay_contains(&scene, "SCORE"));
    assert!(overlay_contains(&scene, "TANKS"));
    assert!(!scene.world_lines.is_empty());
}

#[test]
fn xyzzy_mode_appears_in_overlay_when_activated() {
    let mut game = Game::with_seed(3);
    game.set_viewport(640, 360);
    game.update_with_input(
        ORIGINAL_FRAME_TIME,
        UpdateInput {
            start_requested: true,
            ..UpdateInput::default()
        },
    );

    game.update_with_input(
        ORIGINAL_FRAME_TIME,
        UpdateInput {
            typed_chars: vec!['x', 'y', 'z', 'z', 'y'],
            ..UpdateInput::default()
        },
    );
    game.update_with_input(
        ORIGINAL_FRAME_TIME,
        UpdateInput {
            typed_chars: vec!['f'],
            autopilot_toggle_requested: true,
            ..UpdateInput::default()
        },
    );

    let scene = game.frame();
    assert!(overlay_contains(&scene, "XYZZY MODE"));
    assert!(overlay_contains(&scene, "FIRE RATE 2"));
    assert!(overlay_contains(&scene, "AUTO ON"));
}

#[test]
fn firing_a_shot_emits_event_and_changes_scene_geometry() {
    let mut game = Game::with_seed(4);
    game.set_viewport(640, 360);
    game.update_with_input(
        ORIGINAL_FRAME_TIME,
        UpdateInput {
            start_requested: true,
            ..UpdateInput::default()
        },
    );
    advance_frames(&mut game, 90);
    let _ = game.drain_events();

    let before = game.frame().world_lines.len();
    game.update_with_input(
        ORIGINAL_FRAME_TIME,
        UpdateInput {
            fire: true,
            ..UpdateInput::default()
        },
    );

    let events = game.drain_events();
    assert!(events.contains(&GameEvent::PlayerShot));

    let after = game.frame().world_lines.len();
    assert!(after > before);
}
