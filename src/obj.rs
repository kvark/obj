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


#[cfg(feature = "genmesh")]
pub use genmesh::{Polygon, Quad, Triangle};

use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Error};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use mtl::{Material, Mtl, MtlError};

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct IndexTuple(pub usize, pub Option<usize>, pub Option<usize>);
pub type SimplePolygon = Vec<IndexTuple>;

pub trait GenPolygon: Clone {
    fn new(line_number: usize, data: SimplePolygon) -> Self;
    fn try_new(line_number: usize, data: SimplePolygon) -> Result<Self,ObjError>;
}

impl GenPolygon for SimplePolygon {
    fn new(_line_number: usize, data: Self) -> Self {
        data
    }
    fn try_new(_line_number: usize, data: SimplePolygon) -> Result<Self,ObjError> {
        Ok(data)
    }
}

#[cfg(feature = "genmesh")]
impl GenPolygon for Polygon<IndexTuple> {
    fn new(line_number: usize, gs: SimplePolygon) -> Self {
        Polygon::<IndexTuple>::try_new(line_number, gs).unwrap()
    }
    fn try_new(line_number: usize, gs: SimplePolygon) -> Result<Self,ObjError> {
        match gs.len() {
            3 => Ok(Polygon::PolyTri(Triangle::new(gs[0], gs[1], gs[2]))),
            4 => Ok(Polygon::PolyQuad(Quad::new(gs[0], gs[1], gs[2], gs[3]))),
            n => return Err(ObjError::GenMeshTooManyVertsInPolygon {line_number, vert_count}),
        }
    }
}

/// Errors parsing or loading a .obj file.
#[derive(Debug)]
pub enum ObjError {
    Io(io::Error),
    /// One of the arguments to `f` is malformed.
    MalformedFaceGroup {
        line_number: usize,
        group: String,
    },
    /// An argument list either has unparsable arguments or is
    /// missing one or more arguments.
    ArgumentListFailure {
        line_number: usize,
        list: String,
    },
    /// Command found that is not in the .obj spec.
    UnexpectedCommand {
        line_number: usize,
        command: String,
    },
    /// `mtllib` command issued, but no name was specified.
    MissingMTLName {
        line_number: usize,
    },
    /// [`genmesh::Polygon`] only supports triangles and squares.
    #[cfg(feature = "genmesh")]
    GenMeshTooManyVertsInPolygon{
        line_number: usize,
        vert_count: usize,
    },
}

impl From<io::Error> for ObjError {
    fn from(e: Error) -> Self {
        Self::Io(e)
    }
}

#[derive(Debug, Clone)]
pub struct Object<'a, P>
where
    P: 'a + GenPolygon,
{
    pub name: String,
    pub groups: Vec<Group<'a, P>>,
}

impl<'a, P> Object<'a, P>
where
    P: GenPolygon,
{
    pub fn new(name: String) -> Self {
        Object { name: name, groups: Vec::new() }
    }
}

#[derive(Debug, Clone)]
pub struct Group<'a, P>
where
    P: 'a + GenPolygon,
{
    pub name: String,
    /// An index is used to tell groups apart that share the same name
    pub index: usize,
    pub material: Option<Cow<'a, Material>>,
    pub polys: Vec<P>,
}

impl<'a, P> Group<'a, P>
where
    P: 'a + GenPolygon,
{
    pub fn new(name: String) -> Self {
        Group {
            name: name,
            index: 0,
            material: None,
            polys: Vec::new(),
        }
    }
}

pub struct Obj<'a, P>
where
    P: 'a + GenPolygon,
{
    pub position: Vec<[f32; 3]>,
    pub texture: Vec<[f32; 2]>,
    pub normal: Vec<[f32; 3]>,
    pub objects: Vec<Object<'a, P>>,
    pub material_libs: Vec<String>,
    pub path: PathBuf,
}

fn normalize(idx: isize, len: usize) -> usize {
    if idx < 0 {
        (len as isize + idx) as usize
    } else {
        idx as usize - 1
    }
}

impl<'a, P> Obj<'a, P>
where
    P: GenPolygon,
{
    fn new() -> Self {
        Obj {
            position: Vec::new(),
            texture: Vec::new(),
            normal: Vec::new(),
            objects: Vec::new(),
            material_libs: Vec::new(),
            path: PathBuf::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Obj<P>, ObjError> {
        let f = File::open(path)?;
        let mut obj = Obj::load_buf(&mut BufReader::new(f))?;
        // unwrap is safe as we've read this file before
        obj.path = path.parent().unwrap().to_owned();

        Ok(obj)
    }

    fn load_single_mtl(base_path: impl AsRef<Path>, mtllib: &str) -> Result<Vec<Material>, MtlError> {
        let file = File::open(&base_path.as_ref().join(&mtllib))?;
        let mtl = Mtl::load(&mut BufReader::new(file))?;
        Ok(mtl.materials)
    }

    /// Loads the .mtl files referenced in the .obj file.
    ///
    /// If it encounters an error for an .mtl, it appends its error to the
    /// returning Vec, and tries the rest.
    ///
    /// The Result Err value format is a Vec, which items are tuples with first
    /// index being the the .mtl file and the second its corresponding error.
    pub fn load_mtls(&mut self) -> Result<(), Vec<(String, MtlError)>> {
        let mut errs = Vec::new();
        let mut materials = HashMap::new();

        for m in &self.material_libs {
            match Self::load_single_mtl(&self.path, m) {
                Ok(mtl_materials) => {
                    for m in mtl_materials {
                        materials.insert(m.name.clone(), Cow::from(m));
                    }
                },
                Err(err) => {
                    errs.push((m.clone(), err));
                },
            };
        }

        for object in &mut self.objects {
            for group in &mut object.groups {
                if let Some(ref mut mat) = group.material {
                    match materials.get(&mat.name) {
                        Some(newmat) => *mat = newmat.clone(),
                        None => {}
                    };
                }
            }
        }

        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }

    fn parse_two(line_number: usize, n0: Option<&str>, n1: Option<&str>) -> Result<[f32; 2], ObjError> {
        let (n0, n1) = match (n0, n1) {
            (Some(n0), Some(n1)) => (n0, n1),
            _ => {
                return Err(ObjError::ArgumentListFailure { line_number, list: format!("{:?} {:?}", n0, n1)});
            }
        };
        let normal = match (FromStr::from_str(n0), FromStr::from_str(n1)) {
            (Ok(n0), Ok(n1)) => [n0, n1],
            _ => {
                return Err(ObjError::ArgumentListFailure { line_number, list: format!("{:?} {:?}", n0, n1)});
            }
        };
        Ok(normal)
    }

    fn parse_three(line_number: usize, n0: Option<&str>, n1: Option<&str>, n2: Option<&str>) -> Result<[f32; 3], ObjError> {
        let (n0, n1, n2) = match (n0, n1, n2) {
            (Some(n0), Some(n1), Some(n2)) => (n0, n1, n2),
            _ => {
                return Err(ObjError::ArgumentListFailure { line_number, list: format!("{:?} {:?} {:?}", n0, n1, n2)});
            }
        };
        let normal = match (FromStr::from_str(n0), FromStr::from_str(n1), FromStr::from_str(n2)) {
            (Ok(n0), Ok(n1), Ok(n2)) => [n0, n1, n2],
            _ => {
                return Err(ObjError::ArgumentListFailure { line_number, list: format!("{:?} {:?} {:?}", n0, n1, n2)});
            }
        };
        Ok(normal)
    }

    fn parse_group(&self, line_number: usize, group: &str) -> Result<IndexTuple, ObjError> {
        let mut group_split = group.split('/');
        let p: Option<isize> = group_split.next().and_then(|idx| FromStr::from_str(idx).ok());
        let t: Option<isize> =
            group_split.next().and_then(|idx| if idx != "" { FromStr::from_str(idx).ok() } else { None });
        let n: Option<isize> = group_split.next().and_then(|idx| FromStr::from_str(idx).ok());

        match (p, t, n) {
            (Some(p), v, n) => {
                Ok(IndexTuple(normalize(p, self.position.len()),
                              v.map(|v| normalize(v, self.texture.len())),
                              n.map(|n| normalize(n, self.normal.len()))))
            }
            _ => Err(ObjError::MalformedFaceGroup {line_number, group: String::from(group)}),
        }
    }

    fn parse_face<'b, I>(&self, line_number: usize, groups: &mut I) -> Result<P, ObjError>
    where
        I: Iterator<Item = &'b str>,
    {
        let mut ret = Vec::with_capacity(3);
        for g in groups {
            let ituple = self.parse_group(line_number,g)?;
            ret.push(ituple);
        }
        P::try_new(line_number, ret)
    }

    pub fn load_buf<B>(input: &mut B) -> Result<Self, ObjError>
    where
        B: BufRead,
    {
        let mut dat = Obj::new();
        let mut object = Object::new("default".to_string());
        let mut group: Option<Group<P>> = None;

        for (idx, line) in input.lines().enumerate() {
            let (line, mut words) = match line {
                Ok(ref line) => (line.clone(), line.split_whitespace().filter(|s| !s.is_empty())),
                Err(err) => {
                    return Err(ObjError::Io(io::Error::new(io::ErrorKind::InvalidData, format!("failed to readline {}", err))));
                }
            };
            let first = words.next();

            match first {
                Some("v") => {
                    let (v0, v1, v2) = (words.next(), words.next(), words.next());
                    dat.position.push(Self::parse_three(idx, v0, v1, v2)?);
                }
                Some("vt") => {
                    let (t0, t1) = (words.next(), words.next());
                    dat.texture.push(Self::parse_two(idx, t0, t1)?);
                }
                Some("vn") => {
                    let (n0, n1, n2) = (words.next(), words.next(), words.next());
                    dat.normal.push(Self::parse_three(idx, n0, n1, n2)?);
                }
                Some("f") => {
                    let poly = dat.parse_face(idx, &mut words)?;
                    group = Some(match group {
                                     None => {
                                         let mut g = Group::new("default".to_string());
                                         g.polys.push(poly);
                                         g
                                     }
                                     Some(mut g) => {
                                         g.polys.push(poly);
                                         g
                                     }
                                 });
                }
                Some("o") => {
                    group = match group {
                        Some(val) => {
                            object.groups.push(val);
                            dat.objects.push(object);
                            None
                        }
                        None => None,
                    };
                    object = if line.len() > 2 {
                        let name = line[1..].trim();
                        Object::new(name.to_string())
                    } else {
                        Object::new("default".to_string())
                    };
                }
                Some("g") => {
                    group = match group {
                        Some(val) => {
                            object.groups.push(val);
                            None
                        }
                        None => None,
                    };
                    if line.len() > 2 {
                        let name = line[2..].trim();
                        group = Some(Group::new(name.to_string()));
                    }
                }
                Some("mtllib") => {
                    // Obj strictly does not allow spaces in filenames.
                    // "mtllib Some File.mtl" is forbidden.
                    // However, everyone does it anyway and if we want to ingest blender-outputted files, we need to support it.
                    // This works by walking word by word and combining them with a space in between. This may not be a totally
                    // accurate way to do it, but until the parser can be re-worked, this is good-enough, better-than-before solution.
                    let first_word = words.next().ok_or_else(|| ObjError::MissingMTLName {line_number: idx})?.to_string();
                    let name = words.fold(first_word, |mut existing, next| {
                        existing.push(' ');
                        existing.push_str(next);
                        existing
                    });
                    dat.material_libs.push(name);
                }
                Some("usemtl") => {
                    let mut g = match group {
                        Some(g) => g,
                        None => Group::new("default".to_string()),
                    };
                    // we found a new material that was applied to an existing
                    // object. It is treated as a new group.
                    if g.material.is_some() {
                        object.groups.push(g.clone());
                        g.index += 1;
                        g.polys.clear();
                    }
                    g.material = match words.next() {
                        Some(w) => Some(Cow::from(Material::new(w.to_string()))),
                        None => None,
                    };
                    group = Some(g);
                }
                Some("s") => (),
                Some("l") => (),
                Some(other) => {
                    if !other.starts_with("#") {
                        return Err(ObjError::UnexpectedCommand {line_number: idx, command: other.to_string()})
                    }
                }
                None => (),
            }
        }
        match group {
            Some(g) => object.groups.push(g),
            None => (),
        };
        dat.objects.push(object);
        Ok(dat)
    }
}
