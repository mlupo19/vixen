#[macro_use]
extern crate glium;

mod camera;
mod chunk;
mod input;
mod loader;
mod player;
mod terrain;
mod chunk_mesh;
mod clipboard;

use std::io::Cursor;

use imgui::{Context, FontConfig, FontSource, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use imgui::*;

use glium::glutin::event::{Event, WindowEvent, VirtualKeyCode};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::{glutin, Surface};

struct System {
    event_loop: EventLoop<()>,
    display: glium::Display,
    imgui: Context,
    platform: WinitPlatform,
    renderer: Renderer,
}

fn main() {
    let mut sys = init();


    let image = image::load(
        Cursor::new(&include_bytes!("../res/diffuse.jpg")),
        image::ImageFormat::Jpeg,
    )
    .unwrap()
    .to_rgba8();

    let image_dimensions = image.dimensions();
    let image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let diffuse_texture = glium::texture::SrgbTexture2d::new(&sys.display, image).unwrap();

    let image = image::load(
        Cursor::new(&include_bytes!("../res/normal.png")),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let image_dimensions = image.dimensions();
    let image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let normal_map = glium::texture::Texture2d::new(&sys.display, image).unwrap();

    let vertex_shader_src =
        std::fs::read_to_string("src/shaders/vertex.glsl").expect("Unable to read vertex.glsl");
    let fragment_shader_src =
        std::fs::read_to_string("src/shaders/frag.glsl").expect("Unable to read frag.glsl");

    let program =
        glium::Program::from_source(&sys.display, &vertex_shader_src, &fragment_shader_src, None)
            .unwrap();


    let mut chunk_loader = loader::ChunkLoader::new(0);
    let mut input = input::Input::new();
    let mut player = player::Player::default();

    match sys.display.gl_window().window().set_cursor_grab(true) {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    }
    sys.display.gl_window().window().set_cursor_visible(false);


    // EVENT LOOP

    let mut last_frame = std::time::Instant::now();
    let mut last_tick = last_frame;
    let mut frames = 0;
    let mut last_q_sec = last_frame;
    let mut fps: f64 = 0.0;
    
    sys.event_loop.run(move |event, _, control_flow| {

        match event {
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            Event::NewEvents(cause) => {
                match cause {
                    glutin::event::StartCause::ResumeTimeReached { .. } => (),
                    glutin::event::StartCause::Init => (),
                    _ => return,
                }
                let now = std::time::Instant::now();
                sys.imgui.io_mut().update_delta_time(now - last_frame);
            },
            Event::MainEventsCleared => {
                let now = std::time::Instant::now();
                let delta = (now - last_tick).as_secs_f32();
                last_tick = now;

                if input.is_key_released(&VirtualKeyCode::LAlt) {
        
                    let (uwidth, uheight) = sys.display.get_framebuffer_dimensions();
        
                    player.get_camera_mut().yaw -= input.get_mouse_delta_x() as f32 * delta * player.rot_speed;
                    player.get_camera_mut().pitch += input.get_mouse_delta_y() as f32 * delta * player.rot_speed;
        
                    player.get_camera_mut().pitch = player.get_camera().pitch.min(3.141592);
                    player.get_camera_mut().pitch = player.get_camera().pitch.max(0.0);
        
                    match sys.display.gl_window().window().set_cursor_position(glium::glutin::dpi::PhysicalPosition { x: (uwidth/2), y: (uheight/2) }) {
                        Ok(_) => (),
                        Err(e) => println!("Error: {}", e),
                    }

                    input.update_mouse((0.0, 0.0));
                }
        
                chunk_loader.update(&player, &sys.display);
                player.update(delta, &input, &chunk_loader);

                
                let gl_window = sys.display.gl_window();
                sys.platform
                    .prepare_frame(sys.imgui.io_mut(), gl_window.window())
                    .expect("Failed to prepare frame");
                gl_window.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let delta_duration = now - last_frame;
                let delta = delta_duration.as_secs_f32();
                last_frame = now;
                let since_q_sec = (now - last_q_sec).as_secs_f64();
                
                if frames > 1 && since_q_sec > 0.25 {
                    last_q_sec = now;
                    fps = frames as f64 / since_q_sec;
                    frames = 0;
                }

                if 1.0 / delta < 20.0 {
                    println!("{}", 1.0 / delta);
                }

                let mut ui = sys.imgui.frame();

                let mut run = true;
                run_ui(&mut run, &mut ui, fps);
                if !run {
                    *control_flow = ControlFlow::Exit;
                }


                let mut target = sys.display.draw();
                target.clear_color_and_depth((0.2, 0.5, 0.8, 1.0), 1.0);
                
                let light = [1.4, 0.4, 0.7f32];
        
                let params = glium::DrawParameters {
                    depth: glium::Depth {
                        test: glium::draw_parameters::DepthTest::IfLess,
                        write: true,
                        .. Default::default()
                    },
                    backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                    multisampling: true,
                    .. Default::default()
                };
        
                let perspective = player.get_camera().perspective(&target);
        
                chunk_loader.render(&player, &mut target, &program, &uniform! { model: MODEL, view: player.get_camera().view_matrix(), perspective: perspective,
                    u_light: light, diffuse_tex: &diffuse_texture, normal_tex: &normal_map }, &params);

                let draw_data = ui.render();
                sys.renderer
                    .render(&mut target, draw_data)
                    .expect("Rendering failed");
                target.finish().expect("Failed to swap buffers");

                frames += 1;
            }
            Event::DeviceEvent { device_id:_, ref event } => 
                match event {
                    glutin::event::DeviceEvent::Key(key) => {
                        input.process_event(key.state, key.virtual_keycode.unwrap());
                        match key.virtual_keycode.as_ref().unwrap() {
                            VirtualKeyCode::Escape => {
                                *control_flow = ControlFlow::Exit;
                                return;
                            },
                            VirtualKeyCode::LAlt => {
                                match key.state {
                                    glutin::event::ElementState::Pressed => {
                                        match sys.display.gl_window().window().set_cursor_grab(false) {
                                            Ok(_) => (),
                                            Err(e) => println!("Error: {}", e),
                                        }
                                        sys.display.gl_window().window().set_cursor_visible(true);
                                    },
                                    glutin::event::ElementState::Released => {
                                        match sys.display.gl_window().window().set_cursor_grab(true) {
                                            Ok(_) => (),
                                            Err(e) => println!("Error: {}", e),
                                        }
                                        sys.display.gl_window().window().set_cursor_visible(false);
                                    },
                                }
                            },
                            _ => (),
                        }
                    },
                    glutin::event::DeviceEvent::MouseMotion{delta} => {
                        input.update_mouse(*delta);
                    }
                    _ => (),
                },
            _ => (),
        }
    });
}

fn run_ui(_run: &mut bool, ui: &mut Ui, fps: f64) {
    let window = Window::new("FPS");
    let tok = window.begin(ui).unwrap();
    ui.text(format!("{}", fps));
    ui.dummy([100.0,50.0]);
    tok.end();
}

fn init() -> System {

    let event_loop = EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Vixen")
        .with_inner_size(glium::glutin::dpi::PhysicalSize {
            width: 1920,
            height: 1080,
        })
        .with_position(glium::glutin::dpi::PhysicalPosition { x: 0, y: 0 });
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24).with_multisampling(16);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // IMGUI

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    if let Some(backend) = clipboard::init() {
        imgui.set_clipboard_backend(backend);
    } else {
        eprintln!("Failed to initialize clipboard");
    }

    let mut platform = WinitPlatform::init(&mut imgui);
    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Rounded);
    }

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[
        FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        },
    ]);

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    let renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

    System { event_loop, display, imgui, platform, renderer }
}

const MODEL: [[f32;4];4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0f32]
];
