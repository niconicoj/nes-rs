use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PaletteLoaderError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

#[derive(Asset, Debug, TypePath)]
pub struct Palette {
    pub colors: [Color; 64],
}

impl Palette {
    pub fn get_color(&self, color_id: u8) -> Color {
        self.colors[color_id as usize]
    }
}

#[derive(Default)]
pub struct PaletteLoader;

impl AssetLoader for PaletteLoader {
    type Asset = Palette;

    type Settings = ();

    type Error = PaletteLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buffer = [0; 3];
        let mut colors = [Color::BLACK; 64];
        for i in 0..64 {
            reader.read_exact(&mut buffer).await?;
            let color = Color::srgb(
                buffer[0] as f32 / 255.0,
                buffer[1] as f32 / 255.0,
                buffer[2] as f32 / 255.0,
            );
            colors[i] = color;
        }
        Ok(Palette { colors })
    }

    fn extensions(&self) -> &[&str] {
        &["pal"]
    }
}

#[derive(Resource, Default)]
pub struct PaletteState {
    pub palette_handle: Handle<Palette>,
    pub palette_id: u16,
}
