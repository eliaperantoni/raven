#![feature(try_blocks)]
#![feature(label_break_value)]

use std::collections::HashMap;
use std::error::Error;
use std::ffi::CString;
use std::fs;
use std::path::PathBuf;

use gl;
use glob;
use glutin::ContextBuilder;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use imgui::{Context, StyleColor, Ui, Window};
use imgui_opengl_renderer::Renderer;
use imgui_sys;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use itertools::Itertools;
use palette;
use palette::{FromColor, Saturate, Shade};

use raven_core::component::{HierarchyComponent, NameComponent, SceneComponent, TransformComponent, CameraComponent};
use raven_core::ecs::{Entity, Query};
use raven_core::framebuffer::Framebuffer;
use raven_core::glam::{EulerRot, Mat4, Quat, Vec3};
use raven_core::mat4;
use raven_core::path;
use raven_core::Processor;
use raven_core::resource::Scene;
use raven_core::time::Delta;
use raven_core::FrameError;

mod import;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct OpenProjectState {
    project_root: PathBuf,
    processor: Processor,
    framebuffer: Option<([u32; 2], Framebuffer)>,

    selection: Option<Entity>,

    avail_resources: HashMap<ResourceType, Vec<PathBuf>>,
}

#[derive(Eq, PartialEq, Debug, Hash, Copy, Clone)]
enum ResourceType {
    Scene
}

impl ResourceType {
    fn glob(&self) -> &'static str {
        match self {
            Self::Scene => "*.scn"
        }
    }
}

impl OpenProjectState {
    fn scan_avail_resources(&mut self) -> Result<()> {
        self.avail_resources.clear();

        for r_type in vec![ResourceType::Scene] {
            let mut path = std::path::PathBuf::new();
            path.push(&self.project_root);
            path.push("**");
            path.push(r_type.glob());

            for match_ in glob::glob(path.to_str().expect("non utf8 path")).map_err(|err| Box::<dyn Error>::from(err))? {
                let abs_path = match_?;
                let rel_path = abs_path.strip_prefix(&self.project_root)?;

                let mut raven_path = PathBuf::new();
                raven_path.push(path::PROJECT_ROOT_RUNE);
                raven_path.push(rel_path);

                let vec = self.avail_resources.entry(r_type).or_insert_with(|| Vec::new());
                vec.push(raven_path);
            }
        }

        Ok(())
    }
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

    let mut delta = Delta::default();

    el.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                platform
                    .prepare_frame(imgui.io_mut(), windowed_context.window())
                    .expect("failed to prepare frame");
                windowed_context.window().request_redraw();

                if let Some(delta) = delta.on_frame() {
                    imgui.io_mut().delta_time = delta.as_secs_f32();
                }
            }
            Event::RedrawRequested(_) => {
                let ui = imgui.frame();

                if err.is_some() {
                    // TODO Disable docking

                    Window::new("Error")
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

                            if ui.button_with_size("Ok", [ui.content_region_avail()[0], 25.0]) {
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

    Window::new("ProjectPicker")
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
                if ui.button_with_size("Open existing project", BTN_SIZE) {
                    match nfd::open_pick_folder(None) {
                        Ok(nfd::Response::Okay(path)) => {
                            let processor = Processor::new(&path)?;

                            let mut state = OpenProjectState {
                                project_root: PathBuf::from(&path),
                                processor,
                                framebuffer: None,

                                selection: None,

                                avail_resources: HashMap::new(),
                            };

                            state.scan_avail_resources()?;

                            out = Ok(Some(state));
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
            WindowFlags::MENU_BAR,
        ] {
            w_flags.insert(w_flag);
        }
        w_flags
    };

    let style_stack = {
        use imgui::StyleVar::*;
        vec![
            ui.push_style_var(WindowRounding(0.0)),
            ui.push_style_var(WindowBorderSize(0.0)),
            ui.push_style_var(WindowPadding([0.0, 0.0])),
        ]
    };

    // Don't check if `begin` was successful because we always want to pop the style
    let main_window = Window::new("Raven")
        .flags(w_flags)
        .begin(&ui)
        .ok_or_else(|| Box::<dyn Error>::from("couldn't create main window"))?;
    style_stack.into_iter().for_each(|style| style.end());

    let mut res: Result<()> = Ok(());

    if let Some(menu_bar) = ui.begin_menu_bar() {
        if let Some(menu) = ui.begin_menu("File") {
            res = try {
                if imgui::MenuItem::new("Import external").build(ui) {
                    match nfd::open_file_dialog(None, Some(proj_state.project_root.to_str().expect("non utf8 path"))) {
                        Ok(nfd::Response::Okay(fs_path)) => {
                            let fs_path = PathBuf::from(fs_path);

                            if fs_path.starts_with(&proj_state.project_root) {
                                let rel_path = fs_path.strip_prefix(&proj_state.project_root)?;

                                let mut raven_path = PathBuf::new();
                                raven_path.push(path::PROJECT_ROOT_RUNE);
                                raven_path.push(rel_path);

                                import::import(&raven_path, proj_state)?;
                            } else {
                                let file_name = fs_path
                                    .file_name()
                                    .ok_or_else(|| Box::<dyn Error>::from("invalid path"))?;

                                let mut raven_path = PathBuf::default();
                                raven_path.push(path::PROJECT_ROOT_RUNE);
                                raven_path.push(file_name);

                                fs::copy(
                                    fs_path,
                                    path::as_fs_abs(&proj_state.project_root, &raven_path),
                                )?;

                                import::import(&raven_path, proj_state)?;
                            }

                            proj_state.scan_avail_resources()?;
                        }
                        _ => (),
                    }
                }

                if imgui::MenuItem::new("Open scene").build(ui) {
                    match nfd::open_file_dialog(None, Some(proj_state.project_root.to_str().expect("non utf8 path"))) {
                        Ok(nfd::Response::Okay(fs_path)) => {
                            let fs_path = PathBuf::from(fs_path);

                            if !fs_path.starts_with(&proj_state.project_root) {
                                Err(Box::<dyn Error>::from("non local scene"))?
                            }

                            let rel_path = fs_path.strip_prefix(&proj_state.project_root)?;

                            let mut raven_path = PathBuf::new();
                            raven_path.push(path::PROJECT_ROOT_RUNE);
                            raven_path.push(rel_path);

                            proj_state.processor.load_scene(&raven_path)?;
                        }
                        _ => (),
                    }
                }
            };

            menu.end();
        }
        menu_bar.end();
    }

    match res {
        Err(err) => {
            main_window.end();
            return Err(err);
        }
        _ => (),
    }

    if proj_state.processor.get_scene().is_none() {
        ui.text("No scene");
        return Ok(());
    }

    // Setup docking and dock windows
    unsafe {
        let dock_name = CString::new("dock_space").unwrap();
        let id = imgui_sys::igGetID_Str(dock_name.as_ptr());

        if imgui_sys::igDockBuilderGetNode(id).is_null() {
            imgui_sys::igDockBuilderAddNode(id, imgui_sys::ImGuiDockNodeFlags_DockSpace);
            imgui_sys::igDockBuilderSetNodeSize(id, (*viewport).Size);

            // +------------------------+
            // | TL  | R                |
            // |     |                  |
            // |     |                  |
            // |-----|                  |
            // | BL  |                  |
            // |     |                  |
            // |     |                  |
            // +------------------------+

            let mut left_id = 0;
            let mut right_id = 0;

            imgui_sys::igDockBuilderSplitNode(
                id,
                imgui_sys::ImGuiDir_Left,
                0.2,
                &mut left_id,
                &mut right_id,
            );

            let mut top_left_id = 0;
            let mut bot_left_id = 0;

            imgui_sys::igDockBuilderSplitNode(
                left_id,
                imgui_sys::ImGuiDir_Up,
                0.5,
                &mut top_left_id,
                &mut bot_left_id,
            );

            let window_name = CString::new("Hierarchy").unwrap();
            imgui_sys::igDockBuilderDockWindow(window_name.as_ptr(), top_left_id);

            let window_name = CString::new("Inspector").unwrap();
            imgui_sys::igDockBuilderDockWindow(window_name.as_ptr(), bot_left_id);

            let window_name = CString::new("Viewport").unwrap();
            imgui_sys::igDockBuilderDockWindow(window_name.as_ptr(), right_id);

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
        ui.push_style_var(WindowPadding([0.0, 0.0]))
    };

    Window::new("Viewport")
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
            let res = proj_state.processor.do_frame();
            framebuffer.unbind();

            match res {
                Ok(_) => (),
                Err(FrameError::Generic(err)) => {
                    out = Err(err);
                    return;
                },
                Err(FrameError::NoCamera) => {
                    ui.text("No camera");
                    return;
                }
            }

            // Display it
            imgui::Image::new(
                imgui::TextureId::new(framebuffer.get_tex_id() as _),
                [width, height],
            ).uv0([0.0, 1.0]).uv1([1.0, 0.0]).build(&ui);
        });

    style_stack.pop();

    Window::new("Hierarchy").build(&ui, || {
        let scene = match proj_state.processor.get_scene_mut() {
            Some(scene) => scene,
            None => return,
        };

        if ui.button_with_size("Create new entity", [ui.content_region_avail()[0], 0.0]) {
            let entity = scene.create();
            scene.attach(entity, TransformComponent::default());
            scene.attach(entity, HierarchyComponent::default());
            scene.attach(entity, NameComponent("New entity".to_owned()));
        }

        ui.spacing();
        ui.separator();
        ui.spacing();

        struct Ctx<'me> {
            ui: &'me imgui::Ui<'me>,
            scene: &'me Scene,
            next_nameless_name: &'me mut u32,
            selection: &'me mut Option<Entity>,
        }

        fn draw_tree_node(ctx: &mut Ctx, ent: Entity, hier_comp: &HierarchyComponent) {
            let name = match ctx.scene.get_one::<NameComponent>(ent) {
                Some(name_comp) => name_comp.0.clone(),
                None => {
                    let name = format!("{}", *ctx.next_nameless_name);
                    *ctx.next_nameless_name += 1;
                    name
                }
            };

            let tree_node = imgui::TreeNode::new(&imgui::ImString::from(name))
                .flags(imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH)
                .open_on_arrow(true)
                .selected(*ctx.selection == Some(ent))
                .leaf(hier_comp.children.is_empty())
                .open_on_double_click(true)
                .push(ctx.ui);

            if ctx.ui.is_item_clicked() && !ctx.ui.is_item_toggled_open() {
                *ctx.selection = Some(ent);
            }

            if let Some(tree_node) = tree_node {
                for child in &hier_comp.children {
                    if let Some(hier_comp) = ctx.scene.get_one::<HierarchyComponent>(*child) {
                        draw_tree_node(ctx, *child, &*hier_comp);
                    }
                }

                tree_node.end();
            }
        }

        let mut next_nameless_name = 0;

        let mut ctx = Ctx {
            ui,
            scene,
            next_nameless_name: &mut next_nameless_name,
            selection: &mut proj_state.selection,
        };

        for (ent, (hier_comp, ), _) in <(HierarchyComponent, )>::query_shallow(scene)
            .filter(|(_, (hier_comp, ), _)| hier_comp.parent.is_none())
        {
            draw_tree_node(&mut ctx, ent, &*hier_comp);
        }
    });

    Window::new("Inspector").build(ui, || {
        let selection = match proj_state.selection {
            Some(selection) => selection,
            None => return,
        };

        if ui.button_with_size("Add component", [ui.content_region_avail()[0], 0.0]) {
            ui.open_popup("Component");
        }

        ui.spacing();
        ui.separator();
        ui.spacing();

        ui.popup("Component", || {
            if imgui::Selectable::new("SceneComponent").build(ui) {
                let scene = proj_state.processor.get_scene_mut().unwrap();

                if scene.get_one::<SceneComponent>(selection).is_none() {
                    scene.attach(selection, SceneComponent::default());
                }
            }

            if imgui::Selectable::new("CameraComponent").build(ui) {
                let scene = proj_state.processor.get_scene_mut().unwrap();

                if scene.get_one::<CameraComponent>(selection).is_none() {
                    scene.attach(selection, CameraComponent::default());
                }
            }
        });

        ui.separator();

        match proj_state.processor.get_scene_mut().unwrap().get_one_mut::<NameComponent>(selection) {
            Some(mut name_comp) => {
                if imgui::CollapsingHeader::new("NameComponent").default_open(true).build(ui) {
                    imgui::InputText::new(ui, "Name", &mut name_comp.0).build();
                }
            }
            None => (),
        };

        let mut has_camera_component = true;

        match proj_state.processor.get_scene_mut().unwrap().get_one_mut::<CameraComponent>(selection) {
            Some(_) => {
                drop(imgui::CollapsingHeader::new("CameraComponent").leaf(true).build_with_close_button(ui, &mut has_camera_component));
            }
            None => (),
        };

        if !has_camera_component {
            proj_state.processor.get_scene_mut().unwrap().detach_one::<CameraComponent>(selection);
        }

        match proj_state.processor.get_scene_mut().unwrap().get_one_mut::<TransformComponent>(selection) {
            Some(mut tran_comp) => {
                let m4: &mut Mat4 = &mut tran_comp.0;

                let mut position = Vec3::default();
                let mut scale = Vec3::default();
                let mut rotation = Quat::default();

                mat4::decompose(m4.as_ref(), position.as_mut(), scale.as_mut(), rotation.as_mut());

                let mut rotation_euler = rotation.to_euler(EulerRot::XYZ);

                fn with_color<F: FnOnce() -> bool>(ui: &Ui, color: [f32; 3], f: F) -> bool {
                    fn to_hsv(color: [f32; 3]) -> palette::Hsv {
                        let [red, green, blue] = color;
                        palette::Hsv::from_color(palette::Srgb::new(red, green, blue))
                    }

                    fn from_hsv(color: palette::Hsv) -> [f32; 4] {
                        let (red, green, blue) = palette::Srgb::from_color(color).into_components();
                        [red, green, blue, 1.0]
                    }

                    let style_cols = vec![
                        ui.push_style_color(StyleColor::FrameBg, from_hsv(to_hsv(color).desaturate(0.7).darken(0.7))),
                        ui.push_style_color(StyleColor::FrameBgHovered, from_hsv(to_hsv(color).desaturate(0.7).darken(0.6))),
                        ui.push_style_color(StyleColor::FrameBgActive, from_hsv(to_hsv(color).desaturate(0.7).darken(0.5))),
                    ];
                    let res = f();
                    style_cols.into_iter().for_each(|style_col| style_col.pop());
                    res
                }

                const COL_RED: [f32; 3] = [1.0, 0.0, 0.0];
                const COL_GRE: [f32; 3] = [0.0, 1.0, 0.0];
                const COL_BLU: [f32; 3] = [0.0, 0.0, 1.0];

                const SPEED: f32 = 0.05;

                if imgui::CollapsingHeader::new("TransformComponent").default_open(true).build(ui) {
                    ui.text("Position");

                    ui.columns(3, "Position##Cols", false);

                    ui.set_next_item_width(ui.current_column_width());
                    with_color(ui, COL_RED, || imgui::Drag::new("##PosX").speed(SPEED).build(ui, &mut position.x));
                    ui.next_column();
                    ui.set_next_item_width(ui.current_column_width());
                    with_color(ui, COL_GRE, || imgui::Drag::new("##PosY").speed(SPEED).build(ui, &mut position.y));
                    ui.next_column();
                    ui.set_next_item_width(ui.current_column_width());
                    with_color(ui, COL_BLU, || imgui::Drag::new("##PosZ").speed(SPEED).build(ui, &mut position.z));
                    ui.next_column();

                    ui.spacing();

                    ui.columns(1, "Scale##LabelCol", false);
                    ui.text("Scale");

                    ui.columns(3, "Scale##Cols", false);

                    ui.set_next_item_width(ui.current_column_width());
                    with_color(ui, COL_RED, || imgui::Drag::new("##ScaleX").speed(SPEED).build(ui, &mut scale.x));
                    ui.next_column();
                    ui.set_next_item_width(ui.current_column_width());
                    with_color(ui, COL_GRE, || imgui::Drag::new("##ScaleY").speed(SPEED).build(ui, &mut scale.y));
                    ui.next_column();
                    ui.set_next_item_width(ui.current_column_width());
                    with_color(ui, COL_BLU, || imgui::Drag::new("##ScaleZ").speed(SPEED).build(ui, &mut scale.z));
                    ui.next_column();

                    ui.spacing();

                    ui.columns(1, "Rotation##LabelCol", false);
                    ui.text("Rotation");

                    ui.columns(3, "Rotation##Cols", false);

                    ui.set_next_item_width(ui.current_column_width());
                    with_color(ui, COL_RED, || imgui::Drag::new("##RotX").speed(SPEED).build(ui, &mut rotation_euler.0));
                    ui.next_column();
                    ui.set_next_item_width(ui.current_column_width());
                    with_color(ui, COL_GRE, || imgui::Drag::new("##RotY").speed(SPEED).build(ui, &mut rotation_euler.1));
                    ui.next_column();
                    ui.set_next_item_width(ui.current_column_width());
                    with_color(ui, COL_BLU, || imgui::Drag::new("##RotZ").speed(SPEED).build(ui, &mut rotation_euler.2));
                    ui.next_column();

                    ui.columns(1, "##Reset", false);
                }

                rotation = Quat::from_euler(EulerRot::XYZ, rotation_euler.0, rotation_euler.1, rotation_euler.2);

                mat4::compose(m4.as_mut(), position.as_ref(), scale.as_ref(), rotation.as_ref());
            }
            None => (),
        };

        let mut has_scene_component = true;

        type NewScene = Option<PathBuf>;

        // Some(Option<PathBuf>) if the scene has changed. None otherwise.
        let new_scene: Option<NewScene> = match proj_state.processor.get_scene().unwrap().get_one::<SceneComponent>(selection) {
            Some(scene_comp) => {
                if imgui::CollapsingHeader::new("SceneComponent").default_open(true).build_with_close_button(ui, &mut has_scene_component) {
                    try {
                        let scenes = proj_state.avail_resources.get(&ResourceType::Scene)?;

                        let mut scenes: Vec<Option<&PathBuf>> = scenes.into_iter().map(|scene| Some(scene)).collect::<Vec<_>>();
                        scenes.insert(0, None);

                        let (mut idx, _) = scenes.iter().find_position(|scene| **scene == scene_comp.scene.as_ref())?;

                        let scenes_str: Vec<_> = scenes.iter().map(|scene| match *scene {
                            Some(scene) => scene.to_str().expect("non utf8 path"),
                            None => "",
                        }).collect();

                        let old_idx = idx;

                        ui.set_next_item_width(ui.content_region_avail()[0]);
                        ui.combo_simple_string("##Scene", &mut idx, &scenes_str);

                        if old_idx == idx {
                            None?;
                        }

                        scenes[idx].cloned()
                    }
                } else {
                    None
                }
            }
            None => None,
        };

        if let Some(new_scene) = new_scene {
            let mut scene_comp = proj_state.processor.get_scene_mut().unwrap().get_one_mut::<SceneComponent>(selection).unwrap();
            scene_comp.scene = new_scene;
            scene_comp.loaded = None;
        }

        if !has_scene_component {
            proj_state.processor.get_scene_mut().unwrap().detach_one::<SceneComponent>(selection);
        }
    });

    main_window.end();

    Ok(())
}
