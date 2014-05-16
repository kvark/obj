
use core::str::Words;

use std::io::BufferedReader;
use std::io::{File, Open, Read};
use std::path::Path;
use std::from_str::FromStr;

use cgmath::vector::{Vector3};

pub struct Material {
    pub name: ~str,

    pub ka: Option<Vector3<f32>>,
    pub kd: Option<Vector3<f32>>,
    pub ks: Option<Vector3<f32>>,
    pub ke: Option<Vector3<f32>>,
    pub km: Option<f32>,
    pub tf: Option<Vector3<f32>>,
    pub ns: Option<f32>,
    pub ni: Option<f32>,
    pub tr: Option<f32>,
    pub d: Option<f32>,
    pub illum: Option<int>,

    pub map_ka:   Option<~str>,
    pub map_kd:   Option<~str>,
    pub map_ks:   Option<~str>,
    pub map_ke:   Option<~str>,
    pub map_ns:   Option<~str>,
    pub map_d:    Option<~str>,
    pub map_bump: Option<~str>,
    pub map_refl: Option<~str>,
}

impl Material {
    fn new(name: ~str) -> Material {
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

pub fn to_vec<'a>(w: &mut Words<'a>) -> Option<Vector3<f32>> {
    let (x, y, z) = match (w.next(), w.next(), w.next()) {
        (Some(x), Some(y), Some(z)) => (x, y, z),
        other => {
            println!("invalid {:?}", other);
            return None;
        }
    };

    let x: Option<f32> = FromStr::from_str(x);
    let y: Option<f32> = FromStr::from_str(y);
    let z: Option<f32> = FromStr::from_str(z);

    match (x, y, z) {
        (Some(x), Some(y), Some(z)) => Some(Vector3::new(x, y, z)),
        other => {
            println!("invalid {:?}", other);
            None
        }
    }
}

pub fn to_int<'a>(w: &mut Words<'a>) -> Option<int> {
    let v = match w.next() {
        Some(v) => v,
        other => {
            println!("invalid {:?}", other);
            return None;
        }
    };
    FromStr::from_str(v)
}

pub fn to_f32<'a>(w: &mut Words<'a>) -> Option<f32> {
    let v = match w.next() {
        Some(v) => v,
        other => {
            println!("invalid {:?}", other);
            return None;
        }
    };
    FromStr::from_str(v)
}

pub fn to_string<'a>(w: &mut Words<'a>) -> Option<~str> {
    match w.last() {
        Some(v) => Some(v.to_owned()),
        other => {
            println!("invalid {:?}", other);
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

    pub fn load(path: &Path) -> Option<Mtl> {
        println!("path {}", path.as_str());
        let mut file = match File::open_mode(path, Open, Read) {
            Ok(file) => BufferedReader::new(file),
            Err(err) => {
                println!("{}", err);
                return None
            }
        };

        let mut mtl = Mtl::new();
        let mut material = None;
        for line in file.lines() {
            let mut words = match line {
                Ok(ref line) => line.words(),
                Err(err) => fail!("failed to readline {:?}", err)
            };
            let first = words.next();
            match first {
                Some("newmtl") => {
                    if material.is_some() {
                        mtl.materials.push(material.take().unwrap());
                    }
                    material = Some(Material::new(
                        words.next().expect("Failed to read name").to_owned()
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
                        Some(ref mut m) => { m.map_ka = to_string(&mut words); }
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
                    fail!("unhandled mtl: {}", other);
                }
            }
        }

        if material.is_some() {
            mtl.materials.push(material.take().unwrap());
        }

        Some(mtl)
    }
}