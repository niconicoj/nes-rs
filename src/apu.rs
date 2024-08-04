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
    current_period: u32,
    target_period: u32,
    volume: u8,
    mute: bool,
    // envelope
    envelope_reload: bool,
    decay_level: u8,
    envelope_divider: u8,
    envelope_counter: u8,
    envelope_period: u8,
    // sweep
    sweep_reload: bool,
    sweep_counter: u8,
}

impl Pulse {
    fn update_period(&mut self) {
        self.current_period = self.reg.timer();
        let change_amount = self.reg.timer() >> self.reg.shift_amount();
        self.target_period = if self.reg.negate() != 0 {
            self.current_period - change_amount
        } else {
            self.current_period + change_amount
        };
        self.mute = self.target_period > 0x7FF || self.current_period < 0x08;
    }

    fn clock_length_counter(&mut self) {
        if self.reg.length_counter_halt() == 0 {
            let next_length = self.envelope_counter.saturating_sub(1);
            self.envelope_counter = next_length;
        }
    }

    fn clock_envelope(&mut self) {
        if self.envelope_reload {
            self.decay_level = 15;
            self.envelope_divider = 15;
            self.envelope_period = self.reg.volume() as u8;
        } else {
            self.envelope_divider = self
                .envelope_divider
                .saturating_sub(15 - self.envelope_period);
            if self.envelope_divider != 0 {
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
            self.sweep_counter -= 1;
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
            self.pulse[0].update_period();
            self.pulse[1].update_period();
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
                self.pulse[pulse_id].clock_sweep();
            }
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
            0x4002..=0x4003 => {
                let offset = (addr & 0x03) * 8;
                let data = (data as u32) << offset;
                self.pulse[0].reg.0 &= !(0xFF << offset);
                self.pulse[0].reg.0 |= data;
            }
            0x4006..=0x4007 => {
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
    }
}
