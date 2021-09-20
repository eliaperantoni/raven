#![feature(try_blocks)]

use std::error::Error;
use std::ffi::CString;
use std::fs;
use std::path::PathBuf;

use gl;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use imgui::{im_str, Context, Window};
use imgui_opengl_renderer::Renderer;
use imgui_sys;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use raven_core::component::{HierarchyComponent, NameComponent};
use raven_core::ecs::{Query, Entity};
use raven_core::framebuffer::Framebuffer;
use raven_core::path;
use raven_core::Processor;
use raven_core::resource::Scene;

mod import;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct OpenProjectState {
    project_root: PathBuf,
    processor: Processor,
    framebuffer: Option<([u32; 2], Framebuffer)>,
}

fn main() -> Result<()> {
    let el = EventLoop::new();

    let wb = WindowBuilder::new()
        .with_maximized(true)
        .with_title("Raven");

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);
    imgui.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;

    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(
        imgui.io_mut(),
        windowed_context.window(),
        HiDpiMode::Locked(1.0),
    );

    let renderer = Renderer::new(&mut imgui, |symbol| {
        windowed_context.get_proc_address(symbol)
    });
    gl::load_with(|symbol| windowed_context.get_proc_address(symbol));

    // Error currently displayed
    let mut err: Option<Box<dyn Error>> = None;

    // Currently loaded project
    let mut proj_state: Option<OpenProjectState> = None;

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

                if err.is_some() {
                    // TODO Disable docking

                    Window::new(im_str!("Error"))
                        .resizable(false)
                        .collapsible(false)
                        .position(
                            {
                                let [width, height] = ui.io().display_size;
                                [0.5 * width, 0.5 * height]
                            },
                            imgui::Condition::Once,
                        )
                        .position_pivot([0.5, 0.5])
                        .build(&ui, || {
                            ui.text("An error occurred:");
                            ui.text(err.as_ref().unwrap().to_string());

                            // Spacing
                            ui.dummy([0.0, 10.0]);

                            if ui.button(im_str!("Ok"), [ui.content_region_avail()[0], 25.0]) {
                                err = None;
                            }
                        });
                }

                match try {
                    match proj_state.as_mut() {
                        Some(proj_state) => draw_editor_window(&ui, proj_state)?,
                        None => match draw_select_project_window(&ui)? {
                            Some(new_proj_state) => proj_state = Some(new_proj_state),
                            None => (),
                        },
                    }
                } {
                    Ok(_) => (),
                    Err(new_err) => err = Some(new_err),
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

fn draw_select_project_window(ui: &imgui::Ui) -> Result<Option<OpenProjectState>> {
    const BTN_SIZE: [f32; 2] = [200.0, 30.0];

    let mut out = Ok(None);

    Window::new(im_str!("ProjectPicker"))
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .movable(false)
        .position(
            {
                let [width, height] = ui.io().display_size;
                [0.5 * width, 0.5 * height]
            },
            imgui::Condition::Always,
        )
        .position_pivot([0.5, 0.5])
        .build(ui, || {
            let maybe_err: Result<()> = try {
                if ui.button(im_str!("Open existing project"), BTN_SIZE) {
                    match nfd::open_pick_folder(None) {
                        Ok(nfd::Response::Okay(path)) => {
                            let mut processor = Processor::new(&path).unwrap();
                            processor.load_scene("$/main.scn").unwrap();

                            out = Ok(Some(OpenProjectState {
                                project_root: PathBuf::from(&path),
                                processor,
                                framebuffer: None,
                            }))
                        }
                        _ => (),
                    }
                }
            };

            match maybe_err {
                Ok(_) => (),
                Err(err) => out = Err(err),
            }
        });

    out
}

fn draw_editor_window(ui: &imgui::Ui, proj_state: &mut OpenProjectState) -> Result<()> {
    let mut out = Ok(true);

    let viewport = unsafe { imgui_sys::igGetMainViewport() };

    unsafe {
        imgui_sys::igSetNextWindowPos(
            (*viewport).Pos,
            imgui_sys::ImGuiCond_Always as _,
            imgui_sys::ImVec2::default(),
        );
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

    // Don't check if `begin` was successful because we always want to pop the style
    let main_window = Window::new(im_str!("Raven"))
        .flags(w_flags)
        .begin(&ui)
        .ok_or_else(|| Box::<dyn Error>::from("couldn't create main window"))?;
    style_stack.pop(&ui);

    let mut res: Result<()> = Ok(());

    if let Some(menu_bar) = ui.begin_main_menu_bar() {
        if let Some(menu) = ui.begin_menu(im_str!("File"), true) {
            res = try {
                if imgui::MenuItem::new(im_str!("Import external")).build(ui) {
                    match nfd::open_file_dialog(None, None) {
                        Ok(nfd::Response::Okay(fs_path)) => {
                            let fs_path = PathBuf::from(fs_path);

                            let file_name = fs_path
                                .file_name()
                                .ok_or_else(|| Box::<dyn Error>::from("Invalid path"))?;

                            let mut raven_path = PathBuf::default();
                            raven_path.push(path::PROJECT_ROOT_RUNE);
                            raven_path.push(file_name);

                            fs::copy(
                                fs_path,
                                path::as_fs_abs(&proj_state.project_root, &raven_path),
                            )?;

                            import::import(&raven_path, proj_state)?;
                        }
                        _ => (),
                    }
                }
            };

            menu.end(ui);
        }
        menu_bar.end(ui);
    }

    match res {
        Err(err) => {
            main_window.end(&ui);
            return Err(err);
        }
        _ => (),
    }

    // Setup docking and dock windows
    unsafe {
        let dock_name = CString::new("dock_space").unwrap();
        let id = imgui_sys::igGetIDStr(dock_name.as_ptr());

        if imgui_sys::igDockBuilderGetNode(id).is_null() {
            imgui_sys::igDockBuilderAddNode(id, imgui_sys::ImGuiDockNodeFlags_DockSpace);
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
            proj_state
                .processor
                .set_canvas_size(width as _, height as _);

            // If no framebuffer is present or the panel's size has changed
            if match &proj_state.framebuffer {
                Some((current_size, _)) => current_size != &[width as u32, height as u32],
                None => true,
            } {
                proj_state.framebuffer = Some((
                    [width as _, height as _],
                    Framebuffer::new((width as _, height as _)),
                ));
            }

            // Get a reference to the framebuffer contained in the Option
            let (_, framebuffer) = proj_state.framebuffer.as_ref().unwrap();

            // Render a frame inside the framebuffer
            framebuffer.bind();
            match proj_state.processor.do_frame() {
                Ok(_) => (),
                Err(err) => out = Err(err),
            }
            framebuffer.unbind();

            if out.is_err() {
                return;
            }

            // Display it
            imgui::Image::new(
                imgui::TextureId::new(framebuffer.get_tex_id() as _),
                [width, height],
            )
            .build(&ui);
        });

    style_stack.pop(&ui);

    Window::new(im_str!("Content browser")).build(&ui, || {
        ui.text("Hello I'm the content browser");
    });

    Window::new(im_str!("Hierarchy")).build(&ui, || {
        let scene = match proj_state.processor.get_scene() {
            Some(scene) => scene,
            None => return,
        };

        fn draw_tree_node(ui: &imgui::Ui, ent: Entity, hier_comp: &HierarchyComponent, scene: &Scene, next_nameless_name: &mut u32) {
            let name = match scene.get_one::<NameComponent>(ent) {
                Some(name_comp) => name_comp.0.clone(),
                None => format!("{}", *next_nameless_name),
            };

            let title = imgui::ImString::from(name);
            *next_nameless_name += 1;

            imgui::TreeNode::new(&title).default_open(false).leaf(hier_comp.children.len() == 0).build(ui, || {
                for child in &hier_comp.children {
                    if let Some(hier_comp) = scene.get_one::<HierarchyComponent>(*child) {
                        draw_tree_node(ui, *child, &*hier_comp, scene, next_nameless_name);
                    }
                }
            });
        };

        let mut next_nameless_name = 0;

        for (ent, (hier_comp,), _) in <(HierarchyComponent,)>::query_shallow(scene)
            .filter(|(ent, (hier_comp,), _)| hier_comp.parent.is_none())
        {
            draw_tree_node(ui, ent, &*hier_comp, scene, &mut next_nameless_name);
        }
    });

    main_window.end(&ui);

    Ok(())
}
