use bevy::prelude::*;
use bitfield::bitfield;
use noise::NoisePlugin;
use pulse::PulsePlugin;
use triangle::TrianglePlugin;

mod noise;
mod pulse;
mod triangle;

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
    shift_amount, set_shift_amount: 10, 8;
    negate, set_negate: 11, 11;
    sweep_period, set_sweep_period: 14, 12;
    sweep_enabled, set_sweep_enabled: 15, 15;
    timer, set_timer: 26, 16;
    length_counter, set_length_counter: 31, 27;
}

#[derive(Default)]
struct Pulse {
    reg: PulseRegister,
    target_period: u32,
    volume: u8,
    mute: bool,
    // length
    length_reload: bool,
    length_counter: u8,
    // envelope
    envelope_reload: bool,
    decay_level: u8,
    envelope_divider: u8,
    envelope_counter: u8,
    envelope_period: u8,
    // sweep
    sweep_reload: bool,
    sweep_counter: u8,
    sweep_complement: bool,
}

impl Pulse {
    fn new(sweep_complement: bool) -> Self {
        Self {
            sweep_complement,
            ..default()
        }
    }

    fn update_target_period(&mut self) {
        let change_amount = self.reg.timer() >> self.reg.shift_amount();
        self.target_period = if self.reg.negate() != 0 {
            self.reg
                .timer()
                .saturating_sub(change_amount + (self.sweep_complement as u32))
        } else {
            self.reg.timer() + change_amount
        };
        self.mute = self.target_period > 0x7FF || self.reg.timer() < 0x08;
    }

    fn clock_length_counter(&mut self) {
        if self.length_reload {
            self.length_counter = LENGTH_COUNTER_TABLE[self.reg.length_counter() as usize];
            self.length_reload = false;
        } else {
            if self.reg.length_counter_halt() == 0 {
                let next_length = self.length_counter.saturating_sub(1);
                self.length_counter = next_length;
            }
        }
    }

    fn clock_envelope(&mut self) {
        if self.envelope_reload {
            self.envelope_reload = false;
            self.decay_level = 15;
            self.envelope_period = (self.reg.volume() + 1) as u8;
            self.envelope_divider = self.envelope_period;
        } else {
            self.envelope_divider = self.envelope_divider.saturating_sub(1);
            if self.envelope_divider == 0 {
                self.envelope_divider = self.envelope_period;
                if self.decay_level > 0 {
                    self.decay_level -= 1;
                } else if self.reg.envelope_loop() != 0 {
                    self.decay_level = 15;
                }
            }
        }
        if self.reg.constant_volume() != 0 {
            self.volume = self.reg.volume() as u8;
        } else {
            self.volume = self.decay_level;
        }
    }

    fn clock_sweep(&mut self) {
        if self.reg.sweep_enabled() != 0 && self.sweep_counter == 0 && self.reg.shift_amount() != 0
        {
            if !self.mute {
                self.reg.set_timer(self.target_period);
            }
        }
        if self.sweep_reload || self.sweep_counter == 0 {
            self.sweep_counter = self.reg.sweep_period() as u8;
            self.sweep_reload = false;
        } else {
            self.sweep_counter = self.sweep_counter.saturating_sub(1);
        }
    }
}

bitfield! {
    #[derive(Default)]
    struct TriangleRegister(u32);
    impl Debug;
    linear_counter, set_linear_counter: 6, 0;
    linear_control, set_linear_control: 7, 7;
    length_counter_halt, set_length_counter_halt: 7, 7;
    timer, set_timer: 26, 16;
    length_counter, set_length_counter: 31, 27;
}

#[derive(Default)]
struct Triangle {
    reg: TriangleRegister,
    length_counter_reload: bool,
    length_counter: u8,
    linear_counter_reload: bool,
    linear_counter: u8,
}

impl Triangle {
    fn clock_linear_counter(&mut self) {
        if self.linear_counter_reload {
            self.linear_counter = self.reg.linear_counter() as u8;
        } else {
            self.linear_counter = self.linear_counter.saturating_sub(1);
        }
        if self.reg.linear_control() == 0 {
            self.linear_counter_reload = false;
        }
    }

    fn clock_length_counter(&mut self) {
        if self.length_counter_reload {
            self.length_counter = LENGTH_COUNTER_TABLE[self.reg.length_counter() as usize];
            self.length_counter_reload = false;
        } else {
            if self.reg.length_counter_halt() == 0 {
                self.length_counter = self.length_counter.saturating_sub(1);
            }
        }
    }
}

bitfield! {
    #[derive(Default)]
    struct NoiseRegister(u32);
    impl Debug;
    volume, set_volume: 3, 0;
    constant_volume, set_constant_volume: 4, 4;
    envelope_loop, set_envelope_loop: 5, 5;
    length_counter_halt, set_length_counter_halt: 5, 5;
    period, set_period: 19, 16;
    loop_noise, set_loop_noise: 23, 23;
    length_counter, set_length_counter: 31, 27;
}

#[derive(Default)]
struct Noise {
    reg: NoiseRegister,
    volume: u8,
    // length
    length_reload: bool,
    length_counter: u8,
    // envelope
    envelope_reload: bool,
    decay_level: u8,
    envelope_divider: u8,
    envelope_counter: u8,
    envelope_period: u8,
}

impl Noise {
    fn clock_length_counter(&mut self) {
        if self.length_reload {
            self.length_counter = LENGTH_COUNTER_TABLE[self.reg.length_counter() as usize];
            self.length_reload = false;
        } else {
            if self.reg.length_counter_halt() == 0 {
                let next_length = self.length_counter.saturating_sub(1);
                self.length_counter = next_length;
            }
        }
    }

    fn clock_envelope(&mut self) {
        if self.envelope_reload {
            self.envelope_reload = false;
            self.decay_level = 15;
            self.envelope_period = (self.reg.volume() + 1) as u8;
            self.envelope_divider = self.envelope_period;
        } else {
            self.envelope_divider = self.envelope_divider.saturating_sub(1);
            if self.envelope_divider == 0 {
                self.envelope_divider = self.envelope_period;
                if self.decay_level > 0 {
                    self.decay_level -= 1;
                } else if self.reg.envelope_loop() != 0 {
                    self.decay_level = 15;
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

#[derive(Component)]
pub struct Apu {
    pulse: [Pulse; 2],
    triangle: Triangle,
    noise: Noise,
    status: ApuStatus,
    frame_counter: FrameCounter,
    cycles: usize,
}

impl Default for Apu {
    fn default() -> Self {
        Self {
            pulse: [Pulse::new(false), Pulse::new(true)],
            triangle: Triangle::default(),
            noise: Noise::default(),
            status: ApuStatus::default(),
            frame_counter: FrameCounter::default(),
            cycles: 0,
        }
    }
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
                _ => return false,
            }
            self.pulse[0].update_target_period();
            self.pulse[1].update_target_period();
            return true;
        } else {
            false
        }
    }

    pub fn quarter_frame_tick(&mut self) {
        for pulse_id in 0..=1 {
            if (self.status.pulse() >> pulse_id) & 1 != 0 {
                self.pulse[pulse_id].clock_envelope();
            }
        }
        if self.status.noise() {
            self.noise.clock_envelope();
        }
        if self.status.triangle() {
            self.triangle.clock_linear_counter();
        }
    }

    pub fn half_frame_tick(&mut self) {
        for pulse_id in 0..=1 {
            if (self.status.pulse() >> pulse_id) & 1 != 0 {
                self.pulse[pulse_id].clock_length_counter();
                self.pulse[pulse_id].clock_sweep();
            }
        }
        if self.status.triangle() {
            self.triangle.clock_length_counter();
        }
        if self.status.noise() {
            self.noise.clock_length_counter();
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4000 | 0x4004 => {
                let pulse_id = ((addr >> 2) & 1) as usize;
                self.pulse[pulse_id].reg.0 &= !0xFF;
                self.pulse[pulse_id].reg.0 |= data as u32;

                if (self.status.pulse() >> pulse_id) & 1 != 0 {
                    self.pulse[pulse_id].envelope_reload = true;
                    self.pulse[pulse_id].envelope_counter =
                        LENGTH_COUNTER_TABLE[self.pulse[pulse_id].reg.length_counter() as usize];
                }
            }
            0x4001 | 0x4005 => {
                let pulse_id = ((addr >> 2) & 1) as usize;
                self.pulse[pulse_id].reg.0 &= !(0xFFu32 << 8);
                self.pulse[pulse_id].reg.0 |= (data as u32) << 8;
                if (self.status.pulse() >> pulse_id) & 1 != 0 {
                    self.pulse[pulse_id].sweep_reload = true;
                }
            }
            0x4002 | 0x4006 => {
                let pulse_id = ((addr >> 2) & 1) as usize;
                self.pulse[pulse_id].reg.0 &= !(0xFFu32 << 16);
                self.pulse[pulse_id].reg.0 |= (data as u32) << 16;
            }
            0x4003 | 0x4007 => {
                let pulse_id = ((addr >> 2) & 1) as usize;
                self.pulse[pulse_id].reg.0 &= !(0xFFu32 << 24);
                self.pulse[pulse_id].reg.0 |= (data as u32) << 24;
                if (self.status.pulse() >> pulse_id) & 1 != 0 {
                    self.pulse[pulse_id].length_reload = true;
                }
            }
            0x4008 => {
                self.triangle.reg.0 &= !0xFFu32;
                self.triangle.reg.0 |= data as u32;
                if self.status.triangle() {
                    self.triangle.linear_counter_reload = true;
                }
            }
            0x400A => {
                self.triangle.reg.0 &= !(0xFFu32 << 16);
                self.triangle.reg.0 |= (data as u32) << 16;
            }
            0x400B => {
                self.triangle.reg.0 &= !(0xFFu32 << 24);
                self.triangle.reg.0 |= (data as u32) << 24;
                if self.status.triangle() {
                    self.triangle.length_counter_reload = true;
                    self.triangle.linear_counter_reload = true;
                }
            }
            0x400C => {
                self.noise.reg.0 &= !0xFF;
                self.noise.reg.0 |= data as u32;
                if self.status.noise() {
                    self.noise.envelope_reload = true;
                    self.noise.envelope_counter =
                        LENGTH_COUNTER_TABLE[self.noise.reg.length_counter() as usize];
                }
            }
            0x400E => {
                self.noise.reg.0 &= !(0xFFu32 << 16);
                self.noise.reg.0 |= (data as u32) << 16;
            }
            0x400F => {
                self.noise.reg.0 &= !(0xFFu32 << 24);
                self.noise.reg.0 |= (data as u32) << 24;
                if self.status.noise() {
                    self.noise.length_reload = true;
                }
            }
            0x4015 => {
                // should also reset length counter
                self.status.0 = data;
                self.pulse[0].length_counter *= (data >> 0) & 1;
                self.pulse[1].length_counter *= (data >> 1) & 1;
                self.triangle.length_counter *= (data >> 2) & 1;
            }
            0x4017 => {
                self.frame_counter.0 = data;
            }
            _ => {}
        }
    }
}

pub struct ApuPlugin;

impl Plugin for ApuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PulsePlugin::<0>,
            PulsePlugin::<1>,
            TrianglePlugin,
            NoisePlugin,
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::apu::Apu;

    #[test]
    fn sweep_reg() {
        let mut apu = Apu::default();
        assert!(apu.pulse[0].reg.sweep_enabled() == 0);

        apu.cpu_write(0x4001, 0xFF);
        assert!(apu.pulse[0].reg.sweep_enabled() == 1);

        apu.cpu_write(0x4001, 0x7F);
        assert!(apu.pulse[0].reg.sweep_enabled() == 0);

        apu.cpu_write(0x4002, 0b11001010);
        apu.cpu_write(0x4003, 0b11011101);
        assert_eq!(apu.pulse[0].reg.timer(), 0b10111001010);
        assert_eq!(apu.pulse[0].reg.length_counter(), 0b11011);

        apu.cpu_write(0x4006, 0b11001010);
        apu.cpu_write(0x4007, 0b11011101);
        assert_eq!(
            apu.pulse[1].reg.timer(),
            0b10111001010,
            "{:#010b}",
            apu.pulse[1].reg.timer()
        );
        assert_eq!(apu.pulse[1].reg.length_counter(), 0b11011);
    }
}
