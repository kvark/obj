#![crate_id = "github.com/csherratt/snowmew#snowmew-loader:0.1"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "An asset loader for snowmew"]

extern crate core;
extern crate snowmew;
extern crate cgmath;
extern crate collections;
extern crate graphics = "snowmew-graphics";

pub use obj::Obj;

mod obj;
mod mtl;