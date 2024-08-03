use bevy::prelude::*;
use bitfield::bitfield;
use pulse::PulsePlugin;

mod pulse;

const CPU_HZ: f32 = 1789773.0;

bitfield! {
    #[derive(Default)]
    struct Pulse(u32);
    impl Debug;
    volume, set_volume: 3, 0;
    constant_volume, set_constant_volume: 4, 4;
    envelope_loop, set_envelope_loop: 5, 5;
    duty, set_duty: 7, 6;
    shift, set_shift: 10, 8;
    negate, set_negate: 11, 11;
    period, set_period: 14, 12;
    enabled, set_enabled: 15, 15;
    timer, set_timer: 26, 16;
    length_counter, set_length_counter: 31, 27;
}

bitfield! {
    #[derive(Default)]
    struct Triangle(u32);
    impl Debug;
    linear_counter, set_linear_counter: 6, 0;
    linear_control, set_linear_control: 7, 7;
    timer, set_timer: 26, 16;
    length_counter, set_length_counter: 31, 27;
}

bitfield! {
    #[derive(Default)]
    struct Noise(u32);
    impl Debug;
    volume, set_volume: 3, 0;
    constant_volume, set_constant_volume: 4, 4;
    envelope_loop, set_envelope_loop: 5, 5;
    period, set_period: 19, 16;
    loop_noise, set_loop_noise: 23, 23;
    length_counter, set_length_counter: 31, 27;
}

bitfield! {
    #[derive(Default)]
    struct ApuStatus(u8);
    impl Debug;
    pulse, set_pulse: 1, 0;
    triangle, set_triangle: 2;
    noise, set_noise: 3;
}

bitfield! {
    #[derive(Default)]
    struct FrameCounter(u8);
    impl Debug;
    irq_inhibit, set_irq_inhibit: 6, 6;
    step_mode, set_step_mode: 7, 7;
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Step {
    One,
    Two,
    Three,
    Four,
    Five,
}

#[derive(Component, Default)]
pub struct Apu {
    pulse: [Pulse; 2],
    triangle: Triangle,
    noise: Noise,
    status: ApuStatus,
    frame_counter: FrameCounter,
    cycles: usize,
    step: Option<Step>,
}

impl Apu {
    pub fn tick(&mut self, cycles: usize) -> bool {
        if cycles % 6 == 0 {
            self.cycles += 1;
        }
        self.step = match (self.cycles, self.frame_counter.step_mode()) {
            (3729, _) => Some(Step::One),
            (7457, _) => Some(Step::Two),
            (11186, _) => Some(Step::Three),
            (14915, 0x00) => {
                self.cycles = 0;
                Some(Step::Four)
            }
            (14915, 0x01) => Some(Step::Four),
            (18641, 0x01) | (0, 0x01) => {
                self.cycles = 0;
                Some(Step::Five)
            }
            _ => None,
        };
        false
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4000..=0x4003 => {
                let offset = (addr & 0x03) * 8;
                let data = (data as u32) << offset;
                self.pulse[0].0 &= !(0xFF << offset);
                self.pulse[0].0 |= data;
            }
            0x4004..=0x4007 => {
                let offset = (addr & 0x03) * 8;
                let data = (data as u32) << offset;
                self.pulse[1].0 &= !(0xFF << offset);
                self.pulse[1].0 |= data;
            }
            0x4008..=0x400B => {}
            0x4015 => {
                self.status.0 = data;
            }
            0x4017 => {}
            _ => {}
        }
    }
}

pub struct ApuPlugin;

impl Plugin for ApuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PulsePlugin::<0>, PulsePlugin::<1>));
    }
}
