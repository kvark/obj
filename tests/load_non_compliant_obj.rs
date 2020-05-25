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

use obj::{LoadConfig, ObjData};
use std::io::BufReader;

/// This is an example of an obj file augmented with additional custom commands.
/// We expect to be able to load the recognizable parts of these kinds of files.
static SQUARE_EXTENDED: &'static str = "
scale 1
vt 0 0
adjf 0 1
vt 1 0
adjf 0
vt 1 1
adjf 0 1
vt 0 1
adjf 1
v 0 0 0.01
ny 0 0 0
adje 0 1 3
v 1 0 0.01
ny 1 0 0
adje 1 2
v 0 0.98480775301220813 0.18364817766693034
ny 0 1 0
adje 3 4
v 1 0.98480775301220813 0.18364817766693034
ny 1 1 0
adje 0 2 4
e 4 1
e 1 2
e 2 4
e 3 1
e 4 3
f 4/3 1/1 2/2
f 3/4 1/1 4/3
";

/// This is the strictly spec compliant version of `SQUARE_EXTENDED`.
static SQUARE_STRICT: &'static str = "
vt 0 0
vt 1 0
vt 1 1
vt 0 1
v 0 0 0.01
v 1 0 0.01
v 0 0.98480775301220813 0.18364817766693034
v 1 0.98480775301220813 0.18364817766693034
f 4/3 1/1 2/2
f 3/4 1/1 4/3
";

#[test]
fn load_square_non_compliant() {
    let permissive_config = LoadConfig { strict: false };

    // Load the extended version of the square
    let mut reader = BufReader::new(SQUARE_EXTENDED.as_bytes());
    let obj_ext = ObjData::load_buf_with_config(&mut reader, permissive_config).unwrap();

    // Load the vanilla version of the square
    let mut reader = BufReader::new(SQUARE_STRICT.as_bytes());
    let obj_basic = ObjData::load_buf_with_config(&mut reader, permissive_config).unwrap();

    assert_eq!(obj_basic, obj_ext);

    let strict_config = LoadConfig { strict: true };

    let mut reader = BufReader::new(SQUARE_EXTENDED.as_bytes());
    assert!(ObjData::load_buf_with_config(&mut reader, strict_config).is_err());
}
