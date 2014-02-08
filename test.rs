extern mod loader = "snowmew-loader";

use loader::ObjLoader;

#[test]
fn load_teapot()
{
    let teapot = ObjLoader::load(&Path::new("assets/teapot.obj"));

    assert!(teapot.is_some());

    let teapot = teapot.unwrap();

    for &(ref name, _, _) in teapot.objects.iter() {
        assert!(name.as_slice() == "Teapot01");
    }
}