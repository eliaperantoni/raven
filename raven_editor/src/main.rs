use std::error::Error;
use std::time::Instant;

use gl;
use glutin::ContextBuilder;
use glutin::dpi::{LogicalSize, Size};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use imgui::{Context, im_str, Window};
use imgui_opengl_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use raven_core::component::CameraComponent;
use raven_core::entity::Entity;
use raven_core::framebuffer::Framebuffer;
use raven_core::model::ModelLoader;
use raven_core::Raven;

fn main() -> Result<(), Box<dyn Error>> {
    let el = EventLoop::new();

    let wb = WindowBuilder::new()
        .with_inner_size(Size::Logical(LogicalSize {
            width: 1024_f64,
            height: 768_f64,
        }))
        .with_title("Raven");

    let windowed_context = ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(wb, &el)
        .unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), windowed_context.window(), HiDpiMode::Rounded);

    let renderer = Renderer::new(&mut imgui, |symbol| windowed_context.get_proc_address(symbol));
    gl::load_with(|symbol| windowed_context.get_proc_address(symbol));

    let mut last_frame = Instant::now();

    let scene = build_demo_scene()?;
    let mut raven = Raven::from_scene(scene)?;

    let mut framebuffer = Framebuffer::new((800, 600));

    el.run(move |event, _, control_flow| {
        match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::MainEventsCleared => {
                platform
                    .prepare_frame(imgui.io_mut(), windowed_context.window())
                    .expect("Failed to prepare frame");
                windowed_context.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                framebuffer.with(|| {
                    raven.do_frame();
                });

                let ui = imgui.frame();

                let mut run = true;
                Window::new(im_str!("Raven")).build(&ui, || {
                    ui.text("Hello World!");
                    let image = imgui::Image::new(imgui::TextureId::new(framebuffer.get_tex_id() as _), [800_f32, 600_f32]);
                    image.build(&ui);
                });
                if !run {
                    *control_flow = ControlFlow::Exit;
                }

                unsafe {
                    gl::ClearColor(0.1, 0.1, 0.1, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }

                platform.prepare_render(&ui, windowed_context.window());
                renderer.render(ui);

                windowed_context.swap_buffers().unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            ev => platform.handle_event(imgui.io_mut(), windowed_context.window(), &ev)
        }
    });
}

fn build_demo_scene() -> Result<Entity, Box<dyn Error>> {
    let mut scene = Entity::default();
    scene.add_child(
        {
            let mut camera_entity = Entity::default();

            camera_entity.transform.position.z += 9.0;

            camera_entity.add_component(
                CameraComponent::default().into()
            );

            camera_entity
        },
    );
    scene.add_child(
        ModelLoader::from_file("models/cube/cube.obj")?
    );

    Ok(scene)
}
