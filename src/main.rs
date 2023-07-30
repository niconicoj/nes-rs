use crate::cartridge::Cartridge;

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

    let cartridge = Cartridge::from_file("roms/test.rom");

    nes.plug_cartridge(&cartridge);

    nes.step();

    println!("Hello, world!");
}
