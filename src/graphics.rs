/// Lightweight abstractions over raw OpenGL calls.

use std::ffi::{c_void, CString};

use crate::gl;

extern "system" fn debug_message_callback(
    _source: u32,
    ty: u32,
    _id: u32,
    _severity: u32,
    _len: i32,
    msg: *const i8,
    _user_ptr: *mut c_void,
) {
    let display_type = match ty {
        gl::DEBUG_TYPE_ERROR => "error",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "deprecation warning",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined behavior",
        gl::DEBUG_TYPE_PORTABILITY => "portability warning",
        gl::DEBUG_TYPE_PERFORMANCE => "performance warning",
        _ => "unknown warning",
    };

    unsafe {
        let msg = std::ffi::CStr::from_ptr(msg).to_str().unwrap();
        println!("GL {}: {}", display_type, msg);
    }
}

/// Should only be called once.
pub fn init(window: &mut glfw::Window) {
    gl::load_with(|s| window.get_proc_address(s));

    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(Some(debug_message_callback), std::ptr::null());
    }
}

pub trait VertexFormat {
    fn size() -> i32;
    fn ty() -> u32;
}

impl VertexFormat for glam::Vec2 {
    fn size() -> i32 { 2 }
    fn ty() -> u32 { gl::FLOAT }
}

/// A renderable mesh with vertex and index data. Be sure to call init_layout() before add_layout()
pub struct Mesh<V> {
    vbo: u32,
    ibo: u32,
    vao: u32,
    num_indices: i32,
    pd: std::marker::PhantomData<V>,
}

impl<V> Mesh<V> {
    pub fn new() -> Mesh<V> {
        unsafe {
            let mut bufs = [0u32; 2];
            gl::CreateBuffers(2, bufs.as_mut_ptr());

            let mut vao = 0u32;
            gl::CreateVertexArrays(1, &mut vao);

            Mesh {
                vbo: bufs[0],
                ibo: bufs[1],
                vao,
                num_indices: 0,
                pd: std::marker::PhantomData,
            }
        }
    }

    pub fn set_data(&mut self, vertices: &[V], indices: &[u32]) {
        self.num_indices = indices.len() as i32;

        unsafe {
            gl::NamedBufferData(
                self.vbo,
                (vertices.len() * std::mem::size_of::<V>()) as isize,
                vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::NamedBufferData(
                self.ibo,
                (indices.len() * std::mem::size_of::<u32>()) as isize,
                indices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
        }
    }

    pub fn init_layout(&self) {
        unsafe {
            gl::VertexArrayVertexBuffer(self.vao, 0, self.vbo, 0, std::mem::size_of::<V>() as i32);
            gl::VertexArrayElementBuffer(self.vao, self.ibo);
        }
    }

    pub fn add_layout<T: VertexFormat>(&self, index: u32, offset: u32) {
        unsafe {
            gl::EnableVertexArrayAttrib(self.vao, index);
            gl::VertexArrayAttribFormat(self.vao, index, T::size(), T::ty(), gl::FALSE, offset);
            gl::VertexArrayAttribBinding(self.vao, index, 0);
        }
    }

    pub fn bind_and_render(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, self.num_indices, gl::UNSIGNED_INT, std::ptr::null());
        }
    }
}

impl<V> Drop for Mesh<V> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);

            let bufs = [self.vbo, self.ibo];
            gl::DeleteBuffers(2, bufs.as_ptr());
        }
    }
}

/// A texture with filtering disabled
pub struct PixelTexture(u32);

impl PixelTexture {
    pub fn new() -> PixelTexture {
        unsafe {
            let mut texture = 0u32;
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut texture);
            gl::TextureParameteri(texture, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TextureParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TextureParameteri(texture, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TextureParameteri(texture, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            PixelTexture(texture)
        }
    }

    pub fn set_data(&self, image: &image::DynamicImage) {
        unsafe {
            gl::TextureStorage2D(
                self.0,
                1,
                gl::RGBA8,
                image.width() as i32,
                image.height() as i32,
            );
            gl::TextureSubImage2D(
                self.0,
                0,
                0,
                0,
                image.width() as i32,
                image.height() as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                image.to_rgba8().as_ptr() as *const c_void,
            );
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.0);
        }
    }
}

impl Drop for PixelTexture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &mut self.0);
        }
    }
}

pub trait Uniform {
    fn new(location: i32) -> Self;
}

pub struct MatrixUniform(i32);

impl MatrixUniform {
    pub fn set(&self, mat: &glam::Mat4) {
        unsafe {
            gl::UniformMatrix4fv(self.0, 1, gl::FALSE, mat.to_cols_array().as_ptr());
        }
    }
}

impl Uniform for MatrixUniform {
    fn new(location: i32) -> Self {
        MatrixUniform(location)
    }
}

pub struct Program(u32);

impl Program {
    pub fn new(vertex: &str, fragment: &str) -> Program {
        unsafe {
            let program = gl::CreateProgram();

            let vsh = Self::create_shader(gl::VERTEX_SHADER, vertex);
            gl::AttachShader(program, vsh);
            let fsh = Self::create_shader(gl::FRAGMENT_SHADER, fragment);
            gl::AttachShader(program, fsh);

            gl::LinkProgram(program);

            let mut linked = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut linked);
            if linked != gl::TRUE as i32 {
                panic!("failed to link program");
            }

            Self::discard_shader(program, vsh);
            Self::discard_shader(program, fsh);
            Program(program)
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.0);
        }
    }

    pub fn get_uniform<U: Uniform>(&self, name: &str) -> U {
        unsafe {
            let name_cstr = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.0, name_cstr.as_ptr());
            U::new(location)
        }
    }

    unsafe fn create_shader(ty: u32, src: &str) -> u32 {
        let shader = gl::CreateShader(ty);
        let src_cstr = CString::new(src).unwrap();
        gl::ShaderSource(shader, 1, &src_cstr.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        let mut compiled = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compiled);
        if compiled != gl::TRUE as i32 {
            panic!("shader failed to compile");
        }

        shader
    }

    unsafe fn discard_shader(program: u32, shader: u32) {
        gl::DetachShader(program, shader)
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.0);
        }
    }
}
