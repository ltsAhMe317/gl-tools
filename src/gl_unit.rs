use define::*;
use gl::types;
use gl::types::{GLenum, GLint, GLintptr, GLsizei, GLsizeiptr, GLuint};
use glam::IVec2;
use glfw::Context;
use image::ImageFormat;

use core::panic;

use std::ffi::c_void;
use std::path::Path;
use std::ptr::null;
use texture::{Texture, Texture2D, TextureWrapper};

pub mod buffer;
pub mod debug;
pub mod define;
pub mod program;
pub mod texture;
pub mod window;

use window::Window;

use crate::{Buffer, TypeGL};
extern "system" fn debug_callback(
    source: gl::types::GLenum,
    gltype: gl::types::GLenum,
    id: gl::types::GLuint,
    severity: gl::types::GLenum,
    length: gl::types::GLsizei,
    message: *const gl::types::GLchar,
    userParam: *mut std::ffi::c_void,
) {
    let msg = unsafe { std::ffi::CStr::from_ptr(message).to_string_lossy() };

    if severity == gl::DEBUG_SEVERITY_HIGH {
        eprintln!("[OpenGL HIGH] {}", msg);
    } else {
        println!("[OpenGL] {}", msg);
    }
}

pub fn view_port(x: i32, y: i32, w: i32, h: i32) {
    unsafe {
        gl::Viewport(x, y, w, h);
    }
}

static CLEAN_COLOR: [f32; 4] = [0f32, 0f32, 0f32, 1f32];

pub struct GLcontext {}
unsafe impl Send for GLcontext {}

impl GLcontext {
    pub fn with(window: &mut Window) -> Self {
        window.window.make_current();
        gl::load_with(|s| window.window.glfw.get_proc_address_raw(s));
        let mut screen_w = 0;
        let mut resize_h = 0;

        window.window.glfw.with_primary_monitor(|_, monitor| {
            let screen = monitor.unwrap().get_video_mode().unwrap();
            screen_w = screen.width;
            resize_h = screen.height;
        });

        let (w, h) = window.window.get_size();
        let asp = w as f32 / h as f32;
        let width = resize_h as f32 * asp;

        let pos_x = (screen_w as f32 - width) / 2.0f32;
        if window.window.is_maximized() {
            unsafe {
                gl::Viewport(pos_x as GLint, 0, width as GLsizei, resize_h as GLsizei);
            }
        }
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(debug_callback), null());
        }
        Self {}
    }
    pub fn clear(&self, buffer: &FrameBuffer) {
        let id = buffer.frame_buffer as GLint;
        unsafe {
            gl::ClearBufferfv(gl::DEPTH, id, CLEAN_COLOR.as_ptr());
            gl::ClearBufferfv(gl::COLOR, id, CLEAN_COLOR.as_ptr());
            // gl::ClearBufferfv(gl::STENCIL_BUFFER_BIT, id, CLEAN_COLOR.as_ptr());
        }
    }
    pub fn base_clear(&self) {
        self.clear(&FrameBuffer {
            frame_buffer: 0,
            render: None,
            texture: None,
        });
    }
    pub fn view_size(&self, x: i32, y: i32, w: i32, h: i32) {
        unsafe {
            gl::Viewport(x, y, w, h);
        }
    }

    pub fn draw(&mut self, window: &mut Window, func: impl FnOnce(&mut GLcontext, &mut Window)) {
        func(self, window)
    }
    pub fn draw_option(
        &mut self,
        window: &mut Window,
        func: impl FnOnce(&mut GLcontext, &mut Window),
    ) {
        let window_size = window.window.get_size();
        self.view_size(0, 0, window_size.0, window_size.1);
        self.base_clear();
        self.draw(window, func);
    }
}

pub struct RenderBuffer {
    pub w: u32,
    pub h: u32,
    pub render_buffer: GLuint,
}

impl RenderBuffer {
    pub fn new(type_: GLenum, w: u32, h: u32) -> Self {
        let mut id = 0;
        unsafe {
            gl::GenRenderbuffers(1, &mut id);
            gl::BindRenderbuffer(gl::RENDERBUFFER, id);
            gl::RenderbufferStorage(gl::RENDERBUFFER, type_, w as GLsizei, h as GLsizei);
        }
        RenderBuffer {
            w,
            h,
            render_buffer: id,
        }
    }
}

pub fn debug_frame_buffer(frame_buffer: &FrameBuffer) {
    if let Some(texture) = frame_buffer.texture.as_ref() {
        let image = texture.get_image();
        image
            .save_with_format(Path::new("./debug/framebuffer.png"), ImageFormat::Png)
            .expect("err save framebuffer");
    }
}
pub struct FrameBuffer {
    pub frame_buffer: GLuint,
    pub render: Option<RenderBuffer>,
    pub texture: Option<TextureWrapper<Texture2D>>,
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameBuffer {
    pub fn new() -> Self {
        let mut id = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut id);
        }
        Self {
            frame_buffer: id,
            texture: None,
            render: None,
        }
    }
    pub fn view_port(&self) {
        unsafe {
            if let Some(texture) = self.texture.as_ref() {
                gl::Viewport(0, 0, texture.w as GLsizei, texture.h as GLsizei);
            }
            if let Some(render) = self.render.as_ref() {
                gl::Viewport(0, 0, render.w as GLsizei, render.h as GLsizei);
            }
        }
    }
    pub fn link_texture(
        &mut self,
        texture: texture::TextureWrapper<Texture2D>,
        attachment: GLenum,
    ) {
        unsafe {
            self.bind(gl::FRAMEBUFFER);
            texture.send_to_texture();
            self.texture = Option::Some(texture);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                attachment,
                gl::TEXTURE_2D,
                self.texture.as_ref().unwrap().texture,
                0,
            );
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                panic!("framebuffer error:{}", status)
            }
        }
    }
    pub fn link_buffer(&mut self, w: u32, h: u32, attachment: GLenum, _type: GLenum) {
        self.bind(gl::FRAMEBUFFER);
        let buffer = RenderBuffer::new(_type, w, h);
        let id = buffer.render_buffer;
        self.render = Some(buffer);
        unsafe {
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, attachment, gl::RENDERBUFFER, id);
        }
    }
    pub fn bind(&self, type_: GLenum) {
        unsafe {
            gl::BindFramebuffer(type_, self.frame_buffer);
        }
    }
    pub fn clear(&self) {
        self.bind(gl::FRAMEBUFFER);
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
    pub fn get_size(&self) -> Option<IVec2> {
        if let Some(texture) = self.texture.as_ref() {
            return Some(IVec2::new(texture.w as i32, texture.h as i32));
        }
        if let Some(render) = self.render.as_ref() {
            return Some(IVec2::new(render.w as i32, render.h as i32));
        }
        None
    }

    pub fn unbind() {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn blit(
        &self,
        other: &Self,
        src_down: IVec2,
        src_up: IVec2,
        dst_down: IVec2,
        dst_up: IVec2,
        attachment: GLenum,
        mode: GLenum,
    ) {
        self.bind(gl::READ_FRAMEBUFFER);
        other.bind(gl::DRAW_FRAMEBUFFER);
        unsafe {
            gl::BlitFramebuffer(
                src_down.x, src_down.y, src_up.x, src_up.y, dst_down.x, dst_down.y, dst_up.x,
                dst_up.y, attachment, mode,
            );
        }
    }
    pub fn blit_all(
        &self,
        other: &Self,
        src_size: IVec2,
        dst_size: IVec2,
        attachment: GLenum,
        mode: GLenum,
    ) {
        self.blit(
            other,
            IVec2::new(0, src_size.y),
            IVec2::new(src_size.x, 0),
            IVec2::new(0, dst_size.y),
            IVec2::new(dst_size.x, 0),
            attachment,
            mode,
        )
    }

    // pub fn get_now_bind_id() -> GLint {
    //     unsafe {
    //         let mut id: GLint = 0;
    //         gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut id as *mut GLint);
    //         id
    //     }
    // }
    // pub fn get_now_attachment_id(attachment: GLenum) -> GLint {
    //     // self.bind(gl::FRAMEBUFFER);
    //     unsafe {
    //         let mut id: GLint = 0;
    //         gl::GetFramebufferAttachmentParameteriv(
    //             gl::FRAMEBUFFER,
    //             attachment,
    //             gl::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME,
    //             &mut id as *mut GLint,
    //         );
    //         id
    //     }
    // }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        Self::unbind();
        unsafe {
            gl::DeleteFramebuffers(1, &self.frame_buffer);
        }
    }
}

pub struct VertexArray {
    array_id: GLuint,
}

impl Default for VertexArray {
    fn default() -> Self {
        Self::new()
    }
}
impl VertexArray {
    pub fn new() -> Self {
        let mut id = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        Self { array_id: id }
    }

    pub fn bind_set<T: TypeGL>(&self, date: &Buffer<T>, pointer: VertexArrayAttribPointerGen) {
        if date.target != BufferTarget::Vertex {
            panic!("[VAO err]buffer target != vertex");
        }
        self.bind();
        date.bind_target();

        let (index, once_size, is_normalized, stride, pointer) = (
            pointer.index,
            pointer.once_size,
            pointer.is_normalized,
            pointer.stride,
            pointer.pointer,
        );

        unsafe {
            gl::EnableVertexAttribArray(index);
            gl::VertexAttribPointer(
                index,
                once_size,
                T::as_gl(),
                if is_normalized { gl::TRUE } else { gl::FALSE },
                stride,
                &pointer as *const u32 as *const c_void,
            );
        }
        date.unbind_target();
    }
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.array_id);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        println!("delete vao");
        unsafe {
            gl::BindVertexArray(0);
            gl::DeleteVertexArrays(1, &self.array_id as *const GLuint);
        }
    }
}

fn blend(src: Blend, dst: Blend) {
    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(src.as_gl(), dst.as_gl());
    }
}

pub enum ConstBlend {
    SrcOnly,
    // 正常混合 (src alpha, 1 - src alpha)
    Normal,
    // 加法混合 (src color + dst color)
    Additive,
    // 乘法混合 (src color * dst color)
    Multiply,
    // 屏幕混合 (1 - (1 - src color) * (1 - dst color))
    Screen,
    // 叠加混合 (根据底色决定 multiply 或 screen)
    Overlay,
    // 预乘 alpha 混合 (src alpha, 1)
    Premultiplied,
    // 自定义混合模式
    Custom(Blend, Blend),
}

impl ConstBlend {
    pub const fn blend(self) -> (Blend, Blend) {
        match self {
            ConstBlend::Normal => (Blend::SrcAlpha, Blend::OneMinusSrcAlpha),
            ConstBlend::Additive => (Blend::SrcAlpha, Blend::One),
            ConstBlend::Multiply => (Blend::DstColor, Blend::Zero),
            ConstBlend::Screen => (Blend::One, Blend::OneMinusSrcColor),
            ConstBlend::Overlay => (Blend::One, Blend::OneMinusSrcAlpha),
            ConstBlend::Premultiplied => (Blend::One, Blend::OneMinusSrcAlpha),
            ConstBlend::Custom(src, dst) => (src, dst),
            ConstBlend::SrcOnly => (Blend::One, Blend::Zero),
        }
    }
}

pub fn const_blend(b: ConstBlend) {
    let (src, dst) = b.blend();
    blend(src, dst);
}

pub fn polygon_mode(face: Face, mode: PolygonMode) {
    unsafe {
        gl::PolygonMode(face.as_gl(), mode.as_gl());
    }
}

pub fn flush() {
    unsafe {
        gl::Flush();
    }
}
pub fn finish() {
    unsafe {
        gl::Finish();
    }
}
