use std::collections::HashMap;
use std::sync::Arc;
use std::io::BufReader;
use std::fs::File;
use serde::{Deserialize, Serialize};

use glium::{Texture2d, texture::SrgbTexture2d};
use glium::texture::RawImage2d;

pub struct TextureMap {
    pub base: SrgbTexture2d,
    pub normal: Texture2d,
    pub info: Arc<HashMap<u16, [[[f32;2];4];6]>>,
}

#[derive(Serialize, Deserialize)]
struct TextureMapUnit {
    name: String,
    loc: [u32;6],
}

#[derive(Serialize, Deserialize)]
struct TextureMapInfo {
    grid: u32,
    blocks: HashMap<u16, TextureMapUnit>,
}

pub fn load_texture_map(base: &str, normal: Option<&str>, display: &glium::Display) -> TextureMap {
    // Load image and normal map

    let file = match File::open(base) {
        Ok(file) => file,
        Err(e) => {
            panic!("{}",e);
        }
    };

    let image = image::load(
        BufReader::new(file),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();

    let image_dimensions = image.dimensions();
    let raw = &image.into_raw();
    
    let image =
        RawImage2d::from_raw_rgba_reversed(raw, image_dimensions);
    let base = SrgbTexture2d::new(display, image).unwrap();

    let image = match normal {
        Some(normal) => {
            let image = image::load(
                BufReader::new(File::open(normal).ok().unwrap()),
                image::ImageFormat::Png,
            )
            .unwrap()
            .to_rgba8();
            let image_dimensions = image.dimensions();
            RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions)
        },
        None => {
            RawImage2d::from_raw_rgba_reversed(&vec![0u8; (image_dimensions.0 * image_dimensions.1 * 4) as usize], image_dimensions)
        }
    };
    let normal = Texture2d::new(display, image).unwrap();


    // Load texture map info

    let blocks = match File::open("res/blocks.json") {
        Ok(blocks) => blocks,
        Err(e) => {
            panic!("Error opening blocks.json: {}", e);
        }
    };

    let info: TextureMapInfo = match serde_json::from_reader(blocks) {
        Ok(v) => v,
        Err(e) => {
            panic!("Error parsing blocks.json: {}",e);
        }
    };
    assert_eq!(image_dimensions.0, image_dimensions.1);
    
    // Process info
    let unit_grid_size = image_dimensions.0 / info.grid;
    
    let info = calculate(unit_grid_size, info.grid, image_dimensions.0, info);

    TextureMap { base, normal, info }
}

#[inline]
fn calculate(unit_grid_size: u32, grid_size: u32, total_side_length: u32, info: TextureMapInfo) -> Arc<HashMap<u16, [[[f32;2];4];6]>> {
    let mut map = HashMap::new();
    for (id, unit) in info.blocks {
        let mut faces = [[[0.0;2];4];6];
        for (i, loc) in unit.loc.iter().enumerate() {
            let (y, x) = ((loc / grid_size) as u32, (loc % grid_size) as u32);
            let (min_x, min_y, max_x, max_y) = (x as f32 * unit_grid_size as f32 / total_side_length as f32, y as f32 * unit_grid_size as f32 / total_side_length as f32, (x+1) as f32 * unit_grid_size as f32 / total_side_length as f32, (y+1) as f32 * unit_grid_size as f32 / total_side_length as f32);
            faces[i] = [[max_x, min_y], [max_x, max_y], [min_x, max_y], [min_x, min_y]];
        }
        map.insert(id, faces);
    }

    Arc::new(map)
}