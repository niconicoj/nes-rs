use crate::cartridge::{mapper::nrom, Cartridge};

mod bus;
mod cartridge;
mod cpu;
mod nes;
mod ppu;
mod ram;

fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).expect("failed to setup tracing");

    let mut nes = nes::Nes::default();

    let mapper = nrom::NRom128::default();
    let cartridge = Cartridge::new(mapper);

    nes.plug_cartridge(&cartridge);

    nes.tick();

    println!("Hello, world!");
}
