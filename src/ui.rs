use std::{ops::Deref, sync::LazyLock};

use glam::{Mat4, Vec2};

use crate::{
    draws::window_ort,
    gl_unit::{
        define::{DrawMode, TextureParm, TextureType, VertexArrayAttribPointerGen},
        program::{Program, PROGRAM2D_TWO},
        texture::{Texture, Texture2D, TextureMap, TextureWrapper},
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
    UI_PROGRAM.put_matrix_name(&window_ort(window_size), "project");

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
    program.put_matrix_name(&Mat4::IDENTITY, "model_mat");

    program.put_matrix_name(&window_ort(window_size), "project_mat");
    texture.bind_unit(0);
    program.put_texture(0, program.get_uniform("image"));

    let (x, y, w, h) = (pos.x, pos.y, size.x, size.y);
    VERTEX_MUT.sub_data(&[x, y, x + w, y, x + w, y - h, x, y - h], 0);
VAO_MUT.bind(|vao|{
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

pub struct Frame {
    object: Vec<Box<dyn UIobject>>,
    fb: Vec<Option<FrameBuffer>>,
}
impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

impl Frame {
    pub const fn new() -> Self {
        Frame {
            object: Vec::new(),
            fb: Vec::new(),
        }
    }
    pub fn add<T: UIobject + 'static>(&mut self, obj: T) {
        let alloc = T::alloc();
        match alloc {
            Some((w, h)) => {
                let texture = TextureWrapper(Texture2D::with_size(
                    w as u32,
                    h as u32,
                    TextureType::RGBA8,
                    TextureParm::new(),
                ));
                let mut frame = FrameBuffer::new();
                frame.link_texture(texture, gl::COLOR_ATTACHMENT0);
                self.fb.push(Some(frame));
            }
            None => self.fb.push(None),
        }
        self.object.push(Box::new(obj) as Box<dyn UIobject>);
    }
    pub fn draw(&mut self, window: &mut Window) {
        for (obj, fb) in self.object.iter_mut().zip(self.fb.iter()) {
            window.view_port();
            if let Some(fb) = fb {
                obj.draw(window, fb);
            }
            if let Some(fb) = fb {
                fb.view_port();
                fb.texture.as_ref().unwrap().bind_unit(0);
                PROGRAM2D_TWO.put_texture(0, PROGRAM2D_TWO.get_uniform("image"));

                
                VERTEX_MUT.sub_data(&[-1f32, 1f32, 1f32, 1f32, 1f32, -1f32, -1f32, -1f32], 0);
                PROGRAM2D_TWO.bind();
                PROGRAM2D_TWO.put_matrix_name(&Mat4::IDENTITY, "project_mat");
                PROGRAM2D_TWO.put_matrix_name(&Mat4::IDENTITY, "model_mat");
                    VAO_STATIC.bind(|vao|{
                vao.draw_arrays(DrawMode::Quads, 0, 4);
                });
            } else {
                obj.draw_fast(window);
            }
        }
    }
}

pub trait UIobject {
    fn draw(&mut self, window: &mut Window, frame: &FrameBuffer);
    fn alloc() -> Option<(usize, usize)>
    where
        Self: Sized;
    fn draw_fast(&mut self, window: &mut Window) {}
}

pub struct UIbutton{
    pub text_color:(f32,f32,f32,f32),
    pub pos:(f32,f32),
    pub size:(f32,f32),
    pub text:String,
    pub action:&'static dyn FnMut(&UIbutton)
}
impl UIobject for UIbutton{
    fn draw_fast(&mut self, window: &mut Window) {
        font::font(|font|{
            font.draw(&self.text,window.window.get_size(), self.pos.0, self.pos.1,25i32, self.text_color);
        });
    }
    fn draw(&mut self, window: &mut Window, frame: &FrameBuffer) {}
    fn alloc() -> Option<(usize, usize)>
    where
        Self: Sized {
        None
    }
}
