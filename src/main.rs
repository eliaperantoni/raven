use std::mem;

use gl::{self, types::*};
use glutin::ContextBuilder;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;

fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Raven");

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    gl::load_with(|s| windowed_context.get_proc_address(s));

    let vao = setup();

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::RedrawRequested(_) => {
                draw_frame(vao);
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}

const VERTEX_DATA: [GLfloat; 3 * (2 + 3)] = [
    0.5, -0.5, 1.0, 0.0, 0.0,
    -0.5, -0.5, 0.0, 1.0, 0.0,
    0.0, 0.5, 0.0, 0.0, 1.0,
];

fn setup() -> GLuint {
    let (mut vao, mut vbo) = (0, 0);

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            (VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            mem::transmute(&VERTEX_DATA[0]),
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0, 2, gl::FLOAT, gl::FALSE,
            (mem::size_of::<GLfloat>() * (2 + 3)) as GLsizei,
            (mem::size_of::<GLfloat>() * 0) as *const _,
        );
        gl::EnableVertexArrayAttrib(vao, 0);

        gl::VertexAttribPointer(
            1, 3, gl::FLOAT, gl::FALSE,
            (mem::size_of::<GLfloat>() * (2 + 3)) as GLsizei,
            (mem::size_of::<GLfloat>() * 2) as *const _,
        );
        gl::EnableVertexArrayAttrib(vao, 1);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    };

    return vao;
}

fn draw_frame(vao: GLuint) {
    unsafe {
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        gl::BindVertexArray(vao);
        gl::DrawArrays(gl::TRIANGLES, 0, 3);
    };
}
