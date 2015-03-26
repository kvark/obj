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

extern crate genmesh;
extern crate obj;

use std::io::BufReader;
use obj::Obj;
use genmesh::{MapToVertices, Polygon};

static SQUARE: &'static str = "
v 0 1 0
v 0 0 0
v 1 0 0
v 1 1 0
f 1 2 3 4
";

static SQUARE_VBO: &'static [[f32; 3]] = &[
    [0., 1., 0.],
    [0., 0., 0.],
    [1., 0., 0.],
    [1., 1., 0.],
];

#[test]
fn test_load_square() {
    let mut reader = BufReader::new(SQUARE.as_bytes());
    let obj = Obj::load(&mut reader);

    let v = obj.position();

    for (a, b) in v.iter().zip(SQUARE_VBO.iter()) {
        assert_eq!(a, b);
    }

    for o in obj.object_iter() {
        for g in o.group_iter() {
            let p: Vec<Polygon<([f32;  3],[f32;  2],[f32;  3])>> =
                g.indices().iter().map(|x| *x)
                .vertex(|(p, t, n)| 
                    (
                        obj.position()[p],
                        t.map_or([0., 0.], |t| obj.texture()[t]),
                        n.map_or([1., 0., 0.,], |n| obj.normal()[n])
                    )
                )
                .collect();
            drop(p)
        }
    }

}

static CUBE: &'static str = "
v 0 1 1
v 0 0 1
v 1 0 1
v 1 1 1
v 0 1 0
v 0 0 0
v 1 0 0
v 1 1 0
# 8 vertices

o cube
g front cube
f 1 2 3 4
g back cube
f 8 7 6 5
g right cube
f 4 3 7 8
g top cube
f 5 1 4 8
g left cube
f 5 6 2 1
g bottom cube
f 2 6 7 3
# 6 elements
";

static CUBE_VBO: &'static [[f32; 3]] = &[
    [0., 1., 1.],
    [0., 0., 1.],
    [1., 0., 1.],
    [1., 1., 1.],
    [0., 1., 0.],
    [0., 0., 0.],
    [1., 0., 0.],
    [1., 1., 0.],
];

static CUBE_NAMES: &'static [&'static str] = &[
    "front cube",
    "back cube",
    "right cube",
    "top cube",
    "left cube",
    "bottom cube",
];


#[test]
fn test_load_cube() {
    let mut reader = BufReader::new(CUBE.as_bytes());
    let obj = Obj::load(&mut reader);

    let v = obj.position();

    for (a, b) in v.iter().zip(CUBE_VBO.iter()) {
        assert_eq!(a, b);
    }

    for obj in obj.object_iter() {
        assert_eq!(obj.name, "cube");
        for (g, &name) in obj.group_iter().zip(CUBE_NAMES.iter()) {
            assert_eq!(name, g.name);
        }
    }
}

static CUBE_NEGATIVE_VBO: &'static [[f32; 3]] = &[
    [0., 1., 1.],
    [0., 0., 1.],
    [1., 0., 1.],
    [1., 1., 1.],
    [1., 1., 0.],
    [1., 0., 0.],
    [0., 0., 0.],
    [0., 1., 0.],
    [1., 1., 1.],
    [1., 0., 1.],
    [1., 0., 0.],
    [1., 1., 0.],
    [0., 1., 0.],
    [0., 1., 1.],
    [1., 1., 1.],
    [1., 1., 0.],
    [0., 1., 0.],
    [0., 0., 0.],
    [0., 0., 1.],
    [0., 1., 1.],
    [0., 0., 1.],
    [0., 0., 0.],
    [1., 0., 0.],
    [1., 0., 1.],
];

static CUBE_NEGATIVE: &'static str = "
v 0 1 1
v 0 0 1
v 1 0 1
v 1 1 1
f -4 -3 -2 -1

v 1 1 0
v 1 0 0
v 0 0 0
v 0 1 0
f -4 -3 -2 -1

v 1 1 1
v 1 0 1
v 1 0 0
v 1 1 0
f -4 -3 -2 -1

v 0 1 0
v 0 1 1
v 1 1 1
v 1 1 0
f -4 -3 -2 -1

v 0 1 0
v 0 0 0
v 0 0 1
v 0 1 1
f -4 -3 -2 -1

v 0 0 1
v 0 0 0
v 1 0 0
v 1 0 1
f -4 -3 -2 -1
";

#[test]
fn test_load_cube_negative() {
    let mut reader = BufReader::new(CUBE_NEGATIVE.as_bytes());
    let obj = Obj::load(&mut reader);

    let v = obj.position();

    for (a, b) in v.iter().zip(CUBE_NEGATIVE_VBO.iter()) {
        assert_eq!(a, b);
    }
}