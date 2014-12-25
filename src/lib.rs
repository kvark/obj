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

#![crate_name = "obj"]
#![crate_type = "lib"]

extern crate collections;
extern crate core;
extern crate genmesh;

use std::io::{BufferedReader, File, IoResult};
use std::collections::HashMap;
use std::rc::Rc;

pub use obj::{Obj, Object, Group, IndexTuple};
pub use mtl::{Mtl, Material};

mod obj;
mod mtl;

pub fn load(path: &Path) -> IoResult<Obj<Rc<Material>>> {
    File::open(path).map(|f| {
        let mut f = BufferedReader::new(f);
        let obj = Obj::load(&mut f);

        let mut materials = HashMap::new();

        for m in obj.materials().iter() {
            let mut p = path.clone();
            p.pop();
            p.push(m.as_slice());
            let file = File::open(&p).ok().expect("failed to open material");
            let mut f = BufferedReader::new(file);
            let m = Mtl::load(&mut f);
            for m in m.materials.into_iter() {
                materials.insert(m.name.clone(), Rc::new(m));
            }
        }

        obj.map(|g| {
            let Group {
                name,
                material,
                indices
            } = g;

            let material: Option<Rc<Material>> = match material {
                Some(m) => materials.get(&m).map(|m| m.clone()),
                None => None
            };

            Group {
                name: name,
                material: material,
                indices: indices
            }
        })
    })
}

