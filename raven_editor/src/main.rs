use std::error::Error;
use std::ffi::CString;

use gl;
use glutin::ContextBuilder;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use imgui::{Context, im_str, Window};
use imgui_opengl_renderer::Renderer;
use imgui_sys;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use raven_core::framebuffer::Framebuffer;
use raven_core::Processor;

const PROJECT_ROOT: &'static str = "/home/elia/code/raven_proj";

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct ProjectState {
    processor: Processor,
    framebuffer: Option<([u32; 2], Framebuffer)>,
}

fn main() -> Result<()> {
    let el = EventLoop::new();

    let wb = WindowBuilder::new()
        .with_maximized(true)
        .with_title("Raven");

    let windowed_context = ContextBuilder::new()
        .build_windowed(wb, &el)
        .unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);
    imgui.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;

    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), windowed_context.window(), HiDpiMode::Locked(1.0));

    let renderer = Renderer::new(&mut imgui, |symbol| windowed_context.get_proc_address(symbol));
    gl::load_with(|symbol| windowed_context.get_proc_address(symbol));

    let mut proj_state: Option<ProjectState> = None;

    el.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                platform
                    .prepare_frame(imgui.io_mut(), windowed_context.window())
                    .expect("failed to prepare frame");
                windowed_context.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                let ui = imgui.frame();

                match proj_state.as_mut() {
                    Some(proj_state) => {
                        if !draw_editor_window(&ui, proj_state) {
                            *control_flow = ControlFlow::Exit;
                        }
                    },
                    None => {
                        if let Some(some_proj_state) = draw_select_project_window(&ui) {
                            proj_state.insert(some_proj_state);
                        }
                    },
                }

                unsafe {
                    gl::ClearColor(14.0 / 255.0, 12.0 / 255.0, 16.0 / 255.0, 1.0);
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
            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                windowed_context.resize(physical_size);
                imgui.io_mut().display_size = {
                    let width = physical_size.width;
                    let height = physical_size.height;
                    [width as f32, height as f32]
                };
            }
            ev => platform.handle_event(imgui.io_mut(), windowed_context.window(), &ev),
        }
    });
}

fn draw_select_project_window(ui: &imgui::Ui) -> Option<Result<ProjectState>> {
    const BTN_SIZE: [f32; 2] = [200.0, 30.0];

    Window::new(im_str!("ProjectPicker"))
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .movable(false)
        .position({
            let [width, height] = ui.io().display_size;
            [0.5 * width, 0.5 * height]
        }, imgui::Condition::Always)
        .position_pivot([0.5, 0.5])
        .build(ui, || {
            if ui.button(im_str!("Open existing project"), BTN_SIZE) {
                match nfd::open_pick_folder(None) {
                    Ok(nfd::Response::Okay(path)) => {
                        let mut processor = match Processor::new(path) {
                            Err(err) => return Some(Err(err)),
                        };

                        return Some(ProjectState {
                            processor,
                        });
                    }
                    _ => (),
                }
            }

            ui.button(im_str!("Create new project"), BTN_SIZE);
        });

    None
}

fn draw_editor_window(ui: &imgui::Ui, proj_state: &mut ProjectState) -> bool {
    let viewport = unsafe { imgui_sys::igGetMainViewport() };

    unsafe {
        imgui_sys::igSetNextWindowPos(
            (*viewport).Pos,
            imgui_sys::ImGuiCond_Always as _,
            imgui_sys::ImVec2::default(),
        );
        imgui_sys::igSetNextWindowSize(
            (*viewport).Size,
            imgui_sys::ImGuiCond_Always as _,
        );
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

    // Don't check if `begin` was successful because we always want to pop the style
    let main_window = Window::new(im_str!("Raven")).flags(w_flags).begin(&ui);
    style_stack.pop(&ui);

    // Setup docking and dock windows
    unsafe {
        let dock_name = CString::new("dock_space").unwrap();
        let id = imgui_sys::igGetIDStr(dock_name.as_ptr());

        if imgui_sys::igDockBuilderGetNode(id).is_null() {
            imgui_sys::igDockBuilderAddNode(
                id,
                imgui_sys::ImGuiDockNodeFlags_DockSpace,
            );
            imgui_sys::igDockBuilderSetNodeSize(id, (*viewport).Size);

            let mut hierarchy_id = 0;
            let mut viewport_id = 0;
            let mut cbrowser_id = 0;

            imgui_sys::igDockBuilderSplitNode(
                id,
                imgui_sys::ImGuiDir_Left,
                0.2,
                &mut hierarchy_id,
                &mut viewport_id,
            );
            imgui_sys::igDockBuilderSplitNode(
                viewport_id,
                imgui_sys::ImGuiDir_Down,
                0.2,
                &mut cbrowser_id,
                &mut viewport_id,
            );

            let window_name = CString::new("Viewport").unwrap();
            imgui_sys::igDockBuilderDockWindow(window_name.as_ptr(), viewport_id);

            let window_name = CString::new("Hierarchy").unwrap();
            imgui_sys::igDockBuilderDockWindow(window_name.as_ptr(), hierarchy_id);

            let window_name = CString::new("Content browser").unwrap();
            imgui_sys::igDockBuilderDockWindow(window_name.as_ptr(), cbrowser_id);

            imgui_sys::igDockBuilderFinish(id);
        }

        imgui_sys::igDockSpace(
            id,
            imgui_sys::ImVec2::new(0.0, 0.0),
            imgui_sys::ImGuiDockNodeFlags_PassthruCentralNode as _,
            0 as _,
        );
    }

    let style_stack = {
        use imgui::StyleVar::*;
        ui.push_style_vars(vec![&WindowPadding([0.0, 0.0])])
    };

    Window::new(im_str!("Viewport"))
        .size([800.0, 600.0], imgui::Condition::Once)
        .build(&ui, || {
            let [width, height] = ui.content_region_avail();

            // Resizes OpenGL viewport and sets camera aspect ratio
            proj_state.processor.set_canvas_size(width as _, height as _);

            // If no framebuffer is present or the panel's size has changed
            if match &proj_state.framebuffer {
                Some((current_size, _)) => current_size != &[width as u32, height as u32],
                None => true,
            } {
                proj_state.framebuffer = Some((
                    [width as _, height as _],
                    Framebuffer::new((width as _, height as _))
                ));
            }

            // Get a reference to the framebuffer contained in the Option
            let (_, framebuffer) = proj_state.framebuffer.as_ref().unwrap();

            // Render a frame inside the framebuffer
            framebuffer.bind();
            proj_state.processor.do_frame().expect("couldn't do frame");
            framebuffer.unbind();

            // Display it
            imgui::Image::new(
                imgui::TextureId::new(framebuffer.get_tex_id() as _),
                [width, height],
            ).build(&ui);
        });

    style_stack.pop(&ui);

    Window::new(im_str!("Content browser")).build(&ui, || {
        ui.text("Hello I'm the content browser");
    });

    Window::new(im_str!("Hierarchy")).build(&ui, || {
        ui.text("Hello I'm the hierarchy");
    });

    // If window was created (should always be the case) then end it
    if let Some(main_window) = main_window {
        main_window.end(&ui);
    }

    true
}
