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

//! Parsing and writing of a .mtl file as defined in the
//! [full spec](http://paulbourke.net/dataformats/mtl/).

use std::sync::Arc;
use std::borrow::Cow;
use std::io::{self, BufRead, BufReader, Error, Read, Write};
use std::path::Path;
use std::str::FromStr;
use std::fmt;

/// The model of an a single Material as defined in the .mtl spec.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub name: String,

    // Material color and illumination
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

    // Texture and reflection maps
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
    I32,
    /// f32
    F32,
    /// String
    String,
}

impl fmt::Display for MtlMissingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MtlMissingType::I32 => write!(f, "i32"),
            MtlMissingType::F32 => write!(f, "f32"),
            MtlMissingType::String => write!(f, "String"),
        }
    }
}


/// Errors parsing or loading a .mtl file.
#[derive(Debug)]
pub enum MtlError {
    Io(io::Error),
    /// Given instruction was not in .mtl spec.
    InvalidInstruction(String),
    /// Attempted to parse value, but failed.
    InvalidValue(String),
    /// `newmtl` issued, but no name provided.
    MissingMaterialName,
    /// Instruction requires a value, but that value was not provided.
    MissingValue(MtlMissingType),
}

impl std::error::Error for MtlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MtlError::Io(err) => Some(err),
            _ => None
        }
    }
}

impl fmt::Display for MtlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MtlError::Io(err) => write!(f, "I/O error loading a .mtl file: {}", err),
            MtlError::InvalidInstruction(instruction) =>
                write!(f, "Unsupported mtl instruction: {}", instruction),
            MtlError::InvalidValue(val) =>
                write!(f, "Attempted to parse the value '{}' but failed.", val),
            MtlError::MissingMaterialName =>
                write!(f, "newmtl issued, but no name provided."),
            MtlError::MissingValue(ty) =>
                write!(f, "Instruction is missing a value of type '{}'", ty),
        }
    }
}

impl From<io::Error> for MtlError {
    fn from(e: Error) -> Self {
        Self::Io(e)
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
    fn get_vec(&mut self) -> Result<[f32; 3], MtlError> {
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

    fn get_i32(&mut self) -> Result<i32, MtlError> {
        match self.0.next() {
            Some(v) => FromStr::from_str(v).map_err(|_| MtlError::InvalidValue(v.to_string())),
            None => {
                Err(MtlError::MissingValue(MtlMissingType::I32))
            }
        }
    }

    fn get_f32(&mut self) -> Result<f32, MtlError> {
        match self.0.next() {
            Some(v) => FromStr::from_str(v).map_err(|_| MtlError::InvalidValue(v.to_string())),
            None => {
                Err(MtlError::MissingValue(MtlMissingType::F32))
            }
        }
    }

    fn into_string(mut self) -> Result<String, MtlError> {
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
                Err(MtlError::MissingValue(MtlMissingType::String))
            }
        }
    }
}

/// The data represented by the `mtllib` command.
///
/// The material name is replaced by the actual material data when the material libraries are
/// laoded if a match is found.
#[derive(Debug, Clone, PartialEq)]
pub struct Mtl {
    /// Name of the .mtl file.
    pub filename: String,
    /// A list of loaded materials.
    ///
    /// The individual materials are wrapped into an `Arc` to facilitate referencing this data
    /// where these materials are assigned in the `.obj` file.
    pub materials: Vec<Arc<Material>>,
}

impl Mtl {
    /// Construct a new empty mtl lib with the given file name.
    pub fn new(filename: String) -> Self {
        Mtl { filename, materials: Vec::new() }
    }

    /// Load the mtl library from the input buffer generated by the given closure.
    ///
    /// This function overwrites the contents of this library if it has already been loaded.
    pub fn reload_with<R, F>(&mut self, obj_dir: impl AsRef<Path>, mut resolve: F) -> Result<&mut Self, MtlError>
    where
        R: BufRead,
        F: FnMut(&Path, &str) -> io::Result<R>
    {
        self.reload(resolve(obj_dir.as_ref(), &self.filename)?)
    }

    /// Load the mtl library from the given input buffer.
    ///
    /// This function overwrites the contents of this library if it has already been loaded.
    pub fn reload(&mut self, input: impl Read) -> Result<&mut Self, MtlError> {
        self.materials.clear();
        let input = BufReader::new(input);
        let mut material = None;
        for line in input.lines() {
            let mut parser = match line {
                Ok(ref line) => Parser(line.split_whitespace().filter(|s| !s.is_empty())),
                Err(err) => return Err(MtlError::Io(err)),
            };
            match parser.0.next() {
                Some("newmtl") => {
                    self.materials.extend(material.take().map(Arc::new));
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
                        m.map_ka = Some(parser.into_string()?);
                    }
                }
                Some("map_Kd") => {
                    if let Some(ref mut m) = material {
                        m.map_kd = Some(parser.into_string()?);
                    }
                }
                Some("map_Ks") => {
                    if let Some(ref mut m) = material {
                        m.map_ks = Some(parser.into_string()?);
                    }
                }
                Some("map_d") => {
                    if let Some(ref mut m) = material {
                        m.map_d = Some(parser.into_string()?);
                    }
                }
                Some("map_refl") | Some("refl") => {
                    if let Some(ref mut m) = material {
                        m.map_refl = Some(parser.into_string()?);
                    }
                }
                Some("map_bump") | Some("map_Bump") | Some("bump") => {
                    if let Some(ref mut m) = material {
                        m.map_bump = Some(parser.into_string()?);
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
            self.materials.push(Arc::new(material));
        }

        Ok(self)
    }

    pub fn write_to_buf(&self, out: &mut impl Write) -> Result<(), io::Error> {
        for mtl in &self.materials {
            writeln!(out, "newmtl {}", mtl.name)?;
            if let Some([ka0, ka1, ka2]) = mtl.ka {
                writeln!(out, "Ka {} {} {}", ka0, ka1, ka2)?;
            }
            if let Some([kd0, kd1, kd2]) = mtl.kd {
                writeln!(out, "Kd {} {} {}", kd0, kd1, kd2)?;
            }
            if let Some([ks0, ks1, ks2]) = mtl.ks {
                writeln!(out, "Ks {} {} {}", ks0, ks1, ks2)?;
            }
            if let Some([ke0, ke1, ke2]) = mtl.ke {
                writeln!(out, "Ke {} {} {}", ke0, ke1, ke2)?;
            }
            if let Some(ns) = mtl.ns {
                writeln!(out, "Ns {}", ns)?;
            }
            if let Some(ns) = mtl.ns {
                writeln!(out, "Ns {}", ns)?;
            }
            if let Some(ni) = mtl.ni {
                writeln!(out, "Ni {}", ni)?;
            }
            if let Some(km) = mtl.km {
                writeln!(out, "Km {}", km)?;
            }
            if let Some(d) = mtl.d {
                writeln!(out, "d {}", d)?;
            }
            if let Some(tr) = mtl.tr {
                writeln!(out, "Tr {}", tr)?;
            }
            if let Some([tf0, tf1, tf2]) = mtl.tf {
                writeln!(out, "Tf {} {} {}", tf0, tf1, tf2)?;
            }
            if let Some(illum) = mtl.illum {
                writeln!(out, "illum {}", illum)?;
            }
            if let Some(map_ka) = &mtl.map_ka {
                writeln!(out, "map_Ka {}", map_ka)?;
            }
            if let Some(map_kd) = &mtl.map_kd {
                writeln!(out, "map_Kd {}", map_kd)?;
            }
            if let Some(map_ks) = &mtl.map_ks {
                writeln!(out, "map_Ks {}", map_ks)?;
            }
            if let Some(map_d) = &mtl.map_d {
                writeln!(out, "map_d {}", map_d)?;
            }
            if let Some(map_refl) = &mtl.map_refl {
                writeln!(out, "refl {}", map_refl)?;
            }
            if let Some(map_bump) = &mtl.map_bump {
                writeln!(out, "bump {}", map_bump)?;
            }
        }
        Ok(())
    }
}
