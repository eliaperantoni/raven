use std::error::Error;

use glutin::ContextBuilder;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;

use raven_core::component::*;
use raven_core::ecs::Component;
use raven_core::io::Serializable;
use raven_core::Processor;
use raven_core::resource::*;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const PROJECT_ROOT: &'static str = "/home/elia/code/raven_proj";

fn main() -> Result<()> {
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Raven");

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    gl::load_with(|symbol| windowed_context.get_proc_address(symbol));

    let mut processor = Processor::new("/home/elia/code/raven_proj")?;
    processor.load_scene("$/.import/ferris/ferris.fbx/main.scn")?;

    el.run(move |event, _, control_flow| {
        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::MainEventsCleared => {
                windowed_context.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                processor.do_frame().unwrap();
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
