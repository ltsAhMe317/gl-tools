use define::*;
use gl::types;
use gl::types::{GLenum, GLint, GLintptr, GLsizei, GLsizeiptr, GLuint};
use glam::IVec2;
use glfw::Context;
use image::ImageFormat;

use core::panic;

use std::ffi::c_void;
use std::marker::PhantomData;
use std::path::Path;
use texture::{Texture, Texture2D, TextureWrapper};

pub mod buffer;
pub mod define;
pub mod program;
pub mod texture;
pub mod window;

use window::Window;

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
        Self {}
    }
    pub fn clear(&self, buffer: &FrameBuffer) {
        let id = buffer.frame_buffer as GLint;
        unsafe {
            gl::ClearBufferfv(gl::DEPTH, id, CLEAN_COLOR.as_ptr());
            gl::ClearBufferfv(gl::COLOR, id, CLEAN_COLOR.as_ptr());
            gl::ClearBufferfv(gl::STENCIL, id, CLEAN_COLOR.as_ptr());
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

    pub fn get_now_bind_id() -> GLint {
        unsafe {
            let mut id: GLint = 0;
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut id as *mut GLint);
            id
        }
    }
    pub fn get_now_attachment_id(attachment: GLenum) -> GLint {
        // self.bind(gl::FRAMEBUFFER);
        unsafe {
            let mut id: GLint = 0;
            gl::GetFramebufferAttachmentParameteriv(
                gl::FRAMEBUFFER,
                attachment,
                gl::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME,
                &mut id as *mut GLint,
            );
            id
        }
    }
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
    pub fn with<T>(
        &self,
        date: &VertexBuffer<T>,
        index: types::GLuint,
        once_count: types::GLint,
        date_type: types::GLenum,
        after: usize,
    ) {
        self.bind_with(date);
        date.with(index, once_count, date_type, after);
    }
    pub fn bind_with<T>(&self, date: &VertexBuffer<T>) {
        self.bind();
        date.bind();
    }
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.array_id);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::BindVertexArray(0);
            gl::DeleteVertexArrays(1, &self.array_id as *const GLuint);
        }
    }
}

pub struct VertexBuffer<T> {
    date_id: GLuint,
    target: GLenum,
    type_const: PhantomData<T>,
    size: usize,
}

impl<T> VertexBuffer<T> {
    pub fn new(target: GLenum, vertex: &[T], size: usize, save_mod: GLenum) -> Self {
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(target, id);
            gl::BufferData(
                target,
                (size * std::mem::size_of::<T>()) as GLsizeiptr,
                vertex.as_ptr() as *const c_void,
                save_mod,
            );
        }
        VertexBuffer {
            date_id: id,
            target,
            type_const: PhantomData,
            size: vertex.len(),
        }
    }
    pub fn sub(&self, vertex: &[T], offset: GLint) {
        if self.size < vertex.len() {
            panic!("vec's len bigger than vertex")
        }
        self.bind();
        unsafe {
            gl::BufferSubData(
                self.target,
                offset as GLintptr,
                std::mem::size_of_val(vertex) as GLsizeiptr,
                vertex.as_ptr() as *const c_void,
            )
        }
    }
    pub fn new_array(vertex: &[T], save_mod: GLenum) -> Self {
        Self::new(gl::ARRAY_BUFFER, vertex, vertex.len(), save_mod)
    }
    pub fn new_array_size(vertex: &[T], size: usize, save_mod: GLenum) -> Self {
        Self::new(gl::ARRAY_BUFFER, vertex, size, save_mod)
    }
    pub fn with(
        &self,
        index: types::GLuint,
        once_count: types::GLint,
        date_type: types::GLenum,
        after: usize,
    ) {
        self.bind();
        unsafe {
            gl::EnableVertexAttribArray(index);
            gl::VertexAttribPointer(
                index,
                once_count,
                date_type,
                gl::FALSE,
                (size_of::<T>() as i32) * once_count,
                after as *const c_void,
            );
        }
    }
    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(self.target, self.date_id);
        }
    }
}

impl<T> Drop for VertexBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::BindBuffer(self.target, 0);
            gl::DeleteBuffers(1, &self.date_id as *const GLuint);
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
