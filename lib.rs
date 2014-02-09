#[crate_id = "github.com/csherratt/snowmew#snowmew-loader:0.1"];
#[license = "ASL2"];
#[crate_type = "lib"];
#[comment = "An asset loader for snowmew"];

extern mod snowmew;
extern mod cgmath;

pub use obj::Obj;

mod obj;
