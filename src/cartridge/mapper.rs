use bevy_egui::egui;

pub trait Mapper: Send + Sync {
    fn cpu_read(&self, addr: u16) -> Option<u8>;
    #[must_use]
    fn cpu_write(&mut self, addr: u16, data: u8) -> Option<()>;
    fn ppu_read(&self, addr: u16) -> Option<u8>;
    #[must_use]
    fn ppu_write(&mut self, addr: u16, data: u8) -> Option<()>;
    fn ui(&self, ui: &mut egui::Ui);
}
