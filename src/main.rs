use std::time::Duration;

use rtrb::{Consumer, RingBuffer};

use glow::*;

#[cfg(target_os = "macos")]
use baseview::copy_to_clipboard;
use baseview::{Event, EventStatus, Window, WindowHandler, WindowScalePolicy};

#[derive(Debug, Clone)]
enum Message {
    Hello,
}

struct OpenWindowExample {
    _rx: Consumer<Message>,
    gl: Option<glow::Context>,      // Zmienione na Option
    program: Option<NativeProgram>, // Zmienione na Option
    vao: Option<NativeVertexArray>,
    is_gl_initialized: bool, // Add this field
}

impl OpenWindowExample {
    fn init(&mut self, window: &Window) {
        let context = window
            .gl_context()
            .expect("failed to get baseview gl context");
        unsafe {
            context.make_current();
        }

        unsafe {
            self.gl = Some(glow::Context::from_loader_function(|s| {
                context.get_proc_address(s) as *const _
            }));

            // Create a program
            let program = self
                .gl
                .as_ref()
                .unwrap()
                .create_program()
                .expect("Cannot create program");
            self.program = Some(program); // Store the program in your struct

            let (vertex_shader_src, fragment_shader_src) = (
                r#"const vec2 verts[3] = vec2[3](
                vec2(0.5f, 1.0f),
                vec2(0.0f, 0.0f),
                vec2(1.0f, 0.0f)
            );
            out vec2 vert;
            void main() {
                vert = verts[gl_VertexID];
                gl_Position = vec4(vert - 0.5, 0.0, 1.0);
            }"#,
                r#"precision mediump float;
            in vec2 vert;
            out vec4 color;
            void main() {
                color = vec4(vert, 0.5, 1.0);
            }"#,
            );

            let shader_version = "#version 300 es";

            let vertex_shader = self
                .gl
                .as_ref()
                .unwrap()
                .create_shader(glow::VERTEX_SHADER)
                .unwrap();
            self.gl.as_ref().unwrap().shader_source(
                vertex_shader,
                &format!("{}\n{}", shader_version, vertex_shader_src),
            );
            self.gl.as_ref().unwrap().compile_shader(vertex_shader);
            if !self
                .gl
                .as_ref()
                .unwrap()
                .get_shader_compile_status(vertex_shader)
            {
                panic!(
                    "{}",
                    self.gl.as_ref().unwrap().get_shader_info_log(vertex_shader)
                );
            }

            self.gl
                .as_ref()
                .unwrap()
                .attach_shader(self.program.unwrap(), vertex_shader);

            let fragment_shader = self
                .gl
                .as_ref()
                .unwrap()
                .create_shader(glow::FRAGMENT_SHADER)
                .unwrap();
            self.gl.as_ref().unwrap().shader_source(
                fragment_shader,
                &format!("{}\n{}", shader_version, fragment_shader_src),
            );
            self.gl.as_ref().unwrap().compile_shader(fragment_shader);
            self.gl.as_ref().unwrap().compile_shader(fragment_shader);
            if !self
                .gl
                .as_ref()
                .unwrap()
                .get_shader_compile_status(fragment_shader)
            {
                panic!(
                    "{}",
                    self.gl
                        .as_ref()
                        .unwrap()
                        .get_shader_info_log(fragment_shader)
                );
            }

            self.gl
                .as_ref()
                .unwrap()
                .attach_shader(self.program.unwrap(), fragment_shader);

            self.gl.as_ref().unwrap().link_program(program);
            if !self.gl.as_ref().unwrap().get_program_link_status(program) {
                panic!(
                    "{}",
                    self.gl.as_ref().unwrap().get_program_info_log(program)
                );
            }

            self.gl
                .as_ref()
                .unwrap()
                .detach_shader(program, vertex_shader);
            self.gl.as_ref().unwrap().delete_shader(vertex_shader);

            self.gl
                .as_ref()
                .unwrap()
                .detach_shader(program, fragment_shader);
            self.gl.as_ref().unwrap().delete_shader(fragment_shader);


            self.vao = Some(
                self.gl
                    .as_ref()
                    .unwrap()
                    .create_vertex_array()
                    .expect("Cannot create vertex array"),
            );
            self.gl
                .as_ref()
                .unwrap()
                .bind_vertex_array(Some(self.vao.unwrap()));
        }
    }

    fn draw(&self) {
        unsafe {
            // Set the background color
            self.gl.as_ref().unwrap().clear_color(0.2, 0.3, 0.3, 1.0);
            // Clear the color buffer
            self.gl.as_ref().unwrap().clear(glow::COLOR_BUFFER_BIT);

            self.gl
                .as_ref()
                .unwrap()
                .use_program(Some(self.program.unwrap()));
            self.gl
                .as_ref()
                .unwrap()
                .bind_vertex_array(Some(self.vao.unwrap()));

            self.gl.as_ref().unwrap().draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }
}

// struct OpenWindowExample {
//     rx: Consumer<Message>,
// }

impl WindowHandler for OpenWindowExample {
    fn on_frame(&mut self, window: &mut Window) {
        let context = window
            .gl_context()
            .expect("Failed to get baseview gl context");

        if !self.is_gl_initialized {
            self.init(window);
            self.is_gl_initialized = true; // Set to true after initialization
        }

        self.draw();

        // while let Ok(message) = self.rx.pop() {
        //     println!("Message: {:?}", message);
        // }
        unsafe {
            context.make_current();
            context.swap_buffers();
        }
    }

    fn on_event(&mut self, _window: &mut Window, event: Event) -> EventStatus {
        match event {
            Event::Mouse(e) => {
                println!("Mouse event: {:?}", e);

                #[cfg(target_os = "macos")]
                match e {
                    MouseEvent::ButtonPressed { .. } => copy_to_clipboard(&"This is a test!"),
                    _ => (),
                }
            }
            Event::Keyboard(e) => println!("Keyboard event: {:?}", e),
            Event::Window(e) => println!("Window event: {:?}", e),
        }

        EventStatus::Captured
    }
}

fn main() {
    let window_open_options = baseview::WindowOpenOptions {
        title: "baseview".into(),
        size: baseview::Size::new(512.0, 512.0),
        scale: WindowScalePolicy::SystemScaleFactor,
        //#[cfg(feature = "opengl")]
        gl_config: Some(Default::default()),
    };

    let (mut tx, rx) = RingBuffer::new(128);

    ::std::thread::spawn(move || loop {
        ::std::thread::sleep(Duration::from_secs(5));

        if let Err(_) = tx.push(Message::Hello) {
            println!("Failed sending message");
        }
    });

    //Window::open_blocking(window_open_options, |_| OpenWindowExample { rx });
    Window::open_blocking(window_open_options, move |_| OpenWindowExample {
        _rx: rx,
        gl: None,      // Teraz jest to Option
        program: None, // Teraz jest to Option
        vao: None,     // Teraz jest to Option
        is_gl_initialized: false,
    });
}
