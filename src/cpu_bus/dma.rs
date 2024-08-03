use bevy::prelude::Component;

#[derive(Default, Component)]
pub struct Dma {
    pub page: u8,
    pub addr: u8,
    pub data: u8,
    pub status: DmaStatus,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum DmaStatus {
    #[default]
    Inactive,
    Idling,
    Transfering,
}
