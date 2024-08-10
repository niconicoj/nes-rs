use bevy::prelude::*;
use bevy_fundsp::prelude::*;
use hacker::resample;
use uuid::Uuid;

use super::{Apu, CPU_HZ};

#[derive(Component)]
struct NoiseMarker;

#[derive(Resource)]
struct NoiseVar {
    sample_rate: Shared,
    volume: Shared,
}

#[derive(Resource)]
struct NoiseId(Uuid);

struct NoiseDsp {
    uuid: Uuid,
    sample_rate: Shared,
    volume: Shared,
}

impl NoiseDsp {
    fn new(sample_rate: Shared, volume: Shared) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            sample_rate,
            volume,
        }
    }
}

impl DspGraph for NoiseDsp {
    fn id(&self) -> Uuid {
        self.uuid
    }

    fn generate_graph(&self) -> Box<dyn AudioUnit> {
        Box::new(noise() * var(&self.volume) >> split::<U2>() * 0.1)
    }
}

pub struct NoisePlugin;

impl Plugin for NoisePlugin {
    fn build(&self, app: &mut App) {
        let sample_rate = shared(1.0);
        let volume = shared(1.0);
        let noise_var = NoiseVar {
            sample_rate: sample_rate.clone(),
            volume: volume.clone(),
        };

        let noise_dsp = NoiseDsp::new(sample_rate.clone(), volume.clone());
        let noise_id = noise_dsp.id();

        app.add_dsp_source(noise_dsp, SourceType::Dynamic)
            .insert_resource(noise_var)
            .insert_resource(NoiseId(noise_id))
            .add_systems(PostStartup, setup_noise)
            .add_systems(FixedPostUpdate, update_noise);
    }
}

fn setup_noise(
    mut commands: Commands,
    mut assets: ResMut<Assets<DspSource>>,
    dsp_manager: Res<DspManager>,
    noise_id: Res<NoiseId>,
) {
    let source = assets.add(
        dsp_manager
            .get_graph_by_id(&noise_id.0)
            .expect("Noise DSP source not found"),
    );

    commands.spawn((
        AudioSourceBundle {
            source,
            settings: PlaybackSettings {
                paused: true,
                ..default()
            },
        },
        NoiseMarker,
    ));
}

// const SAMPLE_RATE: [f32; 4] = [44100.0, 48000.0, 88200.0, 96000.0];

fn update_noise(
    apu: Query<&Apu>,
    sink: Query<(&mut AudioSink, &NoiseMarker)>,
    noise_var: Res<NoiseVar>,
) {
    if let (Ok(apu), Ok((sink, _))) = (apu.get_single(), sink.get_single()) {
        let noise = &apu.noise;
        let volume = (noise.volume as f32) / 15.0;
        // noise_var.hz.set(hz);
        noise_var.volume.set(volume);
        let enabled = apu.status.noise();

        if !enabled || noise.length_counter == 0 {
            sink.pause();
        } else {
            sink.play();
        }
    }
}
