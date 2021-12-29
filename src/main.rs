#[macro_use]
extern crate glium;

mod camera;
mod chunk;
mod input;
mod loader;
mod player;
mod terrain;

use chunk::Chunk;
use std::io::Cursor;

fn main() {
    use glium::{glutin, Surface};

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Vixen")
        .with_inner_size(glium::glutin::dpi::PhysicalSize {
            width: 1920,
            height: 1080,
        })
        .with_position(glium::glutin::dpi::PhysicalPosition { x: 0, y: 0 });
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let mut chunk_loader = loader::ChunkLoader::new(0);

    let image = image::load(
        Cursor::new(&include_bytes!("../res/diffuse.jpg")),
        image::ImageFormat::Jpeg,
    )
    .unwrap()
    .to_rgba8();

    let image_dimensions = image.dimensions();
    let image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let diffuse_texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

    let image = image::load(
        Cursor::new(&include_bytes!("../res/normal.png")),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let image_dimensions = image.dimensions();
    let image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let normal_map = glium::texture::Texture2d::new(&display, image).unwrap();

    let vertex_shader_src =
        std::fs::read_to_string("src/shaders/vertex.glsl").expect("Unable to read vertex.glsl");
    let fragment_shader_src =
        std::fs::read_to_string("src/shaders/frag.glsl").expect("Unable to read frag.glsl");

    let program =
        glium::Program::from_source(&display, &vertex_shader_src, &fragment_shader_src, None)
            .unwrap();

    let mut keyboard_state = input::KeyboardState::new();
    let mut mouse_info = input::MouseInfo {
        position: glutin::dpi::PhysicalPosition { x: 0.0, y: 0.0 },
    };

    let mut player = player::Player::default();

    let mut last = std::time::Instant::now();

    match display.gl_window().window().set_cursor_grab(true) {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    }
    display.gl_window().window().set_cursor_visible(false);

    event_loop.run(move |event, _, control_flow| {
        let now = std::time::Instant::now();
        let delta = (now - last).as_secs_f32();
        last = now;

        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                glutin::event::WindowEvent::CursorMoved {device_id:_, position, ..} => mouse_info.position = position,
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            glutin::event::Event::DeviceEvent { device_id:_, event } => match event {
                glutin::event::DeviceEvent::Key(key) => {keyboard_state.process_event(key.state, key.virtual_keycode.unwrap());
                match key.virtual_keycode.as_ref().unwrap() {
                    glutin::event::VirtualKeyCode::Escape => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                        return;
                },
                    _ => (),
                }},
                _ => (),
            },
            _ => (),
        }

        let mut target = display.draw();
        target.clear_color_and_depth((0.2, 0.5, 0.8, 1.0), 1.0);

        let (uwidth, uheight) = target.get_dimensions();
        let (mouse_x, mouse_y) = (mouse_info.position.x as f32, mouse_info.position.y as f32);

        player.get_camera_mut().yaw -= (mouse_x - (uwidth / 2) as f32) * delta * player.rot_speed;
        player.get_camera_mut().pitch += (mouse_y - (uheight / 2) as f32) * delta * player.rot_speed;

        player.get_camera_mut().pitch = player.get_camera().pitch.min(3.141592);
        player.get_camera_mut().pitch = player.get_camera().pitch.max(0.0);

        match display.gl_window().window().set_cursor_position(glium::glutin::dpi::PhysicalPosition { x: (uwidth/2), y: (uheight/2) }) {
            Ok(_) => (),
            Err(e) => println!("Error: {}", e),
        }

        player.update(delta, &keyboard_state);
        chunk_loader.update(&player, &display);

        let model = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32]
        ];

        let light = [1.4, 0.4, 0.7f32];

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            .. Default::default()
        };

        let perspective = player.get_camera().perspective(&target);

        chunk_loader.render(&player, &mut target, &program, &uniform! { model: model, view: player.get_camera().view_matrix(), perspective: perspective,
            u_light: light, diffuse_tex: &diffuse_texture, normal_tex: &normal_map }, &params);

        target.finish().unwrap();
    });
}
