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

use std::str::Words;
use std::str::FromStr;

pub struct Material {
    pub name: String,

    pub ka: Option<[f32, ..3]>,
    pub kd: Option<[f32, ..3]>,
    pub ks: Option<[f32, ..3]>,
    pub ke: Option<[f32, ..3]>,
    pub km: Option<f32>,
    pub tf: Option<[f32, ..3]>,
    pub ns: Option<f32>,
    pub ni: Option<f32>,
    pub tr: Option<f32>,
    pub d: Option<f32>,
    pub illum: Option<int>,

    pub map_ka:   Option<String>,
    pub map_kd:   Option<String>,
    pub map_ks:   Option<String>,
    pub map_ke:   Option<String>,
    pub map_ns:   Option<String>,
    pub map_d:    Option<String>,
    pub map_bump: Option<String>,
    pub map_refl: Option<String>,
}

impl Material {
    fn new(name: String) -> Material {
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
            illum: None
        }
    }
}

fn to_vec<'a>(w: &mut Words<'a>) -> Option<[f32, ..3]> {
    let (x, y, z) = match (w.next(), w.next(), w.next()) {
        (Some(x), Some(y), Some(z)) => (x, y, z),
        other => {
            println!("invalid {}", other);
            return None;
        }
    };

    let x: Option<f32> = FromStr::from_str(x);
    let y: Option<f32> = FromStr::from_str(y);
    let z: Option<f32> = FromStr::from_str(z);

    match (x, y, z) {
        (Some(x), Some(y), Some(z)) => Some([x, y, z]),
        other => {
            println!("invalid {}", other);
            None
        }
    }
}

fn to_int<'a>(w: &mut Words<'a>) -> Option<int> {
    let v = match w.next() {
        Some(v) => v,
        other => {
            println!("invalid {}", other);
            return None;
        }
    };
    FromStr::from_str(v)
}

fn to_f32<'a>(w: &mut Words<'a>) -> Option<f32> {
    let v = match w.next() {
        Some(v) => v,
        other => {
            println!("invalid {}", other);
            return None;
        }
    };
    FromStr::from_str(v)
}

fn to_string<'a>(w: &mut Words<'a>) -> Option<String> {
    match w.by_ref().last() {
        Some(v) => Some(v.to_string()),
        other => {
            println!("invalid {}", other);
            None
        }
    }
}

pub struct Mtl {
    pub materials: Vec<Material>
}

impl Mtl {
    fn new() -> Mtl {
        Mtl {
            materials: Vec::new()
        }
    }

    pub fn load<B: Buffer>(file: &mut B) -> Mtl {
        let mut mtl = Mtl::new();
        let mut material = None;
        for line in file.lines() {
            let mut words = match line {
                Ok(ref line) => line.as_slice().words(),
                Err(err) => panic!("failed to readline {}", err)
            };
            let first = words.next();
            match first {
                Some("newmtl") => {
                    if material.is_some() {
                        mtl.materials.push(material.take().unwrap());
                    }
                    material = Some(Material::new(
                        words.next().expect("Failed to read name").to_string()
                    ))
                }
                Some("Ka") => {
                    match material {
                        Some(ref mut m) => { m.ka = to_vec(&mut words); }
                        None => ()
                    }
                }
                Some("Kd") => {
                    match material {
                        Some(ref mut m) => { m.kd = to_vec(&mut words); }
                        None => ()
                    }
                }
                Some("Ks") => {
                    match material {
                        Some(ref mut m) => { m.ks = to_vec(&mut words); }
                        None => ()
                    }
                }
                Some("Ke") => {
                    match material {
                        Some(ref mut m) => { m.ke = to_vec(&mut words); }
                        None => ()
                    }
                }
                Some("Ns") => {
                    match material {
                        Some(ref mut m) => { m.ns = to_f32(&mut words); }
                        None => ()
                    }
                }
                Some("Ni") => {
                    match material {
                        Some(ref mut m) => { m.ni = to_f32(&mut words); }
                        None => ()
                    }
                }
                Some("Km") => {
                    match material {
                        Some(ref mut m) => { m.km = to_f32(&mut words); }
                        None => ()
                    }
                }
                Some("d") => {
                    match material {
                        Some(ref mut m) => { m.d = to_f32(&mut words); }
                        None => ()
                    }
                }
                Some("Tr") => {
                    match material {
                        Some(ref mut m) => { m.tr = to_f32(&mut words); }
                        None => ()
                    }
                }
                Some("Tf") => {
                    match material {
                        Some(ref mut m) => { m.tf = to_vec(&mut words); }
                        None => ()
                    }
                }
                Some("illum") => {
                    match material {
                        Some(ref mut m) => { m.illum = to_int(&mut words); }
                        None => ()
                    }
                }
                Some("map_Ka") => {
                    match material {
                        Some(ref mut m) => { m.map_ka = to_string(&mut words); }
                        None => ()
                    }
                }
                Some("map_Kd") => {
                    match material {
                        Some(ref mut m) => { m.map_kd = to_string(&mut words); }
                        None => ()
                    }
                }
                Some("map_Ks") => {
                    match material {
                        Some(ref mut m) => { m.map_ks = to_string(&mut words); }
                        None => ()
                    }
                }
                Some("map_d") => {
                    match material {
                        Some(ref mut m) => { m.map_d = to_string(&mut words); }
                        None => ()                        
                    }
                }
                Some("map_refl") => {
                    match material {
                        Some(ref mut m) => { m.map_refl = to_string(&mut words); }
                        None => ()                        
                    }
                }
                Some("map_bump") | Some("map_Bump") | Some("bump") => {
                    match material {
                        Some(ref mut m) => { m.map_bump = to_string(&mut words); }
                        None => ()
                    }                   
                }
                Some("#") | None => {},
                other => {
                    panic!("unhandled mtl: {}", other);
                }
            }
        }

        if material.is_some() {
            mtl.materials.push(material.take().unwrap());
        }

        mtl
    }
}