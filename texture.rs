

use image::image::{load_with_depth, Error, ImageU8, ImageF32};
use graphics::Texture;

pub fn load_texture(path: &Path) -> Texture {
    match load_with_depth(path, 0, true) {
        Error(s) => fail!("failed to load image: {:s} {:s}", s, path.as_str().unwrap()),
        ImageU8(d) => {
            println!("loaded texture {:s}", path.as_str().unwrap());
            Texture::new(d.width, d.height, d.depth, d.data.iter().map(|v| *v as f32).collect())
        }
        ImageF32(d) => {
            println!("loaded texture {:s}", path.as_str().unwrap());
            Texture::new(d.width, d.height, d.depth, d.data)
        }
    }
}