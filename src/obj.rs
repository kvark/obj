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
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

use mtl::{Material, Mtl};

#[derive(Debug, Clone, Copy)]
pub struct IndexTuple(pub usize, pub Option<usize>, pub Option<usize>);
pub type SimplePolygon = Vec<IndexTuple>;

pub trait GenPolygon: Clone {
    fn new(data: SimplePolygon) -> Self;
}

impl GenPolygon for SimplePolygon {
    fn new(data: Self) -> Self {
        data
    }
}

#[cfg(feature = "genmesh")]
impl GenPolygon for Polygon<IndexTuple> {
    fn new(gs: SimplePolygon) -> Self {
        match gs.len() {
            3 => Polygon::PolyTri(Triangle::new(gs[0], gs[1], gs[2])),
            4 => Polygon::PolyQuad(Quad::new(gs[0], gs[1], gs[2], gs[3])),
            _ => panic!("Unsupported"),
        }
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
        }
    }

    pub fn load(path: &Path) -> io::Result<Obj<P>> {
        let f = File::open(path)?;
        let obj = Obj::load_buf(&mut BufReader::new(f))?;

        Ok(obj)
    }

    /// Loads the .mtl files referenced in the .obj file.
    ///
    /// If it encounters an error for an .mtl, it appends its error to the
    /// returning Vec, and tries the rest.
    ///
    /// The Result Err value format is a Vec, which items are tuples with first
    /// index being the the .mtl file and the second its corresponding error.
    pub fn load_mtls(&mut self) -> Result<(), Vec<(String, io::Error)>> {
        let mut errs = Vec::new();
        let mut materials = HashMap::new();

        for m in &self.material_libs {
            let file = match File::open(&m) {
                Ok(f) => f,
                Err(err) => {
                    errs.push((m.clone(), err));
                    continue;
                }
            };
            let mtl = Mtl::load(&mut BufReader::new(file));
            for m in mtl.materials {
                materials.insert(m.name.clone(), Cow::from(m));
            }
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

    fn parse_vertex(&mut self, v0: Option<&str>, v1: Option<&str>, v2: Option<&str>) {
        let (v0, v1, v2) = match (v0, v1, v2) {
            (Some(v0), Some(v1), Some(v2)) => (v0, v1, v2),
            _ => {
                panic!("could not parse line {:?} {:?} {:?}", v0, v1, v2);
            }
        };
        let vertex = match (FromStr::from_str(v0), FromStr::from_str(v1), FromStr::from_str(v2)) {
            (Ok(v0), Ok(v1), Ok(v2)) => [v0, v1, v2],
            _ => {
                panic!("could not parse line {:?} {:?} {:?}", v0, v1, v2);
            }
        };
        self.position.push(vertex);
    }

    fn parse_texture(&mut self, t0: Option<&str>, t1: Option<&str>) {
        let (t0, t1) = match (t0, t1) {
            (Some(t0), Some(t1)) => (t0, t1),
            _ => {
                panic!("could not parse line {:?} {:?}", t0, t1);
            }
        };
        let texture = match (FromStr::from_str(t0), FromStr::from_str(t1)) {
            (Ok(t0), Ok(t1)) => [t0, t1],
            _ => {
                panic!("could not parse line {:?} {:?}", t0, t1);
            }
        };
        self.texture.push(texture);
    }

    fn parse_normal(&mut self, n0: Option<&str>, n1: Option<&str>, n2: Option<&str>) {
        let (n0, n1, n2) = match (n0, n1, n2) {
            (Some(n0), Some(n1), Some(n2)) => (n0, n1, n2),
            _ => {
                panic!("could not parse line {:?} {:?} {:?}", n0, n1, n2);
            }
        };
        let normal = match (FromStr::from_str(n0), FromStr::from_str(n1), FromStr::from_str(n2)) {
            (Ok(n0), Ok(n1), Ok(n2)) => [n0, n1, n2],
            _ => {
                panic!("could not parse line {:?} {:?} {:?}", n0, n1, n2);
            }
        };
        self.normal.push(normal);
    }

    fn parse_group(&self, group: &str) -> Result<IndexTuple, String> {
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
            _ => Err(format!("poorly formed group {}", group)),
        }
    }

    fn parse_face<'b, I>(&self, groups: &mut I) -> Result<P, String>
    where
        I: Iterator<Item = &'b str>,
    {
        let mut ret = Vec::with_capacity(3);
        for g in groups {
            let ituple = self.parse_group(g)?;
            ret.push(ituple);
        }
        Ok(P::new(ret))
    }

    pub fn load_buf<B>(input: &mut B) -> io::Result<Self>
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
                    return Err(io::Error::new(io::ErrorKind::InvalidData,
                                              format!("failed to readline {}", err)))
                }
            };
            let first = words.next();

            match first {
                Some("v") => {
                    let (v0, v1, v2) = (words.next(), words.next(), words.next());
                    dat.parse_vertex(v0, v1, v2);
                }
                Some("vt") => {
                    let (t0, t1) = (words.next(), words.next());
                    dat.parse_texture(t0, t1);
                }
                Some("vn") => {
                    let (n0, n1, n2) = (words.next(), words.next(), words.next());
                    dat.parse_normal(n0, n1, n2);
                }
                Some("f") => {
                    let poly = match dat.parse_face(&mut words) {
                        Err(e) => {
                            return Err(io::Error::new(io::ErrorKind::InvalidData,
                                                      format!("could not parse line: {}\nline: {}: {}", e, idx, line)))
                        }
                        Ok(poly) => poly,
                    };
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
                    let name = words.next().expect("Failed to find name for mtllib");
                    dat.material_libs.push(name.to_string());
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
                Some(other) => {
                    if !other.starts_with("#") {
                        panic!("Invalid token {:?} {:?}", other, words.next());
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
