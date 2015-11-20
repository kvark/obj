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

use std::slice::Iter;
use std::str::FromStr;
use std::io::{BufRead};

#[cfg(feature = "usegenmesh")]
pub use genmesh::{Triangle, Quad, Polygon};
use words;

pub type IndexTuple = (usize, Option<usize>, Option<usize>);

pub type SimplePolygon = Vec<IndexTuple>;

pub trait GenPolygon: Clone {
    fn new(data: Vec<IndexTuple>) -> Self;
}

impl GenPolygon for SimplePolygon {
    fn new(data: Vec<IndexTuple>) -> Self { data }
}

#[cfg(feature = "usegenmesh")]
impl GenPolygon for Polygon<IndexTuple> {
  fn new(gs: Vec<IndexTuple>) -> Self {
      match gs.len() {
          3 => Polygon::PolyTri(Triangle::new(gs[0], gs[1], gs[2])),
          4 => Polygon::PolyQuad(Quad::new(gs[0], gs[1], gs[2], gs[3])),
          _ => panic!("Unsupported"),
      }
// Slice pattern syntax is experimental
//    match &gs[..] {
//      &[g0, g1, g2] => Polygon::PolyTri(Triangle::new(g0, g1, g2)),
//      &[g0, g1, g2, g3] => Polygon::PolyQuad(Quad::new(g0, g1, g2, g3)),
//      _ => panic!("Unsupported"),
//    }
  }
}

#[derive(Debug, Clone)]
pub struct Object<MTL,P: GenPolygon> {
    pub name: String,
    groups: Vec<Group<MTL,P>>

}

impl<MTL,P: GenPolygon> Object<MTL,P> {
    pub fn new(name: String) -> Object<MTL,P> {
        Object {
            name: name,
            groups: Vec::new()
        }
    }

    pub fn group_iter(&self) -> Iter<Group<MTL,P>> {
        self.groups.iter()
    }
}

#[derive(Debug, Clone)]
pub struct Group<MTL,P: GenPolygon> {
    pub name: String,
    /// An index is used to tell groups apart that share the
    /// same name
    pub index: usize,
    pub material: Option<MTL>,
    pub indices: Vec<P>
}

impl<MTL,P: GenPolygon> Group<MTL,P> {
    pub fn new(name: String) -> Group<MTL,P> {
        Group {
            name: name,
            index: 0,
            material: None,
            indices: Vec::new()
        }
    }

    pub fn indices(&self) -> &[P] {
        &self.indices[..]
    }
}

pub struct Obj<MTL,P: GenPolygon> {
    position: Vec<[f32; 3]>,
    texture: Vec<[f32; 2]>,
    normal: Vec<[f32; 3]>,
    objects: Vec<Object<MTL,P>>,
    materials: Vec<String>
}

fn normalize(idx: isize, len: usize) -> usize {
    if idx < 0 {
        (len as isize + idx) as usize
    } else {
        idx as usize - 1
    }
}

impl<MTL,P: GenPolygon> Obj<MTL,P> {
    fn new() -> Obj<MTL,P> {
        Obj {
            position: Vec::new(),
            texture: Vec::new(),
            normal: Vec::new(),
            objects: Vec::new(),
            materials: Vec::new()
        }
    }

    pub fn object_iter(&self) -> Iter<Object<MTL,P>> {
        self.objects.iter()
    }

    pub fn position(&self) -> &[[f32; 3]] {
        &self.position[..]
    }

    pub fn texture(&self) -> &[[f32; 2]] {
        &self.texture[..]
    }

    pub fn normal(&self) -> &[[f32; 3]] {
        &self.normal[..]
    }

    pub fn materials(&self) -> &[String] {
        &self.materials[..]
    }

    pub fn map<T, F>(self, mut f: F) -> Obj<T,P> where F: FnMut(Group<MTL,P>) -> Group<T,P> {
        let Obj {
            position,
            texture,
            normal,
            objects,
            materials
        } = self;

        let objects = objects.into_iter()
            .map(|obj| {
                let Object {
                    name,
                    groups
                } = obj;

                let groups = groups.into_iter().map(|x| f(x)).collect();

                Object {
                    name: name,
                    groups: groups
                }

            })
            .collect();

        Obj {
            position: position,
            texture: texture,
            normal: normal,
            objects: objects,
            materials: materials
        }
    }
}

impl<P: GenPolygon> Obj<String, P> {
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

    fn parse_group(&self, group: &str)
            -> Result<IndexTuple, String> {
        let mut group_split = group.split('/');
        let p: Option<isize> = group_split.next().and_then(|idx| FromStr::from_str(idx).ok());
        let t: Option<isize> = group_split.next().and_then(|idx| if idx != "" { FromStr::from_str(idx).ok() } else { None } );
        let n: Option<isize> = group_split.next().and_then(|idx| FromStr::from_str(idx).ok());

        match (p, t, n) {
            (Some(p), v, n) => Ok((normalize(p, self.position.len()),
                                   v.map(|v| normalize(v, self.texture.len())),
                                   n.map(|n| normalize(n, self.normal.len()))
                                 )),
            _ => Err(format!("poorly formed group {}", group))
        }
    }


    fn parse_face(&self, groups: &mut ::Words)
          -> Result<P, String>  {
        let mut ret = Vec::with_capacity(3);
        for g in groups {
          let ituple = try!(self.parse_group(g));
          ret.push(ituple);
        }
        Ok(P::new(ret))
    }

    pub fn load<B: BufRead>(input: &mut B) -> Obj<String, P> {
        let mut dat = Obj::new();
        let mut object = Object::new("default".to_string());
        let mut group: Option<Group<String, P>> = None;

        for (idx, line) in input.lines().enumerate() {
            let (line, mut words) = match line {
                Ok(ref line) => (line, words(line)),
                Err(err) => panic!("failed to readline {}", err)
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

                    let poly= match dat.parse_face(&mut words) {
                        Err(e) => panic!("Could not parse line: {}\nline: {}: {}",
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
                    group = match group {
                        Some(val) => {
                            object.groups.push(val);
                            dat.objects.push(object);
                            None
                        },
                        None => None
                    };
                    

                    object = if line.len() > 2 {
                        let name = line[1..].trim();
                        Object::new(name.to_string())
                    } else {
                        Object::new("default".to_string())
                    };
                 
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
                        let name = line[1..].trim();
                        group = Some(Group::new(name.to_string()));
                    }
                },
                Some("mtllib") => {
                    let name = words.next().expect("Failed to find name for mtllib");
                    dat.materials.push(name.to_string());
                }
                Some("usemtl") => {
                    let mut g = match group {
                        Some(g) => g,
                        None => Group::new("default".to_string())
                    };

                    // we found a new material that was applied to an existing
                    // object. It is treated as a new group.
                    if g.material.is_some() {
                        object.groups.push(g.clone());
                        g.index += 1;
                        g.indices.clear();
                    }

                    g.material = match words.next() {
                        Some(w) => Some(w.to_string()),
                        None => None
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
            None => ()
        }
        dat.objects.push(object);
        dat
    }
}
