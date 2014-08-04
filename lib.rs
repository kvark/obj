#![crate_name = "snowmew-loader"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "An asset loader for snowmew"]

extern crate debug;
extern crate collections;
extern crate core;

extern crate cgmath;
extern crate image = "stb_image";

extern crate snowmew  = "snowmew-core";
extern crate graphics = "snowmew-graphics";

pub use obj::Obj;

mod obj;
mod mtl;
mod texture;