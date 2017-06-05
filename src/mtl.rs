//   Copyright 2017 GFX Developers
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

use std::io::BufRead;
use std::str::FromStr;


pub struct Material {
    pub name: String,

    pub ka: Option<[f32; 3]>,
    pub kd: Option<[f32; 3]>,
    pub ks: Option<[f32; 3]>,
    pub ke: Option<[f32; 3]>,
    pub km: Option<f32>,
    pub tf: Option<[f32; 3]>,
    pub ns: Option<f32>,
    pub ni: Option<f32>,
    pub tr: Option<f32>,
    pub d: Option<f32>,
    pub illum: Option<i32>,

    pub map_ka: Option<String>,
    pub map_kd: Option<String>,
    pub map_ks: Option<String>,
    pub map_ke: Option<String>,
    pub map_ns: Option<String>,
    pub map_d: Option<String>,
    pub map_bump: Option<String>,
    pub map_refl: Option<String>,
}

impl Material {
    fn new(name: String) -> Self {
        Material {
            name: name,
            ka: None,
            kd: None,
            ks: None,
            ke: None,
            km: None,
            ns: None,
            ni: None,
            tr: None,
            tf: None,
            d: None,
            map_ka: None,
            map_kd: None,
            map_ks: None,
            map_ke: None,
            map_ns: None,
            map_d: None,
            map_bump: None,
            map_refl: None,
            illum: None,
        }
    }
}

struct Parser<I>(I);

impl<'a, I: Iterator<Item = &'a str>> Parser<I> {
    fn get_vec(&mut self) -> Option<[f32; 3]> {
        let (x, y, z) = match (self.0.next(), self.0.next(), self.0.next()) {
            (Some(x), Some(y), Some(z)) => (x, y, z),
            other => {
                println!("invalid {:?}", other);
                return None;
            }
        };

        match (x.parse::<f32>(), y.parse::<f32>(), z.parse::<f32>()) {
            (Ok(x), Ok(y), Ok(z)) => Some([x, y, z]),
            other => {
                println!("invalid {:?}", other);
                None
            }
        }
    }

    fn get_i32(&mut self) -> Option<i32> {
        match self.0.next() {
            Some(v) => FromStr::from_str(v).ok(),
            None => {
                println!("missing i32");
                None
            }
        }
    }

    fn get_f32(&mut self) -> Option<f32> {
        match self.0.next() {
            Some(v) => FromStr::from_str(v).ok(),
            None => {
                println!("missing f32");
                None
            }
        }
    }

    fn get_string(&mut self) -> Option<String> {
        match self.0.next() {
            Some(v) => Some(v.to_string()),
            None => {
                println!("missing String");
                None
            }
        }
    }
}


pub struct Mtl {
    pub materials: Vec<Material>,
}

impl Mtl {
    fn new() -> Self {
        Mtl { materials: Vec::new() }
    }

    pub fn load<B: BufRead>(file: &mut B) -> Self {
        let mut mtl = Mtl::new();
        let mut material = None;
        for line in file.lines() {
            let mut parser = match line {
                Ok(ref line) => Parser(line.split_whitespace().filter(|s| !s.is_empty())),
                Err(err) => panic!("failed to readline {:?}", err),
            };
            match parser.0.next() {
                Some("newmtl") => {
                    mtl.materials.extend(material.take());
                    material = Some(Material::new(parser.0.next().expect("Failed to read name").to_string()))
                }
                Some("Ka") => {
                    if let Some(ref mut m) = material {
                        m.ka = parser.get_vec();
                    }
                }
                Some("Kd") => {
                    if let Some(ref mut m) = material {
                        m.kd = parser.get_vec();
                    }
                }
                Some("Ks") => {
                    if let Some(ref mut m) = material {
                        m.ks = parser.get_vec();
                    }
                }
                Some("Ke") => {
                    if let Some(ref mut m) = material {
                        m.ke = parser.get_vec();
                    }
                }
                Some("Ns") => {
                    if let Some(ref mut m) = material {
                        m.ns = parser.get_f32();
                    }
                }
                Some("Ni") => {
                    if let Some(ref mut m) = material {
                        m.ni = parser.get_f32();
                    }
                }
                Some("Km") => {
                    if let Some(ref mut m) = material {
                        m.km = parser.get_f32();
                    }
                }
                Some("d") => {
                    if let Some(ref mut m) = material {
                        m.d = parser.get_f32();
                    }
                }
                Some("Tr") => {
                    if let Some(ref mut m) = material {
                        m.tr = parser.get_f32();
                    }
                }
                Some("Tf") => {
                    if let Some(ref mut m) = material {
                        m.tf = parser.get_vec();
                    }
                }
                Some("illum") => {
                    if let Some(ref mut m) = material {
                        m.illum = parser.get_i32();
                    }
                }
                Some("map_Ka") => {
                    if let Some(ref mut m) = material {
                        m.map_ka = parser.get_string();
                    }
                }
                Some("map_Kd") => {
                    if let Some(ref mut m) = material {
                        m.map_kd = parser.get_string();
                    }
                }
                Some("map_Ks") => {
                    if let Some(ref mut m) = material {
                        m.map_ks = parser.get_string();
                    }
                }
                Some("map_d") => {
                    if let Some(ref mut m) = material {
                        m.map_d = parser.get_string();
                    }
                }
                Some("map_refl") => {
                    if let Some(ref mut m) = material {
                        m.map_refl = parser.get_string();
                    }
                }
                Some("map_bump") | Some("map_Bump") | Some("bump") => {
                    if let Some(ref mut m) = material {
                        m.map_bump = parser.get_string();
                    }
                }
                Some("#") | None => {}
                other => {
                    panic!("unhandled mtl: {:?}", other);
                }
            }
        }

        if material.is_some() {
            mtl.materials.push(material.take().unwrap());
        }

        mtl
    }
}