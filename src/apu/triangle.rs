use bevy::prelude::*;
use bevy_fundsp::prelude::*;
use uuid::Uuid;

use super::{Apu, CPU_HZ};

#[derive(Component)]
struct TriangleMarker;

#[derive(Resource)]
struct TriangleVar {
    hz: Shared,
}

#[derive(Resource)]
struct TriangleId(Uuid);

struct TriangleDsp {
    uuid: Uuid,
    hz: Shared,
}

impl TriangleDsp {
    fn new(hz: Shared) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            hz,
        }
    }
}

impl DspGraph for TriangleDsp {
    fn id(&self) -> Uuid {
        self.uuid
    }

    fn generate_graph(&self) -> Box<dyn AudioUnit> {
        Box::new(var(&self.hz) >> triangle() >> split::<U2>() * 0.4)
    }
}

pub struct TrianglePlugin;

impl Plugin for TrianglePlugin {
    fn build(&self, app: &mut App) {
        let hz = shared(440.0);
        let pulse_var = TriangleVar { hz: hz.clone() };

        let pulse_dsp = TriangleDsp::new(hz.clone());
        let pulse_id = pulse_dsp.id();

        app.add_dsp_source(pulse_dsp, SourceType::Dynamic)
            .insert_resource(pulse_var)
            .insert_resource(TriangleId(pulse_id))
            .add_systems(PostStartup, setup_triangle)
            .add_systems(FixedPostUpdate, update_triangle);
    }
}

fn setup_triangle(
    mut commands: Commands,
    mut assets: ResMut<Assets<DspSource>>,
    dsp_manager: Res<DspManager>,
    triangle_id: Res<TriangleId>,
) {
    let source = assets.add(
        dsp_manager
            .get_graph_by_id(&triangle_id.0)
            .expect("Triangle DSP source not found"),
    );

    commands.spawn((
        AudioSourceBundle {
            source,
            settings: PlaybackSettings {
                paused: true,
                ..default()
            },
        },
        TriangleMarker,
    ));
}

fn update_triangle(
    apu: Query<&Apu>,
    sink: Query<(&mut AudioSink, &TriangleMarker)>,
    triangle_var: Res<TriangleVar>,
) {
    if let (Ok(apu), Ok((sink, _))) = (apu.get_single(), sink.get_single()) {
        let triangle = &apu.triangle;
        let hz = CPU_HZ / (32.0 * ((triangle.reg.timer() as f32) + 1.0));
        triangle_var.hz.set(hz);
        let enabled = apu.status.triangle();

        if !enabled || triangle.length_counter == 0 || triangle.linear_counter == 0 {
            sink.pause();
        } else {
            sink.play();
        }
    }
}
