
use std::io::BufferedReader;
use std::io::{File, Open, Read};
use std::path::Path;
use std::from_str::FromStr;

use collections::HashMap;

use snowmew;
use snowmew::common::Common;
use graphics;
use graphics::geometry::{VertexGeo, VertexGeoTex, VertexGeoNorm, VertexGeoTexNorm, Geometry};

use cgmath::vector::{Vector3, Vector2};

use mtl::Mtl;

#[deriving(Eq)]
enum VertexType {
    VertexP,
    VertexPT,
    VertexPN,
    VertexPTN,
}

pub struct Obj {
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
    objects: Vec<(~str, Option<~str>, uint, uint, Option<VertexType>)>,
    materials: Vec<Mtl>
}

impl Obj {
    fn new() -> Obj {
        Obj {
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

    fn parse_group_p(&mut self, p: &str) -> uint {
        let p: uint = FromStr::from_str(p).expect("Invalid number");
        match self.joined_vertices_map_p.find(&p) {
            Some(&idx) => {
                return idx;
            },
            None => {
                let len = self.joined_vertices_p.len();
                self.joined_vertices_map_p.insert(p, len);
                self.joined_vertices_p.push(p);
                return len;
            }
        }
    }

    fn parse_group_pt(&mut self, p: &str, t: &str) -> uint {
        let p: uint = FromStr::from_str(p).expect("Invalid number");
        let t: uint = FromStr::from_str(t).expect("Invalid number");

        let pt = (p, t);
        match self.joined_vertices_map_pt.find(&pt) {
            Some(&idx) => {
                return idx;
            },
            None => {
                let len = self.joined_vertices_pt.len();
                self.joined_vertices_map_pt.insert(pt, len);
                self.joined_vertices_pt.push(pt);
                return len;
            }
        }
    }

    fn parse_group_pn(&mut self, p: &str, n: &str) -> uint {
        let p: uint = FromStr::from_str(p).expect("Invalid number");
        let n: uint = FromStr::from_str(n).expect("Invalid number");

        let pn = (p, n);
        match self.joined_vertices_map_pn.find(&pn) {
            Some(&idx) => {
                return idx;
            },
            None => {
                let len = self.joined_vertices_pn.len();
                self.joined_vertices_map_pn.insert(pn, len);
                self.joined_vertices_pn.push(pn);
                return len;
            }
        }
    }

    fn parse_group_ptn(&mut self, p: &str, t: &str, n: &str) -> uint {
        let p: uint = FromStr::from_str(p).expect("Invalid number");
        let t: uint = FromStr::from_str(t).expect("Invalid number");
        let n: uint = FromStr::from_str(n).expect("Invalid number");

        let ptn = (p, t, n);
        match self.joined_vertices_map_ptn.find(&ptn) {
            Some(&idx) => {
                return idx;
            },
            None => {
                let len = self.joined_vertices_ptn.len();
                self.joined_vertices_map_ptn.insert(ptn, len);
                self.joined_vertices_ptn.push(ptn);
                return len;
            }
        }
    }

    fn parse_group(&mut self, group: &str) -> (VertexType, uint) {
        let mut group_split = group.split('/');
        let p = group_split.next();
        let t = group_split.next();
        let n = group_split.next();

        match (p, t, n) {
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
                (VertexPTN, self.parse_group_ptn(p, t, n))
            },
            _ => fail!("poorly formed group {:s}", group)
        }
    }

    fn parse_triangle(&mut self, g0: &str, g1: &str, g2: &str) -> (VertexType, uint, uint) {
        let (t, g0) = self.parse_group(g0);
        let (_, g1) = self.parse_group(g1);
        let (_, g2) = self.parse_group(g2);

        let indices = match t {
            VertexP => &mut self.indices_p,
            VertexPT => &mut self.indices_pt,
            VertexPN => &mut self.indices_pn,
            VertexPTN => &mut self.indices_ptn,
        };

        let start = indices.len();
        indices.push(g0);
        indices.push(g1);
        indices.push(g2);

        (t, start, 3)
    }

    fn parse_quad(&mut self, g0: &str, g1: &str, g2: &str, g3: &str) -> (VertexType, uint, uint) {
        let (t, g0) = self.parse_group(g0);
        let (_, g1) = self.parse_group(g1);
        let (_, g2) = self.parse_group(g2);
        let (_, g3) = self.parse_group(g3);

        let indices = match t {
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

        (t, start, 6)
    }

    fn parse_face(&mut self, g0: Option<&str>, g1: Option<&str>, g2: Option<&str>, g3: Option<&str>) -> (VertexType, uint, uint) {
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

        let mut group: Option<(~str, Option<~str>, uint, uint, Option<VertexType>)> = None;

        for line in file.lines() {
            let mut words = match line {
                Ok(ref line) => line.words(),
                Err(err) => fail!("failed to readline {:?}", err)
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
                    let (vertex_type, start, size) = dat.parse_face(g0, g1, g2, g3);

                    match group {
                        None => {
                            group = Some(("default".to_owned(), None, start, size, None))
                        }
                        Some((name, mat, 0, 0, _)) => {
                            group = Some((name, mat, start, size, Some(vertex_type)))
                        }
                        Some((name, mat, start, len, vt)) => {
                            assert!(vt == Some(vertex_type));
                            group = Some((name, mat, start, len+size, Some(vertex_type)));
                        }
                    }
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
                            group = Some((name.to_owned(), None, 0, 0, Some(VertexP)))
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
                    match group {
                        None => {}
                        Some((name, _, start, len, vt)) => {
                            let mat = match  words.next() {
                                Some(w) => Some(w.to_owned()),
                                None => None
                            };
                            group = Some((name, mat, start, len, vt));
                        }
                    }
                }
                Some("s") => (),
                Some(other) => {
                    if other.len() != 0 && other[0] != "#"[0] {
                        fail!("Invalid token {} {:?}", other, words.next());
                    }
                }
                None => (),
            }

        }

        if group.is_some() {
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
            for p in self.joined_vertices_p.iter() {
                let p = *self.vertices.get(p-1);

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
                let p = *self.vertices.get(p-1);
                let t = *self.textures.get(t-1);

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
                let p = *self.vertices.get(p-1);
                let n = *self.normals.get(n-1);

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
                let p = *self.vertices.get(p-1);
                let t = *self.textures.get(t-1);
                let n = *self.normals.get(n-1);

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


    fn write_materials(&self, parent: snowmew::ObjectKey, db: &mut graphics::Graphics)
            -> HashMap<~str, snowmew::ObjectKey> {

        let mut name_to_id = HashMap::new();

        let parent = db.new_object(Some(parent), "materials");
        for m_dir in self.materials.iter() {
            for m in m_dir.materials.iter() {
                let mut mat = graphics::Material::new();
                if m.ka.is_some() { mat.set_Ka(*m.ka.as_ref().unwrap()); }
                if m.kd.is_some() { mat.set_Kd(*m.kd.as_ref().unwrap()); }
                if m.ks.is_some() { mat.set_Ks(*m.ks.as_ref().unwrap()); }
                if m.ke.is_some() { mat.set_Ke(*m.ke.as_ref().unwrap()); }

                let id = db.new_material(parent, m.name, mat);
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

        let materials = self.write_materials(parent, db);
        let (vbo_p, vbo_pt, vbo_pn, vbo_ptn) = self.write_vbo(parent, db);
        let geometry = db.add_dir(Some(parent), "geometry");
        let objects = db.add_dir(Some(parent), "objects");
        for &(ref name, ref mat, start, len, vt) in self.objects.iter() {
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
                    let geo = db.new_geometry(geometry, name.clone(), Geometry::triangles(vbo, start, len));
                    if mat.is_some() {
                        let mat = materials.find(mat.as_ref().unwrap());
                        if mat.is_some() {
                            let obj = db.new_object(Some(objects), name.clone());
                            db.set_draw(obj, geo, *mat.unwrap());
                        }
                    }
                }
            }

        }



    }
}