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

use obj::{Obj, ObjData};

#[test]
fn round_trip_sponza_no_mtls() {
    let sponza: Obj = Obj::load("test_assets/sponza.obj").unwrap();

    let mut obj = Vec::new();
    sponza.data.write_to_buf(&mut obj).unwrap();
    let sponza_round_trip = ObjData::load_buf(obj.as_slice()).unwrap();

    assert_eq!(sponza_round_trip, sponza.data);
}

#[test]
fn round_trip_sponza_with_mtl() {
    let mut sponza: Obj = Obj::load("test_assets/sponza.obj").unwrap();
    sponza.load_mtls().unwrap();

    // Write obj to string, and then load it from that string to create a round trip Obj instance.
    let mut obj = Vec::new();
    sponza.data.write_to_buf(&mut obj).unwrap();
    let mut sponza_round_trip: Obj = Obj {
        data: ObjData::load_buf(obj.as_slice()).unwrap(),
        path: sponza.path,
    };

    // Write each mtl lib to a string and load it back using load_mtls_fn into sponza_round_trip.
    let mut round_trip_mtl_libs = std::collections::HashMap::new();
    for mtl in sponza.data.material_libs.iter() {
        let mut out = Vec::new();
        mtl.write_to_buf(&mut out).unwrap();
        round_trip_mtl_libs.insert(mtl.filename.as_str(), out);
    }
    sponza_round_trip
        .load_mtls_fn(|_, mtllib| Ok(round_trip_mtl_libs.get(mtllib).unwrap().as_slice()))
        .unwrap();

    assert_eq!(sponza_round_trip.data, sponza.data);
}
