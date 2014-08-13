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


extern crate obj = "obj-import-rs";

use obj::ObjFile;
use std::io::BufReader;

static cube: &'static str = "
v 0 0 0
v 0 0 1
v 0 1 0
v 0 1 1
v 1 0 0
v 1 0 1
v 1 1 0
v 1 1 1

g cube
f 1 2 3 4
f 5 6 7 8
f 1 2 5 6
f 3 4 7 8
f 1 3 5 7
f 2 4 6 8
";

static cube_vbo: &'static [[f32, ..3]] = &[
    [0., 0., 0.],
    [0., 0., 1.],
    [0., 1., 0.],
    [0., 1., 1.],
    [1., 0., 0.],
    [1., 0., 1.],
    [1., 1., 0.],
    [1., 1., 1.]
];

#[test]
fn test_load_cube() {
    let mut reader = BufReader::new(cube.as_bytes());
    let obj = ObjFile::load(&mut reader);

    let (v, _) = obj.vertex_position();

    for (a, b) in v.iter().zip(cube_vbo.iter()) {
        assert_eq!(a.as_slice(), b.as_slice());
    }
}