use gl::types;
use gl::types::{GLenum, GLint, GLintptr, GLsizei, GLsizeiptr, GLuint};
use glam::{IVec2, Mat3, Mat4};
use glfw::Context;
use image::ImageFormat;
use json::JsonValue;

use core::panic;

use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::fs;
use std::marker::PhantomData;
use std::ops::Add;
use std::path::Path;
use std::sync::LazyLock;
use texture::{Texture, Texture2D, TextureWrapper};

pub mod define;
pub mod texture;
pub mod window;

use window::Window;

pub fn view_port(x: i32, y: i32, w: i32, h: i32) {
    unsafe {
        gl::Viewport(x, y, w, h);
    }
}

const PROGRAM2D_VERT_TWO: &str = "
    #version 330
    layout (location = 0) in vec2 vert;
    layout (location = 1) in vec2 uv;
    out vec2 tex_uv;
    uniform mat4 project_mat;
    uniform mat4 model_mat;
    void main(){
        gl_Position = project_mat * model_mat * vec4(vert,0,1);
        tex_uv = uv;
    }
";
const PROGRAM2D_FRAG_TWO: &str = "
    #version 330
    in vec2 tex_uv;
    uniform sampler2D image;
    out vec4 color;
    void main(){
        color = texture(image,tex_uv);
      }
";

pub static PROGRAM2D_TWO: LazyLock<Program> =
    LazyLock::new(|| Program::basic_new(PROGRAM2D_VERT_TWO, PROGRAM2D_FRAG_TWO, None));

const PROGRAM2D_VERT_ONE: &str = "
    #version 330
    layout (location = 0) in vec4 vert;
    out vec2 uv;
    uniform mat4 project_mat;
    uniform mat4 model_mat;
    void main(){
        uv = vert.zw;
    
        gl_Position = project_mat * model_mat * vec4(vert.xy,0,1);
     }
";
const PROGRAM2D_FRAG_ONE: &str = "
    #version 330
    in vec2 uv;
    uniform sampler2D image;
    out vec4 color;
    void main(){
        color = texture(image,uv);
        // color = vec4(1,1,1,1);
    }
";

pub static PROGRAM2D_ONE: LazyLock<Program> =
    LazyLock::new(|| Program::basic_new(PROGRAM2D_VERT_ONE, PROGRAM2D_FRAG_ONE, None));

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

pub struct Shader {
    shader_id: GLuint,
}

impl Shader {
    pub fn load_file(path: &Path, type_: GLenum) -> Self {
        Self::load(&fs::read_to_string(path).unwrap(), type_)
    }
    pub fn load(code: &str, type_: GLenum) -> Self {
        let id;
        let code = std::ffi::CString::new(code).unwrap();
        unsafe {
            id = gl::CreateShader(type_);
            gl::ShaderSource(
                id,
                1,
                &code.as_ptr() as *const *const c_char,
                std::ptr::null(),
            );
            gl::CompileShader(id);
        }
        Shader { shader_id: id }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.shader_id) }
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

static mut GENE_VERT: Option<VertexBuffer<f32>> = None;
static mut GENE_FLIP_TEX_VERT: Option<VertexBuffer<f32>> = None;
static mut GENE_FLIP_VAO: Option<VertexArray> = None;

pub static mut GENE_IN_TEXTURE: Option<FrameBuffer> = None;
pub static mut GENE_OUT_TEXTURE: Option<FrameBuffer> = None;
const GENE_TEXTURE_W: u32 = 800;
const GENE_TEXTURE_H: u32 = 600;

struct ProgramPass {
    program: Program,
    in_out: (String, String),
    uniform: Option<Vec<(String, JsonValue)>>,
    clear: bool,
}

pub struct ProgramGene {
    pub frames: HashMap<String, FrameBuffer>,
    pass: Vec<(String, ProgramPass)>,
}
impl ProgramGene {
    pub fn new(json: &str, programs: &HashMap<String, Program>) -> Self {
        let mut post = Self {
            frames: HashMap::new(),
            pass: Vec::new(),
        };

        let json = json::parse(json).unwrap();
        let frames = &json["frame"];
        let pass = &json["pass"];

        for frame_size in 0..frames.len() {
            let name = frames[frame_size]["name"].as_str().unwrap().to_string();
            let size = {
                let vec = &frames[frame_size]["size"];
                (vec[0].as_u32().unwrap(), vec[1].as_u32().unwrap())
            };
            post.frames.insert(name, {
                let mut buffer = FrameBuffer::new();
                let texture = TextureWrapper(texture::Texture2D::with_size(
                    size.0,
                    size.1,
                    texture::TextureType::RGBA8,
                    texture::TextureParm::new(),
                ));
                buffer.link_texture(texture, gl::COLOR_ATTACHMENT0);
                //
                // FrameBuffer::get_now_bind_id();
                // FrameBuffer::get_now_attachment_id(gl::COLOR_ATTACHMENT0);
                // eprintln!("im link!!!");
                // eprintln!("id:{} link:{}",FrameBuffer::get_now_bind_id(),FrameBuffer::get_now_attachment_id(gl::COLOR_ATTACHMENT0));
                buffer
            });
        }

        for pass_size in 0..pass.len() {
            let pass = &pass[pass_size];
            let name = pass["name"].as_str().unwrap().to_string();
            let uniform = {
                let uniform = &pass["uniforms"];
                if uniform.is_null() {
                    None
                } else {
                    let mut vec = Vec::new();
                    for i in 0..uniform.len() {
                        let name = uniform[i]["name"].as_str().unwrap().to_string();
                        let value = uniform[i]["value"].clone();
                        vec.push((name, value))
                    }
                    Some(vec)
                }
            };
            let tex_in = pass["in"].as_str().unwrap().to_string();
            let tex_out = pass["out"].as_str().unwrap().to_string();

            let clear = pass["clear"].as_bool().unwrap();

            let pass = ProgramPass {
                program: programs.get(&name).unwrap().clone(),
                in_out: (tex_in, tex_out),
                uniform,
                clear,
            };
            post.pass.push((name, pass));
        }

        post
    }
    pub fn gene(&self, _in: &FrameBuffer, _out: &FrameBuffer) {
        unsafe {
            if GENE_IN_TEXTURE.is_none() {
                let mut frame = FrameBuffer::new();
                frame.link_texture(
                    TextureWrapper(texture::Texture2D::with_size(
                        GENE_TEXTURE_W,
                        GENE_TEXTURE_H,
                        texture::TextureType::RGBA8,
                        texture::TextureParm::new(),
                    )),
                    gl::COLOR_ATTACHMENT0,
                );
                GENE_IN_TEXTURE = Some(frame);

                let mut frame = FrameBuffer::new();
                frame.link_texture(
                    TextureWrapper(texture::Texture2D::with_size(
                        GENE_TEXTURE_W,
                        GENE_TEXTURE_H,
                        texture::TextureType::RGBA8,
                        texture::TextureParm::new(),
                    )),
                    gl::COLOR_ATTACHMENT0,
                );
                GENE_OUT_TEXTURE = Some(frame);

                let vao = VertexArray::new();
                let vbo_tex = VertexBuffer::new_array(
                    &[0f32, 1f32, 1f32, 1f32, 1f32, 0f32, 0f32, 0f32],
                    gl::STATIC_DRAW,
                );
                let vbo_vert = VertexBuffer::new_array(
                    &[-1f32, 1f32, 1f32, 1f32, 1f32, -1f32, -1f32, -1f32],
                    gl::STATIC_DRAW,
                );
                vao.with(&vbo_vert, 0, 2, gl::FLOAT, 0);
                vao.with(&vbo_tex, 1, 2, gl::FLOAT, 0);
                GENE_VERT = Some(vbo_vert);
                GENE_FLIP_TEX_VERT = Some(vbo_tex);
                GENE_FLIP_VAO = Some(vao);
            }
        }

        let in_frame_size = _in.get_size().unwrap();
        let out_frame_size = _out.get_size().unwrap();
        unsafe {
            _in.blit_all(
                GENE_IN_TEXTURE.as_ref().unwrap(),
                in_frame_size,
                IVec2::new(GENE_TEXTURE_W as i32, GENE_TEXTURE_H as i32),
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );
        }

        for (_, program) in self.pass.iter() {
            if let Some(uniforms) = program.uniform.as_ref() {
                let program = &program.program;
                program.bind();
                for (uniform_name, uniform_value) in uniforms.iter() {
                    if uniform_value[0].as_f32().is_some() {
                        let mut vec = Vec::with_capacity(uniform_value.len());
                        for i in 0..uniform_value.len() {
                            vec.push(uniform_value[i].as_f32().unwrap());
                        }
                        program.put_f32(uniform_name, &vec);
                        continue;
                    }
                    if let Some(bool) = uniform_value[0].as_bool() {
                        program.put_bool(program.get_uniform(uniform_name), bool);
                        continue;
                    }
                }
            }
            let in_out = &program.in_out;

            let in_tex = self.get_frame(&in_out.0);
            let out_tex = self.get_frame(&in_out.1);
            out_tex.bind(gl::FRAMEBUFFER);
            if program.clear {
                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }
            }
            in_tex.texture.as_ref().unwrap().bind_unit(0);
            program
                .program
                .put_texture(0, program.program.get_uniform("image_in"));

            unsafe {
                // let in_size = in_tex.texture.as_ref().unwrap().get_size();
                // let out_size = out_tex.texture.as_ref().unwrap().get_size();
                // if in_size.x * in_size.y < out_size.x * out_size.y{
                //     GENE_FLIP_TEX_VERT.as_mut().unwrap().sub( &[0f32,1f32,1f32,1f32,1f32,0f32,0f32,0f32],0);
                // }else {
                //     let x =  (in_size.x /out_size.x);
                //     let y =  in_size.y/out_size.y;
                //     //mode 2
                //     let x =   (out_size.x/in_size.x);
                // println!("{},{}",x,y);
                // let x = 0.05f32;
                // let y =  out_size.y/in_size.y;
                //

                // GENE_VERT.as_mut().unwrap().sub(&[-1f32, 1f32, x, 1f32, x, -1f32, -1f32, -1f32],0);
                // GENE_VERT.as_mut().unwrap().sub(&[-x, y, x, y, x, -y, -x, -y],0);
                //     GENE_FLIP_TEX_VERT.as_mut().unwrap().sub(&[0f32,y,x,y,x,0f32,0f32,0f32],0);
                // }

                out_tex.view_port();
                GENE_FLIP_VAO.as_ref().unwrap().bind();
            }
            program.program.draw_rect(1);

            //debug
            // unsafe {
            //     let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            //     println!("{} to {}:frame_id:{} status:{} attachment:{}", in_out.0, in_out.1, FrameBuffer::get_now_bind_id(), status, FrameBuffer::get_now_attachment_id(gl::COLOR_ATTACHMENT0));
            // }
        }

        unsafe {
            GENE_OUT_TEXTURE.as_ref().unwrap().blit_all(
                _out,
                IVec2::new(GENE_TEXTURE_W as i32, GENE_TEXTURE_H as i32),
                out_frame_size,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );
            GENE_OUT_TEXTURE.as_ref().unwrap().bind(gl::FRAMEBUFFER);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn gene_def(&self, window: &Window) {
        let size = window.window.get_framebuffer_size();
        let base_frame = FrameBuffer {
            frame_buffer: 0,
            render: Some(RenderBuffer {
                w: size.0 as u32,
                h: size.1 as u32,
                render_buffer: 0,
            }),
            texture: None,
        };
        self.gene(&base_frame, &base_frame)
    }
    pub fn load(path: &Path, programs: &HashMap<String, Program>) -> Self {
        Self::new(&fs::read_to_string(path).unwrap(), programs)
    }
    pub fn get_frame(&self, name: &str) -> &FrameBuffer {
        unsafe {
            if name.eq("in") {
                return GENE_IN_TEXTURE.as_ref().unwrap();
            }
            if name.eq("out") {
                return GENE_OUT_TEXTURE.as_ref().unwrap();
            }
        }
        self.frames.get(name).expect(name)
    }
}

#[derive(Clone)]
pub struct Program {
    program_id: GLuint,
    uniform: JsonValue,
}

impl Program {
    pub fn get_uniform(&self, name: &str) -> GLint {
        unsafe {
            let id = gl::GetUniformLocation(self.program_id, CString::new(name).unwrap().as_ptr());
            if id == -1 {
                println!("can't find {} in program", name)
            }
            id
        }
    }
    pub fn load(path: &Path) -> Self {
        let json = json::parse(&fs::read_to_string(path).unwrap()).unwrap();
        let path_str = path.to_str().unwrap();
        let path = &path_str[..path_str.rfind("\\").unwrap() + 1];
        Self::basic_new(
            fs::read_to_string(Path::new(
                path.to_string()
                    .add(json["vert"].as_str().unwrap())
                    .add(".vert")
                    .as_str(),
            ))
            .ok()
            .as_ref()
            .unwrap(),
            fs::read_to_string(Path::new(
                path.to_string()
                    .add(json["frag"].as_str().unwrap())
                    .add(".frag")
                    .as_str(),
            ))
            .ok()
            .as_ref()
            .unwrap(),
            Some(json["uniforms"].clone()),
        )
    }
    pub fn basic_new(vert: &str, frag: &str, uniform: Option<JsonValue>) -> Self {
        unsafe {
            let program_id = gl::CreateProgram();

            //vert
            gl::AttachShader(program_id, Shader::load(vert, gl::VERTEX_SHADER).shader_id);

            //frag
            gl::AttachShader(
                program_id,
                Shader::load(frag, gl::FRAGMENT_SHADER).shader_id,
            );

            let mut len: GLsizei = 1;
            let mut code: Vec<c_char> = Vec::with_capacity(1000);
            gl::LinkProgram(program_id);
            gl::GetProgramInfoLog(program_id, 1000, &mut len, code.as_mut_ptr());
            let cstr = CStr::from_ptr(code.as_ptr());
            let str_slice = cstr.to_str().expect("Failed to convert CStr to &str");
            if !str_slice.is_empty() {
                panic!("Program err: {}", str_slice);
            }

            Program {
                program_id,
                uniform: uniform.unwrap_or(JsonValue::Null),
            }
        }
    }
    pub fn put_bool(&self, id: GLint, bool: bool) {
        let num = match bool {
            true => 1,
            false => 0,
        };
        unsafe {
            gl::Uniform1i(id, num);
        }
    }
    pub fn draw(&self, mode: GLenum, count: GLsizei) {
        unsafe {
            gl::DrawArrays(mode, 0, count);
        }
    }
    pub fn draw_rect(&self, count: GLsizei) {
        self.draw(gl::QUADS, count * 4);
    }
    pub fn put_matrix3(&self, date: &Mat3, id: GLint) {
        unsafe {
            gl::UniformMatrix3fv(id, 1, gl::FALSE, date.as_ref().as_ptr());
        }
    }
    pub fn put_matrix(&self, date: &Mat4, id: GLint) {
        unsafe {
            gl::UniformMatrix4fv(id, 1, gl::FALSE, date.as_ref().as_ptr());
        }
    }
    pub fn put_matrix_name(&self, date: &Mat4, name: &str) {
        self.put_matrix(date, self.get_uniform(name));
    }
    pub fn put_vec4(&self, date: [f32; 4], id: GLint) {
        unsafe { gl::Uniform4f(id, date[0], date[1], date[2], date[3]) }
    }
    pub fn put_one(&self, date: f32, id: GLint) {
        unsafe {
            gl::Uniform1f(id, date);
        }
    }
    pub fn put_vec(&self, date: &[f32], id: GLint) {
        unsafe {
            gl::Uniform1fv(id, date.len() as GLsizei, date.as_ptr());
        }
    }
    pub fn put_f32(&self, name: &str, date: &Vec<f32>) {
        let id = self.get_uniform(name);
        match date.len() {
            1 => self.put_one(date[0], id),

            4 => self.put_vec4([date[0], date[1], date[2], date[3]], id),
            _ => {
                self.put_vec(date, id);
            }
        }
    }
    pub fn put_texture(&self, tex_unit: GLint, id: GLint) {
        unsafe {
            gl::Uniform1i(id, tex_unit);
        }
    }
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program_id);
            // gl::Enable(gl::BLEND);
            // gl::BlendEquation(Self::get_enum(self.blend_func["func"].as_str().unwrap()));
            // let src = Self::get_enum(self.blend_func["srcrgb"].as_str().unwrap());
            // let dst = Self::get_enum(self.blend_func["dstrgb"].as_str().unwrap());
            // gl::BlendFunc(src, dst);
        }
    }
    pub fn load_uniform(&self) {
        self.bind();
        for i in 0..self.uniform.len() {
            let mut temp = self.uniform[i].clone();
            let name = temp["name"].as_str().unwrap().to_string();
            let uniform_type = temp["type"].as_str().unwrap().to_string();
            let value_count = temp["count"].as_usize().unwrap();
            let mut value = Vec::with_capacity(value_count);
            let vec = temp["values"].take();
            for j in 0..value_count {
                if let Some(num) = vec[j].as_f32() {
                    value.push(num);
                }
            }
            unsafe {
                let name_id = self.get_uniform(&name);
                match uniform_type.as_str() {
                    "vec4" => {
                        let vec = [
                            vec[0].as_f32().unwrap(),
                            vec[1].as_f32().unwrap(),
                            vec[2].as_f32().unwrap(),
                            vec[3].as_f32().unwrap(),
                        ];
                        self.put_vec4(vec, name_id)
                    }

                    "float" => self.put_one(value[0], name_id),
                    "matrix4x4" => {
                        self.put_matrix(
                            &Mat4::from_cols_array(&(value.as_slice().try_into().unwrap())),
                            name_id,
                        );
                        // gl::UniformMatrix4fv(name_id, 1, gl::FALSE, value.as_ptr());
                    }
                    "matrix3x3" => {
                        gl::UniformMatrix3fv(name_id, 1, gl::FALSE, value.as_ptr());
                    }
                    "bool" => {
                        let some;
                        if vec[0].as_str().unwrap().eq("true") {
                            some = 1;
                        } else {
                            some = 0;
                        }
                        gl::Uniform1i(name_id, some);
                    }
                    &_ => {}
                }
            }
        }
    }
    fn get_enum(str: &str) -> GLenum {
        match str {
            "add" => gl::FUNC_ADD,
            "sub" => gl::FUNC_SUBTRACT,
            "reverse" => gl::FUNC_REVERSE_SUBTRACT,
            "one" => gl::ONE,
            "zero" => gl::ZERO,
            &_ => gl::ZERO,
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

#[derive(Clone, Copy)]
pub enum Blend {
    Zero,
    One,
    SrcColor,
    DstColor,
    OneMinusSrcColor,
    OneMinusDstColor,
    OneMinusSrcAlpha,
    OneMinusDstAlpha,

    SrcAlpha,
    DstAlpha,

    ConstColor,
    ConstAlpha,
}
impl Blend {
    pub const fn gl_enum(&self) -> GLenum {
        match self {
            Blend::Zero => gl::ZERO,
            Blend::One => gl::ONE,
            Blend::SrcColor => gl::SRC_COLOR,
            Blend::DstColor => gl::DST_COLOR,
            Blend::OneMinusSrcColor => gl::ONE_MINUS_SRC_COLOR,
            Blend::OneMinusDstColor => gl::ONE_MINUS_DST_COLOR,
            Blend::SrcAlpha => gl::SRC_ALPHA,
            Blend::DstAlpha => gl::DST_ALPHA,
            Blend::ConstColor => gl::CONSTANT_COLOR,
            Blend::ConstAlpha => gl::CONSTANT_ALPHA,
            Blend::OneMinusSrcAlpha => gl::ONE_MINUS_SRC_ALPHA,
            Blend::OneMinusDstAlpha => gl::ONE_MINUS_DST_ALPHA,
        }
    }
}

fn blend(src: Blend, dst: Blend) {
    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(src.gl_enum(), dst.gl_enum());
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
    pub const fn blend(&self) -> (Blend, Blend) {
        match self {
            ConstBlend::Normal => (Blend::SrcAlpha, Blend::OneMinusSrcAlpha),
            ConstBlend::Additive => (Blend::SrcAlpha, Blend::One),
            ConstBlend::Multiply => (Blend::DstColor, Blend::Zero),
            ConstBlend::Screen => (Blend::One, Blend::OneMinusSrcColor),
            ConstBlend::Overlay => (Blend::One, Blend::OneMinusSrcAlpha),
            ConstBlend::Premultiplied => (Blend::One, Blend::OneMinusSrcAlpha),
            ConstBlend::Custom(src, dst) => (*src, *dst),
            ConstBlend::SrcOnly => (Blend::One, Blend::Zero),
        }
    }
}

pub fn const_blend(b: ConstBlend) {
    let (src, dst) = b.blend();
    blend(src, dst);
}
