

use std::io::BufferedReader;
use std::io::{File, Open, Read};
use std::path::Path;
use std::from_str::FromStr;

use std::collections::HashMap;

use snowmew;
use snowmew::common::Common;
use graphics;
use graphics::geometry::{VertexGeo, VertexGeoTex, VertexGeoNorm, VertexGeoTexNorm, Geometry};

use cgmath::vector::{Vector3, Vector2};

use mtl::Mtl;
use texture::load_texture;

#[deriving(PartialEq)]
enum VertexType {
    VertexP,
    VertexPT,
    VertexPN,
    VertexPTN,
}

pub struct Obj {
    path: Path,
    vertices: Vec<Vector3<f32>>,
    textures: Vec<Vector2<f32>>,
    normals: Vec<Vector3<f32>>,
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
    objects: Vec<(String, Option<String>, uint, uint, Option<VertexType>)>,
    materials: Vec<Mtl>
}

fn lookup<'a, T: Clone>(s: &'a [T], idx: uint) -> T {
    s[idx].clone()
}

fn normalize(idx: int, len: uint) -> uint {
    if idx < 0 {
        (len as int + idx) as uint
    } else {
        idx as uint - 1
    }
}

impl Obj {
    fn new() -> Obj {
        Obj {
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
            (Some(v0), Some(v1), Some(v2)) => Vector3::new(v0, v1, v2),
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
            (Some(t0), Some(t1)) => Vector2::new(t0, t1),
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
            (Some(n0), Some(n1), Some(n2)) => Vector3::new(n0, n1, n2),
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

    pub fn load(path: &Path) -> Option<Obj> {
        let mut dat = Obj::new();

        let mut file = match File::open_mode(path, Open, Read) {
            Ok(file) => BufferedReader::new(file),
            Err(err) => {
                println!("{}", err);
                return None
            }
        };

        dat.path = path.clone();

        let mut group: Option<(String, Option<String>, uint, uint, Option<VertexType>)> = None;

        for (idx, line) in file.lines().enumerate() {
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

                    group = match group {
                        None => {
                            Some(("default".to_string(), None, start, size, None))
                        }
                        Some((name, mat, 0, 0, _)) => {
                            Some((name, mat, start, size, Some(vertex_type)))
                        }
                        Some((name, mat, start, len, vt)) => {
                            assert!(vt == Some(vertex_type));
                            Some((name, mat, start, len+size, Some(vertex_type)))
                        }
                    };
                },
                Some("o") | Some("g") => {
                    match group {
                        Some(val) => {
                            group = None;
                            dat.objects.push(val);
                        },
                        None => ()
                    }

                    match words.next() {
                        Some(name) => {
                            println!("Object {:s}", name);
                            group = Some((name.to_string(), None, 0, 0, Some(VertexP)))
                        },
                        None => ()
                    }
                },
                Some("mtllib") => {
                    let mut path = path.clone();
                    drop(path.pop());
                    let name = Path::new(words.next().expect("Failed to find name for mtllib"));
                    let path = path.join(name);
                    dat.materials.push(Mtl::load(&path).expect("Failed to load mtllib"));
                }
                Some("usemtl") => {
                    group = group.map(|(name, _, start, len, vt)| {
                        let mat = match words.next() {
                            Some(w) => Some(w.to_string()),
                            None => None
                        };
                        (name, mat, start, len, vt)
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

        Some(dat)
    }

    fn write_vbo(&self, parent: snowmew::ObjectKey, db: &mut graphics::Graphics) 
            -> (Option<snowmew::ObjectKey>, Option<snowmew::ObjectKey>,
                Option<snowmew::ObjectKey>, Option<snowmew::ObjectKey>) {
        
        let parent = db.new_object(Some(parent), "vertex_buffers");

        let vbo_p = if self.joined_vertices_p.len() != 0 {
            println!("\tvbo_p i {} ix {}",
                self.indices_p.len(),
                self.joined_vertices_p.len(),
            );
            let mut vertices = Vec::new();
            for &p in self.joined_vertices_p.iter() {
                let p = lookup(self.vertices.as_slice(), p);

                vertices.push( VertexGeo {
                    position: p
                });
            }

            let mut indices = Vec::new();
            for i in self.indices_p.iter() {
                indices.push(*i as u32);
            }

            let vb = graphics::VertexBuffer::new_position(vertices, indices);
            Some(db.new_vertex_buffer(parent, "position", vb))
        } else {None};

        let vbo_pt = if self.joined_vertices_pt.len() != 0 {
            println!("\tvbo_pt i {} ix {}",
                self.indices_pt.len(),
                self.joined_vertices_pt.len(),
            );
            let mut vertices = Vec::new();
            for &(p, t) in self.joined_vertices_pt.iter() {
                let p = lookup(self.vertices.as_slice(), p);
                let t = lookup(self.textures.as_slice(), t);

                vertices.push( VertexGeoTex {
                    position: p,
                    texture: t,
                });
            }

            let mut indices = Vec::new();
            for i in self.indices_pt.iter() {
                indices.push(*i as u32);
            }

            let vb = graphics::VertexBuffer::new_position_texture(vertices, indices);
            Some(db.new_vertex_buffer(parent, "position_texture", vb))
        } else {None};

        let vbo_pn = if self.joined_vertices_pn.len() != 0 {
            println!("\tvbo_pn i {} ix {}",
                self.indices_pn.len(),
                self.joined_vertices_pn.len(),
            );
            let mut vertices = Vec::new();
            for &(p, n) in self.joined_vertices_pn.iter() {
                let p = lookup(self.vertices.as_slice(), p);
                let n = lookup(self.normals.as_slice(), n);

                vertices.push( VertexGeoNorm {
                    position: p,
                    normal: n,
                });
            }

            let mut indices = Vec::new();
            for i in self.indices_pn.iter() {
                indices.push(*i as u32);
            }

            let vb = graphics::VertexBuffer::new_position_normal(vertices, indices);
            Some(db.new_vertex_buffer(parent, "position_normal", vb))
        } else {None};

        let vbo_ptn = if self.joined_vertices_ptn.len() != 0 {
            println!("\tvbo_ptn i {} ix {}",
                self.indices_ptn.len(),
                self.joined_vertices_ptn.len(),
            );
            let mut vertices = Vec::new();
            for &(p, t, n) in self.joined_vertices_ptn.iter() {
                let p = lookup(self.vertices.as_slice(), p);
                let t = lookup(self.textures.as_slice(), t);
                let n = lookup(self.normals.as_slice(), n);

                vertices.push( VertexGeoTexNorm {
                    position: p,
                    texture: t,
                    normal: n
                });
            }

            let mut indices = Vec::new();
            for i in self.indices_ptn.iter() {
                indices.push(*i as u32);
            }

            let vb = graphics::VertexBuffer::new_position_texture_normal(vertices, indices);
            Some(db.new_vertex_buffer(parent, "position_texture_normal", vb))
        } else {None};

        (vbo_p, vbo_pt, vbo_pn, vbo_ptn)
    }

    fn write_textures(&self, parent: snowmew::ObjectKey, db: &mut graphics::Graphics)
            -> HashMap<String, snowmew::ObjectKey> {
        let parent = db.new_object(Some(parent), "textures");
        let mut map = HashMap::new();
        for m_dir in self.materials.iter() {
            for m in m_dir.materials.iter() {
                let text = &[&m.map_ka, &m.map_kd, &m.map_ks, &m.map_ke];
                for t in text.iter() {
                    match *t {
                        &None => (),
                        &Some(ref t) => {
                            let insert = map.find(t).is_none();
                            if insert {
                                let mut path = self.path.clone();
                                drop(path.pop());
                                let text = load_texture(&path.join(&Path::new(t.clone())));
                                let id = db.new_texture(parent, t.as_slice(), text);
                                map.insert(t.clone(), id);
                            }
                        }
                    }
                }
            }
        }
        map
    }

    fn write_materials(&self,
                       parent: snowmew::ObjectKey,
                       db: &mut graphics::Graphics,
                       text: &HashMap<String, snowmew::ObjectKey>)
            -> HashMap<String, snowmew::ObjectKey> {

        let mut name_to_id = HashMap::new();

        let lookup = |name| {
            *text.find(name).expect("texture not found")
        };

        let parent = db.new_object(Some(parent), "materials");
        for m_dir in self.materials.iter() {
            for m in m_dir.materials.iter() {
                let mut mat = graphics::Material::new();
                if m.ka.is_some() { mat.set_ka(*m.ka.as_ref().unwrap()); }
                if m.kd.is_some() { mat.set_kd(*m.kd.as_ref().unwrap()); }
                if m.ks.is_some() { mat.set_ks(*m.ks.as_ref().unwrap()); }
            if m.ke.is_some() { mat.set_ke(*m.ke.as_ref().unwrap()); }
                if m.ni.is_some() { mat.set_ni(*m.ni.as_ref().unwrap()); }
                if m.ns.is_some() { mat.set_ns(*m.ns.as_ref().unwrap()); }
                if m.map_ka.is_some() { mat.set_map_ka(lookup(m.map_ka.as_ref().unwrap())); }
                if m.map_kd.is_some() { mat.set_map_kd(lookup(m.map_kd.as_ref().unwrap())); }
                if m.map_ks.is_some() { mat.set_map_ks(lookup(m.map_ks.as_ref().unwrap())); }
                if m.map_ke.is_some() { mat.set_map_ke(lookup(m.map_ke.as_ref().unwrap())); }
                let id = db.new_material(parent, m.name.as_slice(), mat);
                name_to_id.insert(m.name.clone(), id);
            }
        }

        name_to_id
    }


    pub fn import(&self, parent: snowmew::ObjectKey, db: &mut graphics::Graphics) {
        println!("v {} t {} n {}",
            self.vertices.len(),
            self.textures.len(),
            self.normals.len()
        );

        let textures = self.write_textures(parent, db);
        let materials = self.write_materials(parent, db, &textures);
        let (vbo_p, vbo_pt, vbo_pn, vbo_ptn) = self.write_vbo(parent, db);
        let geometry = db.add_dir(Some(parent), "geometry");
        let objects = db.add_dir(Some(parent), "objects");
        for &(ref name, ref mat, start, len, vt) in self.objects.iter() {
            println!("{} {}", name, mat);
            let vbo = match vt {
                Some(VertexP) => vbo_p,
                Some(VertexPN) => vbo_pn,
                Some(VertexPT) => vbo_pt,
                Some(VertexPTN) => vbo_ptn,
                None => None
            };
            match vbo {
                None => (),
                Some(vbo) => {
                    let geo = db.new_geometry(geometry, name.as_slice(), Geometry::triangles(vbo, start, len));
                    if mat.is_some() {
                        let mat = materials.find(mat.as_ref().unwrap());
                        if mat.is_some() {
                            let obj = db.new_object(Some(objects), name.as_slice());
                            db.set_draw(obj, geo, *mat.unwrap());
                        }
                    }
                }
            }

        }

    }
}