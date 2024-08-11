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

use obj::ObjData;
use std::io::BufReader;

static SQUARE: &'static str = "
v 0 0 0
v 1 1 1
v 1 0 1
v 0 1 0
usemtl test
g group_a
f 1 2 3
g group_b
f 1 4 2
";

#[test]
fn material_use_persistence() {
    let mut reader = BufReader::new(SQUARE.as_bytes());
    let obj_data = ObjData::load_buf(&mut reader).unwrap();

    // once the 'usemtl' statement is set, it applies to all
    // elements that follow until it is reset to a different value.
    let mut is_material_set = false;
    for obj in obj_data.objects.iter() {
        for group in obj.groups.iter() {
            if group.material.is_some() {
                is_material_set = true;
            } else {
                assert!(!is_material_set);
            }
        }
    }
}
