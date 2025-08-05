use std::{
    collections::{ HashSet},
    ops::Deref,
    sync::LazyLock,
};
use glam::{Mat4, Vec2};

use crate::{
    draws::window_ort,
    gl_unit::{
        define::{DrawMode, VertexArrayAttribPointerGen},
        program::{Program, PROGRAM2D_TWO},
        texture::{Texture, Texture2D, TextureMap },
        window::Window,
        FrameBuffer,
    },
    TEX_VERTEX_YFLIP_STATIC, VAO_MUT, VAO_STATIC, VERTEX_MUT,
};

pub mod font;

const UI_PROGRAM_VERT: &str = include_str!("../shaders/ui/vert.glsl");
const UI_PROGRAM_FARG: &str = include_str!("../shaders/ui/frag.glsl");

static UI_PROGRAM: LazyLock<Program> =
    LazyLock::new(|| Program::basic_new(UI_PROGRAM_VERT, UI_PROGRAM_FARG, None));
//left down
pub fn color(
    window_size: (i32, i32),
    color: (u8, u8, u8, u8),
    pos: (f32, f32),
    size: (f32, f32),
    radius: usize,
) {
    // //radius
    // todo!();
    UI_PROGRAM.bind();
    UI_PROGRAM.put_matrix_name(window_ort(window_size), "project");

    let (r, g, b, a) = (
        color.0 as f32 / 255.0,
        color.1 as f32 / 255.0,
        color.2 as f32 / 255.0,
        color.3 as f32 / 255.0,
    );
    UI_PROGRAM.put_vec4([r, g, b, a], UI_PROGRAM.get_uniform("draw_color"));

    let (x, y) = pos;
    let (w, h) = size;
    VERTEX_MUT.sub_data(&[x, y, x + w, y, x + w, y - h, x, y - h], 0);
    VAO_STATIC.bind(|vao| {
        vao.draw_arrays(DrawMode::Quads, 0, 4);
    });
}

//left down
pub fn texture_y_flip(window_size: (i32, i32), texture: &Texture2D, pos: Vec2, size: Vec2) {
    let program = &PROGRAM2D_TWO;
    program.bind();
    program.put_matrix_name(Mat4::IDENTITY, "model_mat");

    program.put_matrix_name(window_ort(window_size), "project_mat");
    texture.bind_unit(0);
    program.put_texture(0, program.get_uniform("image"));

    let (x, y, w, h) = (pos.x, pos.y, size.x, size.y);
    VERTEX_MUT.sub_data(&[x, y, x + w, y, x + w, y - h, x, y - h], 0);
    VAO_MUT.bind(|vao| {
        vao.pointer(
            TEX_VERTEX_YFLIP_STATIC.deref(),
            VertexArrayAttribPointerGen::new::<f32>(1, 2),
        );
        vao.pointer(
            VERTEX_MUT.deref(),
            VertexArrayAttribPointerGen::new::<f32>(0, 2),
        );
        vao.draw_element(DrawMode::Quads, 0, 4);
    });
}

pub fn texture_map(texture: &TextureMap<String>, name: &str, pos: Vec2) {
    todo!()
}

#[cfg(test)]
mod test {
    use crate::gl_unit::{window::Window, GLcontext};

    use super::color;

    #[test]
    fn ui_color() {
        let mut window = Window::new(800, 600, "reanim test", false);
        let mut context = GLcontext::with(&mut window);
        window.window.show();
        while !window.update() {
            context.draw_option(&mut window, |_, window| {
                color(
                    window.window.get_size(),
                    (255, 0, 255, 255),
                    (0f32, 0f32),
                    (100f32, 100f32),
                    0,
                );
            });
        }
    }
}
pub struct KeyStream {
    used_key: HashSet<glfw::Key>,
    used_mouse_button: HashSet<glfw::MouseButton>,
    cursor_close: bool,
}
impl KeyStream {
    pub fn new() -> Self {
        Self {
            used_key: HashSet::new(),
            used_mouse_button: HashSet::new(),
            cursor_close: false,
        }
    }
    pub fn use_key(&mut self, key: glfw::Key) -> bool {
        if self.used_key.contains(&key) {
            false
        } else {
            self.used_key.insert(key);
            true
        }
    }
    pub fn use_mouse_button(&mut self, key: glfw::MouseButton) -> bool {
        if self.used_mouse_button.contains(&key) {
            false
        } else {
            self.used_mouse_button.insert(key);
            true
        }
    }
    pub fn cursor_close(&mut self) -> bool {

        if self.cursor_close {
            false
        } else {
            self.cursor_close = true;
            true
        }
    }
    pub fn rewind(&mut self) {
        self.used_key.clear();
        self.used_mouse_button.clear();
        self.cursor_close = false;
    }
}

pub struct Frame {
    object: Vec<Box<dyn UIrender>>,
}
impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

impl Frame {
    pub const fn new() -> Self {
        Frame { object: Vec::new() }
    }
    pub fn add<T: UIrender + 'static>(&mut self, obj: T) {
        self.object.push(Box::new(obj) as Box<dyn UIrender>);
    }
    pub fn draw(&mut self, window: &mut Window, key_stream: &mut KeyStream) {
        for obj in self.object.iter_mut().rev(){
            obj.update(window, key_stream);
        }

        for obj in self.object.iter() {
            if let Some(fb) = obj.draw() {
                fb.view_port();
                fb.texture.as_ref().unwrap().bind_unit(0);
                PROGRAM2D_TWO.put_texture(0, PROGRAM2D_TWO.get_uniform("image"));

                VERTEX_MUT.sub_data(&[-1f32, 1f32, 1f32, 1f32, 1f32, -1f32, -1f32, -1f32], 0);
                PROGRAM2D_TWO.bind();
                PROGRAM2D_TWO.put_matrix_name(Mat4::IDENTITY, "project_mat");
                PROGRAM2D_TWO.put_matrix_name(Mat4::IDENTITY, "model_mat");
                VAO_STATIC.bind(|vao| {
                    vao.draw_arrays(DrawMode::Quads, 0, 4);
                });
            }
            window.view_port();
            obj.fast_draw(window);
        }
        
    }
}

pub trait UIlayout {
    fn size(&self) -> (f32, f32);
    fn set_pos(&mut self, pos: (f32, f32));
}
pub trait UIrender {
    fn draw(&self) -> Option<&FrameBuffer>;

    fn fast_draw(&self, window: &mut Window);
    fn update(&mut self,window: &mut Window,key_stream:&mut KeyStream);
}

trait UIObject: UIrender + UIlayout {}
impl<T: UIrender + UIlayout> UIObject for T {}
pub mod layout;
pub mod object;
