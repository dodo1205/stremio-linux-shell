mod utils;

use std::ptr;

use gl::types::{GLint, GLsizeiptr, GLuint};
use utils::{compile_shader, create_fbo, create_geometry, create_pbo, create_texture};

const FRAGMENT_SRC: &str = include_str!("shader.frag");
const VERTEX_SRC: &str = include_str!("shader.vert");
const BYTES_PER_PIXEL: usize = 4;

#[derive(Debug)]
pub struct Renderer {
    pub program: GLuint,
    pub front_texture: GLuint,
    pub back_texture: GLuint,
    pub vao: GLuint,
    pub vbo: GLuint,
    pub fbo: GLuint,
    pub pbos: [GLuint; 2],
    pub pbo_index: usize,
    pub width: i32,
    pub height: i32,
    pub refresh_rate: u32,
}

impl Renderer {
    pub fn new((width, height): (i32, i32), refresh_rate: u32) -> Self {
        unsafe {
            let vertex_shader = compile_shader(gl::VERTEX_SHADER, VERTEX_SRC);
            let fragment_shader = compile_shader(gl::FRAGMENT_SHADER, FRAGMENT_SRC);
            let program = gl::CreateProgram();

            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);

            gl::LinkProgram(program);
            gl::UseProgram(program);

            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            let front_texture = create_texture(width, height);
            let back_texture = create_texture(width, height);

            let (vao, vbo) = create_geometry(program);
            let fbo = create_fbo(back_texture);

            let pbos = [create_pbo(width, height), create_pbo(width, height)];

            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                panic!("Framebuffer not complete: {}", status);
            }

            Self {
                program,
                front_texture,
                back_texture,
                vao,
                vbo,
                fbo,
                pbos,
                pbo_index: 0,
                width,
                height,
                refresh_rate,
            }
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.width = width;
            self.height = height;

            gl::Viewport(0, 0, width, height);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            for index in 0..self.pbos.len() {
                gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, self.pbos[index]);

                let mut pbo_size = 0;
                gl::GetBufferParameteriv(gl::PIXEL_UNPACK_BUFFER, gl::BUFFER_SIZE, &mut pbo_size);

                let new_size = width * height * 4;
                if new_size > pbo_size {
                    gl::BufferData(
                        gl::PIXEL_UNPACK_BUFFER,
                        new_size as GLsizeiptr,
                        ptr::null(),
                        gl::STREAM_READ,
                    );
                }

                gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);
            }

            gl::BindTexture(gl::TEXTURE_2D, self.back_texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as GLint,
                width,
                height,
                0,
                gl::BGRA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );

            gl::BindTexture(gl::TEXTURE_2D, self.front_texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as GLint,
                width,
                height,
                0,
                gl::BGRA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );
        }
    }

    pub fn paint(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        buffer: *const u8,
        full_width: i32,
    ) {
        unsafe {
            gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, self.pbos[self.pbo_index]);

            let row_bytes = width as usize * BYTES_PER_PIXEL;
            let stride = full_width as usize * BYTES_PER_PIXEL;

            let ptr = gl::MapBuffer(gl::PIXEL_UNPACK_BUFFER, gl::WRITE_ONLY) as *mut u8;
            if !ptr.is_null() {
                for row in 0..height as usize {
                    let src_offset = (y as usize + row) * stride + (x as usize * BYTES_PER_PIXEL);
                    let dst_offset = row * row_bytes;

                    let src_ptr = buffer.add(src_offset);
                    let dst_ptr = ptr.add(dst_offset);

                    ptr::copy_nonoverlapping(src_ptr, dst_ptr, row_bytes);
                }

                gl::UnmapBuffer(gl::PIXEL_UNPACK_BUFFER);
            }

            gl::BindTexture(gl::TEXTURE_2D, self.front_texture);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                x,
                y,
                width,
                height,
                gl::BGRA,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );

            gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);

            self.pbo_index = (self.pbo_index + 1) % self.pbos.len();
        }
    }

    pub fn draw(&self) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            gl::BlendEquation(gl::FUNC_ADD);

            gl::UseProgram(self.program);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.back_texture);
            gl::Uniform1i(
                gl::GetUniformLocation(self.program, c"back_texture".as_ptr() as _),
                0,
            );

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.front_texture);
            gl::Uniform1i(
                gl::GetUniformLocation(self.program, c"front_texture".as_ptr() as _),
                1,
            );

            gl::BindVertexArray(self.vao);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteTextures(1, &self.front_texture);
            gl::DeleteTextures(1, &self.back_texture);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(2, self.pbos.as_ptr());
            gl::DeleteBuffers(1, &self.fbo);
        }
    }
}
