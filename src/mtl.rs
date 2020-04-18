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

use std::borrow::Cow;
use std::io::{BufRead, Error};
use std::str::FromStr;
use std::io;

#[derive(Debug, Clone)]
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
    pub fn new(name: String) -> Self {
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

/// Indicates type of a missing value
#[derive(Debug)]
pub enum MtlMissingType {
    /// i32
    Tyi32,
    /// f32
    Tyf32,
    /// String
    TyString,
}

/// Errors parsing or loading a .mtl file.
#[derive(Debug)]
pub enum MtlError {
    IoError(io::Error),
    /// Given instruction was not in .mtl spec.
    InvalidInstruction(String),
    /// Attempted to parse value, but failed.
    InvalidValue(String),
    /// `newmtl` issued, but no name provided.
    MissingMaterialName,
    /// Instruction requires a value, but that value was not provided.
    MissingValue(MtlMissingType),
}

impl From<io::Error> for MtlError {
    fn from(e: Error) -> Self {
        Self::IoError(e)
    }
}

impl<'a> From<Material> for Cow<'a, Material> {
    #[inline]
    fn from(s: Material) -> Cow<'a, Material> {
        Cow::Owned(s)
    }
}

struct Parser<I>(I);

impl<'a, I: Iterator<Item = &'a str>> Parser<I> {
    fn get_vec(mut self) -> Result<[f32; 3], MtlError> {
        let (x, y, z) = match (self.0.next(), self.0.next(), self.0.next()) {
            (Some(x), Some(y), Some(z)) => (x, y, z),
            other => {
                return Err(MtlError::InvalidValue(format!("{:?}", other)));
            }
        };

        match (x.parse::<f32>(), y.parse::<f32>(), z.parse::<f32>()) {
            (Ok(x), Ok(y), Ok(z)) => Ok([x, y, z]),
            other => {
                Err(MtlError::InvalidValue(format!("{:?}", other)))
            }
        }
    }

    fn get_i32(mut self) -> Result<i32, MtlError> {
        match self.0.next() {
            Some(v) => FromStr::from_str(v).map_err(|_| MtlError::InvalidValue(v.to_string())),
            None => {
                Err(MtlError::MissingValue(MtlMissingType::Tyi32))
            }
        }
    }

    fn get_f32(mut self) -> Result<f32, MtlError> {
        match self.0.next() {
            Some(v) => FromStr::from_str(v).map_err(|_| MtlError::InvalidValue(v.to_string())),
            None => {
                Err(MtlError::MissingValue(MtlMissingType::Tyf32))
            }
        }
    }

    fn get_string(mut self) -> Result<String, MtlError> {
        match self.0.next() {
            Some(v) => {
                // See note on mtllib parsing in obj.rs for why this is needed/works
                Ok(self.0.fold(v.to_string(), |mut existing, next| {
                    existing.push(' ');
                    existing.push_str(next);
                    existing
                }))
            },
            None => {
                Err(MtlError::MissingValue(MtlMissingType::TyString))
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

    pub fn load<B: BufRead>(file: &mut B) -> Result<Self, MtlError> {
        let mut mtl = Mtl::new();
        let mut material = None;
        for line in file.lines() {
            let mut parser = match line {
                Ok(ref line) => Parser(line.split_whitespace().filter(|s| !s.is_empty())),
                Err(err) => return Err(MtlError::IoError(err)),
            };
            match parser.0.next() {
                Some("newmtl") => {
                    mtl.materials.extend(material.take());
                    material = Some(Material::new(parser.0.next().ok_or_else(|| MtlError::MissingMaterialName)?.to_string()));
                }
                Some("Ka") => {
                    if let Some(ref mut m) = material {
                        m.ka = Some(parser.get_vec()?);
                    }
                }
                Some("Kd") => {
                    if let Some(ref mut m) = material {
                        m.kd = Some(parser.get_vec()?);
                    }
                }
                Some("Ks") => {
                    if let Some(ref mut m) = material {
                        m.ks = Some(parser.get_vec()?);
                    }
                }
                Some("Ke") => {
                    if let Some(ref mut m) = material {
                        m.ke = Some(parser.get_vec()?);
                    }
                }
                Some("Ns") => {
                    if let Some(ref mut m) = material {
                        m.ns = Some(parser.get_f32()?);
                    }
                }
                Some("Ni") => {
                    if let Some(ref mut m) = material {
                        m.ni = Some(parser.get_f32()?);
                    }
                }
                Some("Km") => {
                    if let Some(ref mut m) = material {
                        m.km = Some(parser.get_f32()?);
                    }
                }
                Some("d") => {
                    if let Some(ref mut m) = material {
                        m.d = Some(parser.get_f32()?);
                    }
                }
                Some("Tr") => {
                    if let Some(ref mut m) = material {
                        m.tr = Some(parser.get_f32()?);
                    }
                }
                Some("Tf") => {
                    if let Some(ref mut m) = material {
                        m.tf = Some(parser.get_vec()?);
                    }
                }
                Some("illum") => {
                    if let Some(ref mut m) = material {
                        m.illum = Some(parser.get_i32()?);
                    }
                }
                Some("map_Ka") => {
                    if let Some(ref mut m) = material {
                        m.map_ka = Some(parser.get_string()?);
                    }
                }
                Some("map_Kd") => {
                    if let Some(ref mut m) = material {
                        m.map_kd = Some(parser.get_string()?);
                    }
                }
                Some("map_Ks") => {
                    if let Some(ref mut m) = material {
                        m.map_ks = Some(parser.get_string()?);
                    }
                }
                Some("map_d") => {
                    if let Some(ref mut m) = material {
                        m.map_d = Some(parser.get_string()?);
                    }
                }
                Some("map_refl") => {
                    if let Some(ref mut m) = material {
                        m.map_refl = Some(parser.get_string()?);
                    }
                }
                Some("map_bump") | Some("map_Bump") | Some("bump") => {
                    if let Some(ref mut m) = material {
                        m.map_bump = Some(parser.get_string()?);
                    }
                }
                Some(other) => {
                    if !other.starts_with("#") {
                        return Err(MtlError::InvalidInstruction(other.to_string()));
                    }
                }
                None => {}
            }
        }

        if let Some(material) = material {
            mtl.materials.push(material);
        }

        Ok(mtl)
    }
}