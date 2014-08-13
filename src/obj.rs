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

use std::io::BufferedReader;
use std::io::{File, Open, Read};
use std::path::Path;
use std::from_str::FromStr;

use std::collections::HashMap;

use mtl::Mtl;

#[deriving(PartialEq, Show)]
pub enum VertexType {
    VertexP,
    VertexPT,
    VertexPN,
    VertexPTN,
}

pub struct Object {
    name: String,
    material: Option<String>,
    start: uint,
    length: uint,
    vertex_type: VertexType
}

impl Object {
    fn new(name: String) -> Object {
        Object {
            name: name,
            material: None,
            start: 0,
            length: 0,
            vertex_type: VertexP
        }
    }
}

pub struct ObjFile {
    path: Path,
    vertices: Vec<[f32, ..3]>,
    textures: Vec<[f32, ..2]>,
    normals: Vec<[f32, ..3]>,
    joined_vertices_map_p: HashMap<uint, uint>,
    joined_vertices_map_pn: HashMap<(uint, uint), uint>,
    joined_vertices_map_pt: HashMap<(uint, uint), uint>,
    joined_vertices_map_ptn: HashMap<(uint, uint, uint), uint>,
    joined_vertices_p: Vec<uint>,
    joined_vertices_pn: Vec<(uint, uint)>,
    joined_vertices_pt: Vec<(uint, uint)>,
    joined_vertices_ptn: Vec<(uint, uint, uint)>,
    indices_p: Vec<uint>,
    indices_pn: Vec<uint>,
    indices_pt: Vec<uint>,
    indices_ptn: Vec<uint>,
    objects: Vec<Object>,
    materials: Vec<Mtl>
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
            vertices: Vec::new(),
            textures: Vec::new(),
            normals: Vec::new(),
            joined_vertices_p: Vec::new(),
            joined_vertices_pn: Vec::new(),
            joined_vertices_pt: Vec::new(),
            joined_vertices_ptn: Vec::new(),
            joined_vertices_map_p: HashMap::new(),
            joined_vertices_map_pn: HashMap::new(),
            joined_vertices_map_pt: HashMap::new(),
            joined_vertices_map_ptn: HashMap::new(),
            indices_p: Vec::new(),
            indices_pn: Vec::new(),
            indices_pt: Vec::new(),
            indices_ptn: Vec::new(),
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
        self.vertices.push(vertex);
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
        self.textures.push(texture);
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
        self.normals.push(normal);
    }

    fn parse_group_p(&mut self, p: &str) -> Result<uint, String> {
        let p = match FromStr::from_str(p) {
            Some(p) => {
                normalize(p, self.vertices.len())
            },
            None => return Err(format!("'{}'' could not be converted to int", p))
        };

        match self.joined_vertices_map_p.find(&p) {
            Some(&idx) => {
                return Ok(idx);
            },
            None => {
                let len = self.joined_vertices_p.len();
                self.joined_vertices_map_p.insert(p, len);
                self.joined_vertices_p.push(p);
                return Ok(len);
            }
        }
    }

    fn parse_group_pt(&mut self, p: &str, t: &str) -> Result<uint, String> {
        let pt = match (FromStr::from_str(p), FromStr::from_str(t)) {
            (Some(p), Some(t)) => {
                let p = normalize(p, self.vertices.len());
                let t = normalize(t, self.textures.len());
                (p, t)
            },
            (None, _) => return Err(format!("'{}' could not be converted to int", p)),
            (_, None) => return Err(format!("'{}' could not be converted to int", t))
        };

        match self.joined_vertices_map_pt.find(&pt) {
            Some(&idx) => {
                return Ok(idx);
            },
            None => {
                let len = self.joined_vertices_pt.len();
                self.joined_vertices_map_pt.insert(pt, len);
                self.joined_vertices_pt.push(pt);
                return Ok(len);
            }
        }
    }

    fn parse_group_pn(&mut self, p: &str, n: &str) -> Result<uint, String> {
        let pn = match (FromStr::from_str(p), FromStr::from_str(n)) {
            (Some(p), Some(n)) => {
                let p = normalize(p, self.vertices.len());
                let n = normalize(n, self.normals.len());
                (p, n)
            },
            (None, _) => return Err(format!("'{}' could not be converted to int", p)),
            (_, None) => return Err(format!("'{}' could not be converted to int", n))
        };

        match self.joined_vertices_map_pn.find(&pn) {
            Some(&idx) => {
                return Ok(idx);
            },
            None => {
                let len = self.joined_vertices_pn.len();
                self.joined_vertices_map_pn.insert(pn, len);
                self.joined_vertices_pn.push(pn);
                return Ok(len);
            }
        }
    }

    fn parse_group_ptn(&mut self, p: &str, t: &str, n: &str) -> Result<uint, String> {
        let ptn = match (FromStr::from_str(p), FromStr::from_str(t), FromStr::from_str(n)) {
            (Some(p), Some(t), Some(n)) => {
                let p = normalize(p, self.vertices.len());
                let t = normalize(t, self.textures.len());
                let n = normalize(n, self.normals.len());
                (p, t, n)
            },
            (None, _, _) => return Err(format!("'{}' could not be converted to int", p)),
            (_, None, _) => return Err(format!("'{}' could not be converted to int", t)),
            (_, _, None) => return Err(format!("'{}' could not be converted to int", n))
        };


        match self.joined_vertices_map_ptn.find(&ptn) {
            Some(&idx) => {
                return Ok(idx);
            },
            None => {
                let len = self.joined_vertices_ptn.len();
                self.joined_vertices_map_ptn.insert(ptn, len);
                self.joined_vertices_ptn.push(ptn);
                return Ok(len);
            }
        }
    }

    fn parse_group(&mut self, group: &str) -> Result<(VertexType, uint), String> {
        let mut group_split = group.split('/');
        let p = group_split.next();
        let t = group_split.next();
        let n = group_split.next();

        let (vt, res) = match (p, t, n) {
            (Some(p), None, None) => {
                (VertexP, self.parse_group_p(p))
            }
            (Some(p), Some(t), None) => {
                (VertexPT, self.parse_group_pt(p, t))
            }
            (Some(p), None, Some(n)) => {
                (VertexPN, self.parse_group_pn(p, n))
            }
            (Some(p), Some(t), Some(n)) => {
                if t == "" {
                    (VertexPN, self.parse_group_pn(p, n))
                } else {
                    (VertexPTN, self.parse_group_ptn(p, t, n))
                }
            },
            _ => fail!("poorly formed group {:s}", group)
        };

        match res {
            Ok(idx) => Ok((vt, idx)),
            Err(err) => Err(err)
        }
    }

    fn parse_triangle(&mut self, g0: &str, g1: &str, g2: &str) 
            -> Result<(VertexType, uint, uint), String> {
        let g0 = self.parse_group(g0);
        let g1 = self.parse_group(g1);
        let g2 = self.parse_group(g2);

        match (g0, g1, g2) {
            (Ok((t0, g0)), Ok((t1, g1)), Ok((t2, g2))) => {
                if t0 == t1 && t1 == t2 {
                    let indices = match t0 {
                        VertexP => &mut self.indices_p,
                        VertexPT => &mut self.indices_pt,
                        VertexPN => &mut self.indices_pn,
                        VertexPTN => &mut self.indices_ptn,
                    };
                    let start = indices.len();
                    indices.push(g0);
                    indices.push(g1);
                    indices.push(g2);
                    Ok((t0, start, 3))
                } else {
                    Err("Group type does not match".to_string())
                }
            }
            (Err(e), _, _) => { Err(e) }
            (_, Err(e), _) => { Err(e) }
            (_, _, Err(e)) => { Err(e) }
        }
        
    }

    fn parse_quad(&mut self, g0: &str, g1: &str, g2: &str, g3: &str) 
            -> Result<(VertexType, uint, uint), String> {
        let g0 = self.parse_group(g0);
        let g1 = self.parse_group(g1);
        let g2 = self.parse_group(g2);
        let g3 = self.parse_group(g3);

        match (g0, g1, g2, g3) {
            (Ok((t0, g0)), Ok((t1, g1)), Ok((t2, g2)), Ok((t3, g3))) => {
                if t0 == t1 && t1 == t2 && t2 == t3 {
                    let indices = match t0 {
                        VertexP => &mut self.indices_p,
                        VertexPT => &mut self.indices_pt,
                        VertexPN => &mut self.indices_pn,
                        VertexPTN => &mut self.indices_ptn,
                    };
                    let start = indices.len();
                    indices.push(g0);
                    indices.push(g1);
                    indices.push(g2);
                    indices.push(g2);
                    indices.push(g3);
                    indices.push(g0);
                    Ok((t0, start, 6))
                } else {
                    Err("Group type does not match".to_string())
                }
            }
            (Err(e), _, _, _) => { Err(e) }
            (_, Err(e), _, _) => { Err(e) }
            (_, _, Err(e), _) => { Err(e) }
            (_, _, _, Err(e)) => { Err(e) }
        }
    }

    fn parse_face(&mut self, g0: Option<&str>, g1: Option<&str>, g2: Option<&str>, g3: Option<&str>) 
            -> Result<(VertexType, uint, uint), String> {
        match (g0, g1, g2, g3) {
            (Some(g0), Some(g1), Some(g2), None) => self.parse_triangle(g0, g1, g2),
            (Some(g0), Some(g1), Some(g2), Some(g3)) => self.parse_quad(g0, g1, g2, g3),
            _ => {fail!("Unsupported");}
        }
    }

    pub fn load<B: Buffer>(input: &mut B) -> ObjFile {
        let mut dat = ObjFile::new();
        let mut group: Option<Object> = None;

        for (idx, line) in input.lines().enumerate() {
            let mut words = match line {
                Ok(ref line) => line.as_slice().words(),
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

                    let (vertex_type, start, size) = match dat.parse_face(g0, g1, g2, g3) {
                        Err(e) => fail!("Could not parse line: {}\nline: {}: {}",
                            e, idx, line
                        ),
                        Ok((a, b, c)) => (a, b, c)
                    };

                    group = Some(match group {
                        None => {
                            let mut obj = Object::new("default".to_string());
                            obj.vertex_type = vertex_type;
                            obj.start = start;
                            obj.length = size;
                            obj
                        }
                        Some(mut obj) => {
                            if obj.length == 0 {
                                obj.vertex_type = vertex_type;
                                obj.start = start;
                                obj.length = size;
                            } else {
                                obj.length += size;
                            }
                            obj
                        }
                    });
                },
                Some("o") | Some("g") => {
                    group = match group {
                        Some(val) => {
                            dat.objects.push(val);
                            None
                        },
                        None => None
                    };

                    match words.next() {
                        Some(name) => {
                            println!("Object {:s}", name);
                            group = Some(Object {
                                name: name.to_string(),
                                material: None,
                                start: 0,
                                length: 0,
                                vertex_type: VertexP
                            });
                        },
                        None => ()
                    }
                },
                /*Some("mtllib") => {
                    let mut path = path.clone();
                    drop(path.pop());
                    let name = Path::new(words.next().expect("Failed to find name for mtllib"));
                    let path = path.join(name);
                    dat.materials.push(Mtl::load(&path).expect("Failed to load mtllib"));
                }*/
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

        if group.is_some() {
            println!("group push {:?}", group);
            dat.objects.push(group.unwrap());
        }

        dat
    }

    pub fn vertex_position(&self)
            -> (Vec<[f32, ..3]>, Vec<u32>) {

        let vertices = self.joined_vertices_p.iter()
                                             .map(|&i| self.vertices[i])
                                             .collect();

        let indices = self.indices_p.iter()
                                    .map(|&i| i as u32)
                                    .collect();

        (vertices, indices)
    }

    pub fn vertex_position_texture(&self)
            -> (Vec<([f32, ..3], [f32, ..2])>, Vec<u32>) {

        let vertices = self.joined_vertices_pt.iter()
                                              .map(|&(p, t)| (self.vertices[p],
                                                              self.textures[t]))
                                              .collect();

        let indices = self.indices_pt.iter()
                                     .map(|&i| i as u32)
                                     .collect();

        (vertices, indices)
    }

    pub fn vertex_position_normal(&self)
            -> (Vec<([f32, ..3], [f32, ..3])>, Vec<u32>) {

        let vertices = self.joined_vertices_pn.iter()
                                              .map(|&(p, n)| (self.vertices[p],
                                                              self.normals[n]))
                                              .collect();

        let indices = self.indices_pn.iter()
                                     .map(|&i| i as u32)
                                     .collect();

        (vertices, indices)
    }

    pub fn vertex_position_texture_normal(&self) 
            -> (Vec<([f32, ..3], [f32, ..2], [f32, ..3])>, Vec<u32>) {

        let vertices = self.joined_vertices_ptn.iter()
                                               .map(|&(p, t, n)| (self.vertices[p],
                                                                  self.textures[t],
                                                                  self.normals[n]))
                                               .collect();

        let indices = self.indices_pn.iter()
                                     .map(|&i| i as u32)
                                     .collect();

        (vertices, indices)
    }
}