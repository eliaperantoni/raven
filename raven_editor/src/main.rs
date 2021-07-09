use std::error::Error;
use std::ffi::CString;
use std::time::Instant;

use gl;
use glutin::ContextBuilder;
use glutin::dpi::{LogicalSize, Size};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use imgui::{Context, im_str, Window};
use imgui_opengl_renderer::Renderer;
use imgui_sys;
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
    imgui.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;

    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), windowed_context.window(), HiDpiMode::Rounded);

    let renderer = Renderer::new(&mut imgui, |symbol| windowed_context.get_proc_address(symbol));
    gl::load_with(|symbol| windowed_context.get_proc_address(symbol));

    let mut last_frame = Instant::now();

    let scene = build_demo_scene()?;
    let mut raven = Raven::from_scene(scene)?;

    let mut framebuffer: Option<([f32; 2], Framebuffer)> = None;

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
                let ui = imgui.frame();
                let mut run = true;

                let viewport = unsafe { imgui_sys::igGetMainViewport() };

                unsafe {
                    imgui_sys::igSetNextWindowPos((*viewport).Pos, imgui_sys::ImGuiCond_Always as _, imgui_sys::ImVec2::default());
                    imgui_sys::igSetNextWindowSize((*viewport).Size, imgui_sys::ImGuiCond_Always as _);
                    imgui_sys::igSetNextWindowViewport((*viewport).ID);
                }

                let w_flags = {
                    use imgui::WindowFlags;
                    let mut w_flags = WindowFlags::empty();
                    for w_flag in vec![
                        WindowFlags::NO_TITLE_BAR,
                        WindowFlags::NO_COLLAPSE,
                        WindowFlags::NO_RESIZE,
                        WindowFlags::NO_MOVE,
                        WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS,
                        WindowFlags::NO_NAV_FOCUS,
                        WindowFlags::NO_BACKGROUND,
                    ] {
                        w_flags.insert(w_flag);
                    }
                    w_flags
                };

                let style_stack = {
                    use imgui::StyleVar::*;
                    ui.push_style_vars(vec![
                        &WindowRounding(0.0),
                        &WindowBorderSize(0.0),
                        &WindowPadding([0.0, 0.0]),
                    ])
                };

                Window::new(im_str!("Raven")).flags(w_flags).build(&ui, || {
                    unsafe {
                        let dock_name = CString::new("dock_space").unwrap();
                        let id = imgui_sys::igGetIDStr(dock_name.as_ptr());

                        imgui_sys::igDockSpace(id, imgui_sys::ImVec2::new(0.0, 0.0), imgui_sys::ImGuiDockNodeFlags_PassthruCentralNode as _, 0 as _);
                    }

                    Window::new(im_str!("Viewport")).size([800.0, 600.0], imgui::Condition::Once).build(&ui, || {
                        let [width, height] = ui.window_size();

                        if match &framebuffer {
                            Some(([curr_width, curr_height], _)) => *curr_width != width || *curr_height != height,
                            None => true,
                        } {
                            framebuffer.insert(
                                ([width, height], Framebuffer::new((width as _, height as _)))
                            );
                        }

                        let (_, framebuffer) = framebuffer.as_ref().unwrap();

                        framebuffer.with(|| {
                            raven.do_frame();
                        });

                        imgui::Image::new(imgui::TextureId::new(framebuffer.get_tex_id() as _), [width, height]).build(&ui);
                    });
                });

                style_stack.pop(&ui);

                if !run {
                    *control_flow = ControlFlow::Exit;
                }

                unsafe {
                    gl::ClearColor(215.0 / 255.0, 235.0 / 255.0, 245.0 / 255.0, 1.0);
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
