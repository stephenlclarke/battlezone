//! Generates lightweight synthesized sound effects and handles audio playback for game events.

use std::time::Duration;

use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player, Source, source::SineWave};

use crate::game::GameEvent;

struct AudioOutput {
    sink: MixerDeviceSink,
}

pub struct AudioManager {
    output: Option<AudioOutput>,
    title_drone: Option<Player>,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioOutput {
    fn new() -> Option<Self> {
        let mut sink = DeviceSinkBuilder::open_default_sink().ok()?;
        sink.log_on_drop(false);
        Some(Self { sink })
    }

    fn new_sink(&self) -> Player {
        Player::connect_new(self.sink.mixer())
    }
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            output: AudioOutput::new(),
            title_drone: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn silent() -> Self {
        Self {
            output: None,
            title_drone: None,
        }
    }

    pub fn handle_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::TitleScreenEntered => self.start_title_drone(),
            GameEvent::GameStarted => {
                self.stop_title_drone();
                self.play_effect(220.0, 150, 0.12);
                self.play_effect(330.0, 120, 0.09);
            }
            GameEvent::PlayerShot => self.play_effect(760.0, 90, 0.12),
            GameEvent::EnemyShot => self.play_effect(460.0, 110, 0.10),
            GameEvent::EnemyDestroyed => {
                self.play_effect(180.0, 140, 0.16);
                self.play_effect(130.0, 180, 0.10);
            }
            GameEvent::PlayerDestroyed => {
                self.play_effect(150.0, 180, 0.18);
                self.play_effect(90.0, 260, 0.14);
            }
            GameEvent::SaucerDestroyed => {
                self.play_effect(520.0, 90, 0.10);
                self.play_effect(660.0, 90, 0.08);
                self.play_effect(420.0, 160, 0.08);
            }
            GameEvent::RadarPing => self.play_effect(980.0, 50, 0.08),
        }
    }

    fn play_effect(&self, frequency_hz: f32, duration_ms: u64, amplitude: f32) {
        let Some(output) = self.output.as_ref() else {
            return;
        };
        let sink = output.new_sink();
        let source = SineWave::new(frequency_hz)
            .take_duration(Duration::from_millis(duration_ms))
            .amplify(amplitude);
        sink.append(source);
        sink.detach();
    }

    fn start_title_drone(&mut self) {
        self.stop_title_drone();
        let Some(output) = self.output.as_ref() else {
            return;
        };

        let sink = output.new_sink();
        let source = SineWave::new(82.5)
            .take_duration(Duration::from_millis(600))
            .amplify(0.025)
            .mix(
                SineWave::new(110.0)
                    .take_duration(Duration::from_millis(400))
                    .amplify(0.018),
            )
            .repeat_infinite();
        sink.append(source);
        self.title_drone = Some(sink);
    }

    fn stop_title_drone(&mut self) {
        if let Some(sink) = self.title_drone.take() {
            sink.stop();
        }
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        self.stop_title_drone();
    }
}

#[cfg(test)]
mod tests {
    use super::{AudioManager, AudioOutput};
    use crate::game::GameEvent;

    #[test]
    fn silent_audio_manager_handles_every_game_event() {
        let mut audio = AudioManager {
            output: None,
            title_drone: None,
        };

        for event in [
            GameEvent::TitleScreenEntered,
            GameEvent::GameStarted,
            GameEvent::PlayerShot,
            GameEvent::EnemyShot,
            GameEvent::EnemyDestroyed,
            GameEvent::PlayerDestroyed,
            GameEvent::SaucerDestroyed,
            GameEvent::RadarPing,
        ] {
            audio.handle_event(event);
        }

        assert!(audio.title_drone.is_none());
    }

    #[test]
    fn creating_audio_output_is_safe_even_without_a_device() {
        let _ = AudioOutput::new();
    }
}
