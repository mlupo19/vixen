use glium::backend::Facade;
use glium::Program;

pub fn load_shader<F: ?Sized + Facade>(name: &str, display: &F) -> Program {
    const PATH: &str = "src/shaders/";
    let mut path = String::from(PATH);
    path.push_str(name);
    path.push_str("/");

    let vertex_shader_src =
        std::fs::read_to_string(path.clone() + "vertex.glsl").expect("Unable to read vertex.glsl");
    let fragment_shader_src =
        std::fs::read_to_string(path + "frag.glsl").expect("Unable to read frag.glsl");

    Program::from_source(display, &vertex_shader_src, &fragment_shader_src, None).unwrap()
}
