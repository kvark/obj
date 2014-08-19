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

use core::slice::Items;
use core::iter::Map;
use std::io::BufferedReader;
use std::io::{File, Open, Read};
use std::path::Path;
use std::from_str::FromStr;
use std::collections::HashMap;

use mtl::Mtl;
pub use vertex::{Triangle, Quad, Polygon, PolyTri, PolyQuad};

pub type IndexTuple = (uint, Option<uint>, Option<uint>);

pub struct Object {
    pub name: String,
    groups: Vec<Group>

}

impl Object {
    pub fn new(name: String) -> Object {
        Object {
            name: name,
            groups: Vec::new()
        }
    }

    pub fn group_iter(&self) -> Items<Group> {
        self.groups.iter()
    }
}

pub struct Group {
    pub name: String,
    subgroup: uint,
    material: Option<String>,
    indices: Vec<Polygon<IndexTuple>>
}

impl Group {
    pub fn new(name: String) -> Group {
        Group {
            name: name,
            subgroup: 0,
            material: None,
            indices: Vec::new()
        }
    }

    pub fn indices<'a>(&'a self) -> &'a [Polygon<IndexTuple>] {
        self.indices.as_slice()
    }
}

pub struct ObjFile {
    path: Path,
    position: Vec<[f32, ..3]>,
    texture: Vec<[f32, ..2]>,
    normal: Vec<[f32, ..3]>,
    objects: Vec<Object>,
    materials: Vec<String>
}

fn normalize(idx: int, len: uint) -> uint {
    if idx < 0 {
        (len as int + idx) as uint
    } else {
        idx as uint - 1
    }
}

impl ObjFile {
    fn new() -> ObjFile {
        ObjFile {
            path: Path::new(""),
            position: Vec::new(),
            texture: Vec::new(),
            normal: Vec::new(),
            objects: Vec::new(),
            materials: Vec::new()
        }
    }

    fn parse_vertex(&mut self, v0: Option<&str>, v1: Option<&str>, v2: Option<&str>) {
        let (v0, v1, v2) = match (v0, v1, v2) {
            (Some(v0), Some(v1), Some(v2)) => (v0, v1, v2),
            _ => {
                fail!("could not parse line {} {} {}", v0, v1, v2);
            }
        };
        let vertex = match (FromStr::from_str(v0), FromStr::from_str(v1), FromStr::from_str(v2)) {
            (Some(v0), Some(v1), Some(v2)) => [v0, v1, v2],
            _ => {
                fail!("could not parse line {} {} {}", v0, v1, v2);
            }
        };
        self.position.push(vertex);
    }

    fn parse_texture(&mut self, t0: Option<&str>, t1: Option<&str>) {
        let (t0, t1) = match (t0, t1) {
            (Some(t0), Some(t1)) => (t0, t1),
            _ => {
                fail!("could not parse line {} {}", t0, t1);
            }
        };
        let texture = match (FromStr::from_str(t0), FromStr::from_str(t1)) {
            (Some(t0), Some(t1)) => [t0, t1],
            _ => {
                fail!("could not parse line {} {}", t0, t1);
            }
        };
        self.texture.push(texture);
    }

    fn parse_normal(&mut self, n0: Option<&str>, n1: Option<&str>, n2: Option<&str>) {
        let (n0, n1, n2) = match (n0, n1, n2) {
            (Some(n0), Some(n1), Some(n2)) => (n0, n1, n2),
            _ => {
                fail!("could not parse line {} {} {}", n0, n1, n2);
            }
        };
        let normal = match (FromStr::from_str(n0), FromStr::from_str(n1), FromStr::from_str(n2)) {
            (Some(n0), Some(n1), Some(n2)) => [n0, n1, n2],
            _ => {
                fail!("could not parse line {} {} {}", n0, n1, n2);
            }
        };
        self.normal.push(normal);
    }

    fn parse_group(&mut self, group: &str) 
            -> Result<(uint, Option<uint>, Option<uint>), String> {
        let mut group_split = group.split('/');
        let p: Option<int> = group_split.next().and_then(|idx| FromStr::from_str(idx));
        let t: Option<int> = group_split.next().and_then(|idx| if idx != "" { FromStr::from_str(idx) } else { None } );
        let n: Option<int> = group_split.next().and_then(|idx| FromStr::from_str(idx));

        match (p, t, n) {
            (Some(p), v, n) => Ok((normalize(p, self.position.len()),
                                   v.map(|v| normalize(v, self.texture.len())),
                                   n.map(|n| normalize(n, self.normal.len()))
                                 )),
            _ => Err(format!("poorly formed group {:s}", group))
        }
    }

    fn parse_triangle(&mut self, g0: &str, g1: &str, g2: &str) 
            -> Result<Triangle<IndexTuple>, String> {
        let g0 = self.parse_group(g0);
        let g1 = self.parse_group(g1);
        let g2 = self.parse_group(g2);

        match (g0, g1, g2) {
            (Err(e), _, _) => { Err(e) }
            (_, Err(e), _) => { Err(e) }
            (_, _, Err(e)) => { Err(e) }
            (Ok(g0), Ok(g1), Ok(g2)) => {
                Ok(Triangle::new(g0, g1, g2))
            }
        }
       
    }

    fn parse_quad(&mut self, g0: &str, g1: &str, g2: &str, g3: &str) 
            -> Result<Quad<IndexTuple>, String> {

        let g0 = self.parse_group(g0);
        let g1 = self.parse_group(g1);
        let g2 = self.parse_group(g2);
        let g3 = self.parse_group(g3);

        match (g0, g1, g2, g3) {
            (Err(e), _, _, _) => { Err(e) }
            (_, Err(e), _, _) => { Err(e) }
            (_, _, Err(e), _) => { Err(e) }
            (_, _, _, Err(e)) => { Err(e) }
            (Ok(g0), Ok(g1), Ok(g2), Ok(g3)) => {
                Ok(Quad::new(g0, g1, g2, g3))
            }
        }
    }

    fn parse_face(&mut self, g0: Option<&str>, g1: Option<&str>, g2: Option<&str>, g3: Option<&str>)
        -> Result<Polygon<IndexTuple>, String>  {
        match (g0, g1, g2, g3) {
            (Some(g0), Some(g1), Some(g2), None) => {
                self.parse_triangle(g0, g1, g2).map(|p| PolyTri(p))
            }
            (Some(g0), Some(g1), Some(g2), Some(g3)) => {
                self.parse_quad(g0, g1, g2, g3).map(|p| PolyQuad(p))
            }
            _ => {fail!("Unsupported");}
        }
    }

    pub fn load<B: Buffer>(input: &mut B) -> ObjFile {
        let mut dat = ObjFile::new();
        let mut object = Object::new("default".to_string());
        let mut group: Option<Group> = None;

        for (idx, line) in input.lines().enumerate() {
            let (line, mut words) = match line {
                Ok(ref line) => (line.as_slice(), line.as_slice().words()),
                Err(err) => fail!("failed to readline {}", err)
            };
            let first = words.next();

            match first {
                Some("v") => {
                    let (v0, v1, v2) = (words.next(), words.next(), words.next());
                    dat.parse_vertex(v0, v1, v2);
                },
                Some("vt") => {
                    let (t0, t1) = (words.next(), words.next());
                    dat.parse_texture(t0, t1);
                },
                Some("vn") => {
                    let (n0, n1, n2) = (words.next(), words.next(), words.next());
                    dat.parse_normal(n0, n1, n2);
                },
                Some("f") => {
                    let (g0, g1, g2, g3) = (words.next(), words.next(), words.next(), words.next());

                    let poly= match dat.parse_face(g0, g1, g2, g3) {
                        Err(e) => fail!("Could not parse line: {}\nline: {}: {}",
                            e, idx, line
                        ),
                        Ok(poly) => poly
                    };

                    group = Some(match group {
                        None => {
                            let mut obj = Group::new("default".to_string());
                            obj.indices.push(poly);
                            obj
                        }
                        Some(mut obj) => {
                            obj.indices.push(poly);
                            obj
                        }
                    });
                },
                Some("o") => {
                    if dat.objects.len() != 0 {
                        dat.objects.push(object);
                    }


                    object = if line.len() > 2 {
                        let name = line.slice_from(1).trim();
                        Object::new(name.to_string())
                    } else {
                        Object::new("default".to_string())
                    };

                    object = match words.next() {
                        Some(name) => Object::new(name.to_string()),
                        None => Object::new("default".to_string())
                    }                    
                },
                Some("g") => {
                    group = match group {
                        Some(val) => {
                            object.groups.push(val);
                            None
                        },
                        None => None
                    };

                    if line.len() > 2 {
                        let name = line.slice_from(1).trim();
                        group = Some(Group::new(name.to_string()));
                    }
                },
                Some("mtllib") => {
                    let name = words.next().expect("Failed to find name for mtllib");
                    dat.materials.push(name.to_string());
                }
                Some("usemtl") => {
                    group = group.map(|mut obj| {
                        obj.material = match words.next() {
                            Some(w) => Some(w.to_string()),
                            None => None
                        };
                        obj
                    });
                }
                Some("s") => (),
                Some(other) => {
                    if other.len() != 0 && other.char_at(0) != "#".char_at(0) {
                        fail!("Invalid token {} {}", other, words.next());
                    }
                }
                None => (),
            }

        }

        match group {
            Some(g) => object.groups.push(g),
            None => ()
        }
        dat.objects.push(object);
        dat
    }

    pub fn object_iter<'a>(&'a self) -> Items<Object> {
        self.objects.iter()
    }

    pub fn position<'a>(&'a self) -> &'a [[f32, ..3]] {
        self.position.as_slice()
    }

    pub fn texture<'a>(&'a self) -> &'a [[f32, ..2]] {
        self.texture.as_slice()
    }

    pub fn normal<'a>(&'a self) -> &'a [[f32, ..3]] {
        self.normal.as_slice()
    }
}