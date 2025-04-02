use std::{f32::consts::TAU, time::Duration};

use bevy::{
    audio::{AddAudioSource, Source},
    prelude::*,
};
use rand::{Rng, SeedableRng, rngs::StdRng};

pub fn plugin(app: &mut App) {
    app.add_audio_source::<SelectAudio>()
        .add_audio_source::<PlaceAudio>()
        .add_audio_source::<BeatLevelAudio>()
        .add_systems(Startup, setup);
}

#[derive(Resource)]
pub struct SelectAudioHandle(pub Handle<SelectAudio>);

#[derive(Resource)]
pub struct PlaceAudioHandle(pub Handle<PlaceAudio>);

#[derive(Resource)]
pub struct BeatLevelAudioHandle(pub Handle<BeatLevelAudio>);

fn setup(
    mut commands: Commands,
    mut select_audio: ResMut<Assets<SelectAudio>>,
    mut place_audio: ResMut<Assets<PlaceAudio>>,
    mut beat_level_audio: ResMut<Assets<BeatLevelAudio>>,
) {
    commands.insert_resource(SelectAudioHandle(select_audio.add(SelectAudio)));
    commands.insert_resource(PlaceAudioHandle(place_audio.add(PlaceAudio)));
    commands.insert_resource(BeatLevelAudioHandle(beat_level_audio.add(BeatLevelAudio)));
}

const SAMPLE_RATE: u32 = 44100;

#[derive(Asset, TypePath)]
pub struct SelectAudio;

impl Decodable for SelectAudio {
    type DecoderItem = f32;
    type Decoder = SelectDecoder;

    fn decoder(&self) -> Self::Decoder {
        SelectDecoder {
            total_secs: 0.25,
            progress: 0.0,
            rng: StdRng::seed_from_u64(0),
        }
    }
}

pub struct SelectDecoder {
    total_secs: f32,
    progress: f32,
    rng: StdRng,
}

impl Iterator for SelectDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.progress += 1.0 / self.total_secs / SAMPLE_RATE as f32;
        if self.progress <= 1.0 {
            let noise = self.rng.r#gen::<f32>() - 0.5;
            let volume = (0.2_f32.powf(self.progress) - 0.2) * 0.15;
            Some(noise * volume)
        } else {
            None
        }
    }
}

impl Source for SelectDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(self.total_secs))
    }
}

#[derive(Asset, TypePath)]
pub struct PlaceAudio;

impl Decodable for PlaceAudio {
    type DecoderItem = f32;
    type Decoder = PlaceDecoder;

    fn decoder(&self) -> Self::Decoder {
        PlaceDecoder {
            total_secs: 0.5,
            progress: 0.0,
            rng: StdRng::seed_from_u64(0),
        }
    }
}

pub struct PlaceDecoder {
    total_secs: f32,
    progress: f32,
    rng: StdRng,
}

impl Iterator for PlaceDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.progress += 1.0 / self.total_secs / SAMPLE_RATE as f32;
        if self.progress <= 1.0 {
            let noise = self.rng.r#gen::<f32>() - 0.5;
            let volume = 0.05_f32.powf(self.progress) * 0.2;
            Some(noise * volume)
        } else {
            None
        }
    }
}

impl Source for PlaceDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(self.total_secs))
    }
}

#[derive(Asset, TypePath)]
pub struct BeatLevelAudio;

impl Decodable for BeatLevelAudio {
    type DecoderItem = f32;
    type Decoder = BeatLevelDecoder;

    fn decoder(&self) -> Self::Decoder {
        BeatLevelDecoder {
            total_secs: 1.0,
            hz: vec![220.0, 440.0, 880.0],
            progress: 0.0,
        }
    }
}

pub struct BeatLevelDecoder {
    total_secs: f32,
    hz: Vec<f32>,
    progress: f32,
}

impl Iterator for BeatLevelDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.progress += 1.0 / self.total_secs / SAMPLE_RATE as f32;
        if self.progress <= 1.0 {
            let value = self
                .hz
                .iter()
                .map(|hz| (hz * self.progress * self.total_secs * TAU).sin())
                .sum::<f32>()
                / self.hz.len() as f32;
            let volume = (0.5_f32.powf(self.progress) - 0.5) * 2.0;
            Some(value * volume)
        } else {
            None
        }
    }
}

impl Source for BeatLevelDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(self.total_secs))
    }
}
