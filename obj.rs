
use std::io::BufferedReader;
use std::io::{File, Open, Read};
use std::path::Path;
use std::from_str::FromStr;

use collections::HashMap;

use snowmew;
use snowmew::core::Common;
use snowmew::geometry::{VertexGetTexNorm, Geometry};

use cgmath::vector::{Vector3, Vector2};

pub struct Obj
{
    pub vertices: Vec<Vector3<f32>>,
    pub textures: Vec<Vector2<f32>>,
    pub normals: Vec<Vector3<f32>>,
    pub joined_vertices: Vec<(uint, uint, uint)>,
    pub joined_vertices_map: HashMap<(uint, uint, uint), uint>,
    pub indices: Vec<uint>,
    pub objects: Vec<(~str, uint, uint)>

}

impl Obj
{
    fn new() -> Obj
    {
        Obj {
            vertices: Vec::new(),
            textures: Vec::new(),
            normals: Vec::new(),
            joined_vertices: Vec::new(),
            joined_vertices_map: HashMap::new(),
            indices: Vec::new(),
            objects: Vec::new()
        }
    }

    fn parse_vertex(&mut self, v0: Option<&str>, v1: Option<&str>, v2: Option<&str>)
    {
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

    fn parse_texture(&mut self, t0: Option<&str>, t1: Option<&str>)
    {
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

    fn parse_normal(&mut self, n0: Option<&str>, n1: Option<&str>, n2: Option<&str>)
    {
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

    fn parse_group(&mut self, group: &str) -> uint
    {
        let mut group_split = group.split('/');
        let v = group_split.next();
        let t = group_split.next();
        let n = group_split.next();
        let v: uint = FromStr::from_str(v.unwrap()).unwrap();
        let t: uint = FromStr::from_str(t.unwrap()).unwrap();
        let n: uint = FromStr::from_str(n.unwrap()).unwrap();

        let vg = (v, t, n);
        match self.joined_vertices_map.find(&vg) {
            Some(&idx) => {
                return idx;
            },
            None => {
                let len = self.joined_vertices.len();
                self.joined_vertices_map.insert(vg, len);
                self.joined_vertices.push(vg);
                return len;
            }
        }
    }

    fn parse_triangle(&mut self, g0: &str, g1: &str, g2: &str) -> uint
    {
        let g0 = self.parse_group(g0);
        let g1 = self.parse_group(g1);
        let g2 = self.parse_group(g2);

        self.indices.push(g0);
        self.indices.push(g1);
        self.indices.push(g2);

        3
    }

    fn parse_face(&mut self, g0: Option<&str>, g1: Option<&str>, g2: Option<&str>, g3: Option<&str>) -> uint
    {
        match (g0, g1, g2, g3) {
            (Some(g0), Some(g1), Some(g2), None) => self.parse_triangle(g0, g1, g2),
            _ => {fail!("Unsupported");}
        }
    }

    pub fn load(path: &Path) -> Option<Obj>
    {
        let mut dat = Obj::new();

        let mut file = match File::open_mode(path, Open, Read) {
            Ok(file) => BufferedReader::new(file),
            Err(err) => {
                println!("{}", err);
                return None
            }
        };

        let mut group: Option<(~str, uint, uint)> = None;

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
                    let size = dat.parse_face(g0, g1, g2, g3);

                    match group {
                        Some((name, start, len)) => {
                            group = Some((name, start, len+size));
                        }
                        None => {
                            group = Some(("default".to_owned(), dat.indices.len()-size, size))
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
                            group = Some((name.to_owned(), dat.indices.len(), 0))
                        },
                        None => ()
                    }
                },
                Some("mtllib") | Some("usemtl") | Some("s") => (),
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

    pub fn import(&self, parent: snowmew::ObjectKey, db: &mut snowmew::graphics::Graphics)
    {
        println!("v {} t {} n {} i {} ix {}\n",
            self.vertices.len(),
            self.textures.len(),
            self.normals.len(),
            self.joined_vertices.len(),
            self.joined_vertices_map.len()
        );

        // build vertex buffer
        let mut vertices = Vec::new();
        for &(v, t, n) in self.joined_vertices.iter() {
            let v = *self.vertices.get(v-1);
            let t = *self.textures.get(t-1);
            let n = *self.normals.get(n-1);

            vertices.push( VertexGetTexNorm {
                position: v,
                texture: t,
                normal: n
            });

        }

        let mut indices = Vec::new();
        for i in self.indices.iter() {
            indices.push(*i as u32);
        }

        let vb = snowmew::VertexBuffer::new_position_texture_normal(vertices, indices);
        let vbo = db.new_vertex_buffer(parent, "vbo", vb);

        let geometry = db.add_dir(Some(parent), "geometry");

        for &(ref name, start, len) in self.objects.iter() {
            db.new_geometry(geometry, name.clone(), Geometry::triangles(vbo, start, len));
        }
    }
}