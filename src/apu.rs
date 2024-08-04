use bevy::prelude::*;
use bitfield::bitfield;
use pulse::PulsePlugin;

mod pulse;

const CPU_HZ: f32 = 1789773.0;

const LENGTH_COUNTER_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

bitfield! {
    #[derive(Default)]
    struct PulseRegister(u32);
    impl Debug;
    volume, set_volume: 3, 0;
    envelope, set_envelope: 4, 4;
    constant_volume, set_constant_volume: 4, 4;
    length_counter_halt, set_length_counter_halt: 5, 5;
    envelope_loop, set_envelope_loop: 5, 5;
    duty, set_duty: 7, 6;
    shift, set_shift: 10, 8;
    negate, set_negate: 11, 11;
    sweep_period, set_sweep_period: 14, 12;
    enabled, set_enabled: 15, 15;
    timer, set_timer: 26, 16;
    length_counter, set_length_counter: 31, 27;
}

#[derive(Default)]
struct Pulse {
    reg: PulseRegister,
    length_counter: u8,
    start_flag: bool,
    volume: u8,
    decay_level: u8,
    divider_counter: u8,
    divider_period: u8,
}

impl Pulse {
    fn clock_length_counter(&mut self) {
        if self.reg.length_counter_halt() == 0 {
            let next_length = self.length_counter.saturating_sub(1);
            self.length_counter = next_length;
        }
    }

    fn clock_envelope(&mut self) {
        if self.start_flag {
            self.decay_level = 15;
            self.divider_counter = 15;
            self.divider_period = self.reg.volume() as u8;
        } else {
            self.divider_counter = self
                .divider_counter
                .saturating_sub(15 - self.divider_period);
            if self.divider_counter != 0 {
                if self.decay_level != 0 {
                    self.decay_level = self.decay_level.saturating_sub(1);
                } else {
                    self.decay_level = (self.reg.envelope_loop() as u8) * 15;
                }
            }
        }
        if self.reg.constant_volume() != 0 {
            self.volume = self.reg.volume() as u8;
        } else {
            self.volume = self.decay_level;
        }
    }
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
}

impl Apu {
    pub fn tick(&mut self, cycles: usize) -> bool {
        if cycles % 6 == 0 {
            self.cycles += 1;
            match (self.cycles, self.frame_counter.step_mode()) {
                (3729, _) => self.quarter_frame_tick(),
                (7457, _) => {
                    self.half_frame_tick();
                    self.quarter_frame_tick();
                }
                (11186, _) => {
                    self.quarter_frame_tick();
                }
                (14915, 0x00) => {
                    self.half_frame_tick();
                    self.quarter_frame_tick();
                    self.cycles = 0;
                }
                (18641, 0x01) => {
                    self.half_frame_tick();
                    self.quarter_frame_tick();
                    self.cycles = 0;
                }
                _ => {}
            };
        }

        false
    }

    pub fn quarter_frame_tick(&mut self) {
        for pulse_id in 0..=1 {
            if (self.status.pulse() >> pulse_id) & 1 != 0 {
                self.pulse[pulse_id].clock_envelope();
            }
        }
    }

    pub fn half_frame_tick(&mut self) {
        for pulse_id in 0..=1 {
            if (self.status.pulse() >> pulse_id) & 1 != 0 {
                self.pulse[pulse_id].clock_length_counter();
            }
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4000 | 0x4004 => {
                let pulse_id = ((addr >> 2) & 1) as usize;
                let offset = (addr & 0x03) * 8;
                let data = (data as u32) << offset;
                self.pulse[pulse_id].reg.0 &= !(0xFF << offset);
                self.pulse[pulse_id].reg.0 |= data;

                if (self.status.pulse() >> pulse_id) & 1 != 0 {
                    self.pulse[pulse_id].start_flag = true;
                    self.pulse[pulse_id].length_counter =
                        LENGTH_COUNTER_TABLE[self.pulse[pulse_id].reg.length_counter() as usize];
                }
            }
            0x4001..=0x4003 => {
                let offset = (addr & 0x03) * 8;
                let data = (data as u32) << offset;
                self.pulse[0].reg.0 &= !(0xFF << offset);
                self.pulse[0].reg.0 |= data;
            }
            0x4005..=0x4007 => {
                let offset = (addr & 0x03) * 8;
                let data = (data as u32) << offset;
                self.pulse[1].reg.0 &= !(0xFF << offset);
                self.pulse[1].reg.0 |= data;
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
