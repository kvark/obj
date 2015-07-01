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

extern crate genmesh;

use std::fs::File;
use std::path::Path;
use std::io::{self, BufReader};
use std::collections::HashMap;
use std::rc::Rc;
use std::iter::Filter;
use std::str::Split;

pub use obj::{Obj, Object, Group, IndexTuple};
pub use mtl::{Mtl, Material};

mod obj;
mod mtl;

pub fn load(path: &Path) -> io::Result<Obj<Rc<Material>>> {
    File::open(path).map(|f| {
        let mut f = BufReader::new(f);
        let obj = Obj::load(&mut f);

        let mut materials = HashMap::new();

        for m in obj.materials().iter() {
            let mut p = path.to_path_buf();
            p.pop();
            p.push(m);
            let file = File::open(&p).ok().expect("failed to open material");
            let mut f = BufReader::new(file);
            let m = Mtl::load(&mut f);
            for m in m.materials.into_iter() {
                materials.insert(m.name.clone(), Rc::new(m));
            }
        }

        obj.map(|g| {
            let Group {
                name,
                index,
                material,
                indices
            } = g;

            let material: Option<Rc<Material>> = match material {
                Some(m) => materials.get(&m).map(|m| m.clone()),
                None => None
            };

            Group {
                name: name,
                index: index,
                material: material,
                indices: indices
            }
        })
    })
}


type Words<'a> = Filter<Split<'a, fn(char) -> bool>, fn(&&str) -> bool>;

fn words<'a>(s: &'a str) -> Words<'a> {
    fn is_not_empty(s: &&str) -> bool { !s.is_empty() }
    let is_not_empty: fn(&&str) -> bool = is_not_empty; // coerce to fn pointer

    fn is_whitespace(c: char) -> bool { c.is_whitespace() }
    let is_whitespace: fn(char) -> bool = is_whitespace; // coerce to fn pointer!s.is_empty())

    s.split(is_whitespace).filter(is_not_empty)
}
