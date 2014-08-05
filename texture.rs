//   Copyright 2014 Colin Sherratt
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

use image::image::{load, Error, ImageU8, ImageF32};
use graphics::Texture;

pub fn load_texture(path: &Path) -> Texture {
    let mut res = match load(path) {
        Error(s) => fail!("failed to load image: {:s} {:s}", s, path.as_str().unwrap()),
        ImageU8(d) => {
            println!("loaded texture {:s} {} {}", path.as_str().unwrap(), d.data.len(), d.depth);
            Texture::new(d.width, d.height, d.depth, d.data)
        }
        ImageF32(d) => {
            println!("loaded texture {:s} {} {}", path.as_str().unwrap(), d.data.len(), d.depth);
            Texture::new(d.width, d.height, d.depth, d.data.iter().map(|v| *v as u8).collect())
        }
    };
    res.flip();
    res
}