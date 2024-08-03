use bevy::prelude::*;
use bevy_fundsp::prelude::*;
use uuid::Uuid;

use super::{Apu, CPU_HZ};

#[derive(Component)]
struct PulseMarker<const ID: usize>;

#[derive(Resource)]
struct PulseVar<const ID: usize> {
    hz: Shared,
    duty: Shared,
}

#[derive(Resource)]
struct PulseId<const ID: usize>(Uuid);

struct PulseDsp {
    uuid: Uuid,
    hz: Shared,
    duty: Shared,
}

impl PulseDsp {
    fn new(hz: Shared, duty: Shared) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            hz,
            duty,
        }
    }
}

impl DspGraph for PulseDsp {
    fn id(&self) -> Uuid {
        self.uuid
    }

    fn generate_graph(&self) -> Box<dyn AudioUnit> {
        Box::new((var(&self.hz) | var(&self.duty)) >> pulse() >> split::<U2>() * 0.3)
    }
}

pub struct PulsePlugin<const ID: usize>;

impl<const ID: usize> Plugin for PulsePlugin<ID> {
    fn build(&self, app: &mut App) {
        let hz = shared(440.0);
        let duty = shared(0.5);
        let pulse_var = PulseVar::<ID> {
            hz: hz.clone(),
            duty: duty.clone(),
        };

        let pulse_dsp = PulseDsp::new(hz.clone(), duty.clone());
        let pulse_id = pulse_dsp.id();

        app.add_dsp_source(pulse_dsp, SourceType::Dynamic)
            .insert_resource(pulse_var)
            .insert_resource(PulseId::<ID>(pulse_id))
            .add_systems(PostStartup, setup_pulse::<ID>)
            .add_systems(FixedPostUpdate, update_pulse::<ID>);
    }
}

fn setup_pulse<const ID: usize>(
    mut commands: Commands,
    mut assets: ResMut<Assets<DspSource>>,
    dsp_manager: Res<DspManager>,
    pulse_id: Res<PulseId<ID>>,
) {
    let source = assets.add(
        dsp_manager
            .get_graph_by_id(&pulse_id.0)
            .expect("Pulse DSP source not found"),
    );

    commands.spawn((
        AudioSourceBundle {
            source,
            settings: PlaybackSettings {
                paused: true,
                ..default()
            },
        },
        PulseMarker::<ID>,
    ));
}

fn update_pulse<const ID: usize>(
    apu: Query<&Apu>,
    sink: Query<(&mut AudioSink, &PulseMarker<ID>)>,
    pulse_var: Res<PulseVar<ID>>,
) {
    if let (Ok(apu), Ok((sink, _))) = (apu.get_single(), sink.get_single()) {
        let pulse_register = &apu.pulse[ID];
        let hz = CPU_HZ / ((16 * (pulse_register.timer() + 1)) as f32);
        let period = CPU_HZ / (16.0 * hz) - 1.0;
        let duty = match pulse_register.duty() {
            0 => 0.125,
            1 => 0.25,
            2 => 0.5,
            3 => 0.75,
            _ => unreachable!(),
        };
        pulse_var.hz.set(hz);
        pulse_var.duty.set(duty);
        let enabled = apu.status.pulse() & (1 << ID) != 0;

        debug!(
            "Pulse {}: enabled={}, hz={}, duty={}, period={}",
            ID, enabled, hz, duty, period
        );
        if !enabled || period < 8.0 {
            sink.pause();
        } else {
            sink.play();
        }
    }
}
