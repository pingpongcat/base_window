use std::ffi::CString;
use std::time::Duration;

use rtrb::{Consumer, RingBuffer};

use gl::types::*;

#[cfg(target_os = "macos")]
use baseview::copy_to_clipboard;
use baseview::{Event, EventStatus, MouseEvent, Window, WindowHandler, WindowScalePolicy};

#[derive(Debug, Clone)]
enum Message {
    Hello,
}

struct OpenWindowExample {
    rx: Consumer<Message>,
    program: GLuint,
    vao: GLuint,
    is_gl_initialized: bool, // Add this field
}

impl OpenWindowExample {
    fn init_gl(&mut self) {
        unsafe {
            // Shader source code
            let vertex_shader_src = r#"
                #version 330 core
                layout(location = 0) in vec3 position;
                void main() {
                    gl_Position = vec4(position, 1.0);
                }
            "#;

            let fragment_shader_src = r#"
                #version 330 core
                out vec4 color;
                void main() {
                    color = vec4(1.0, 0.5, 0.2, 1.0);
                }
            "#;

            // Compile shaders
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(
                vertex_shader,
                1,
                &CString::new(vertex_shader_src).unwrap().as_ptr(),
                std::ptr::null(),
            );
            gl::CompileShader(vertex_shader);

            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(
                fragment_shader,
                1,
                &CString::new(fragment_shader_src).unwrap().as_ptr(),
                std::ptr::null(),
            );
            gl::CompileShader(fragment_shader);

            // Link shaders into a program
            self.program = gl::CreateProgram();
            gl::AttachShader(self.program, vertex_shader);
            gl::AttachShader(self.program, fragment_shader);
            gl::LinkProgram(self.program);

            // Clean up shaders as they're linked into the program now and no longer necessary
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            // Vertex data for a triangle
            let vertices: [GLfloat; 9] = [-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0];

            // Generate and bind a Vertex Array Object (VAO)
            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            // Generate and bind a Vertex Buffer Object (VBO)
            let mut vbo: GLuint = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // Vertex attribute pointer for the position
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                3 * std::mem::size_of::<GLfloat>() as GLsizei,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            // Unbind the VBO and VAO
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
    }

    fn draw(&self) {
        unsafe {
            // Set the background color
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            // Clear the color buffer
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Use the shader program
            gl::UseProgram(self.program);
            // Bind the Vertex Array Object (VAO)
            gl::BindVertexArray(self.vao);

            // Draw the triangle
            gl::DrawArrays(gl::TRIANGLES, 0, 3);

            // Unbind the VAO (not strictly necessary in this simple context, but good practice)
            gl::BindVertexArray(0);
        }
    }
}

// struct OpenWindowExample {
//     rx: Consumer<Message>,
// }

impl WindowHandler for OpenWindowExample {
    fn on_frame(&mut self, _window: &mut Window) {
        if !self.is_gl_initialized {
            self.init_gl();
            self.is_gl_initialized = true; // Set to true after initialization
        }

        self.draw();

        // while let Ok(message) = self.rx.pop() {
        //     println!("Message: {:?}", message);
        // }
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
        #[cfg(feature = "opengl")]
       // gl_config: Some(Default::default()),
        gl_config: Some(baseview::GlConfig::default()),
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
        rx,
        program: 0, // Will be initialized in the OpenWindowExample::init_gl
        vao: 0,     // Will be initialized in the OpenWindowExample::init_gl
        is_gl_initialized: false,
    });
}
