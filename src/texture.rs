use glium::texture::RawImage2d;
use glium::texture::SrgbTexture2d;

pub struct Texture {
    base: RawImage2d,
    normal: SrgbTexture2d,
}

pub fn load_texture(name: &str, display: glium::Display) -> Texture {
    
}