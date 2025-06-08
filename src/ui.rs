use std::sync::LazyLock;

use glam::{vec3, Mat3, Mat4, Vec2};

use crate::{
    gl_unit::{
        texture::{Texture, Texture2D, TextureMap, TextureParm, TextureType, TextureWrapper},
        window::Window,
        FrameBuffer, Program, PROGRAM2D_TWO,
    },
    TEX_VERTEX_YFLIP_STATIC, VAO_MUT, VAO_STATIC, VERTEX_MUT,
};

pub mod font;

const UI_PROGRAM_VERT: &str = "
    #version 460
    layout (location = 0) in vec2 vert;
    uniform mat4 project;
    void main(){
        gl_Position = project*vec4(vert,0,1);
    }    
";
const UI_PROGRAM_FARG: &str = "
    #version 460
    uniform vec4 draw_color;
    out vec4 color;
    void main(){
         color = draw_color;
    }
";

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
    UI_PROGRAM.bind();

    let (w, h) = window_size;
    let (w, h) = (w as f32 / 2f32, h as f32 / 2f32);
    UI_PROGRAM.put_matrix_name(
        &Mat4::orthographic_rh_gl(-w, w, -h, h, -1f32, 1f32),
        "project",
    );

    let (r, g, b, a) = (
        color.0 as f32 / 255.0,
        color.1 as f32 / 255.0,
        color.2 as f32 / 255.0,
        color.3 as f32 / 255.0,
    );
    UI_PROGRAM.put_vec4([r, g, b, a], UI_PROGRAM.get_uniform("draw_color"));

    let (x, y) = pos;
    let (w, h) = size;
    VAO_STATIC.bind();
    VERTEX_MUT.sub(&[x, y, x + w, y, x + w, y - h, x, y - h], 0);

    UI_PROGRAM.draw_rect(1);
}

//left down
pub fn texture_y_flip(window_size: (i32, i32), texture: &Texture2D, pos: Vec2, size: Vec2) {
    let program = &PROGRAM2D_TWO;
    program.bind();
    program.put_matrix_name(&Mat4::IDENTITY, "model_mat");

    let (w, h) = window_size;
    let (w, h) = (w as f32 / 2f32, h as f32 / 2f32);
    program.put_matrix_name(
        &Mat4::orthographic_rh_gl(-w, w, -h, h, -1f32, 1f32),
        "project_mat",
    );
    texture.bind_unit(0);
    program.put_texture(0, program.get_uniform("image"));

    let (x, y, w, h) = (pos.x, pos.y, size.x, size.y);
    VERTEX_MUT.sub(&[x, y, x + w, y, x + w, y - h, x, y - h], 0);

    VAO_MUT.with(&TEX_VERTEX_YFLIP_STATIC, 1, 2, gl::FLOAT, 0);
    VAO_MUT.with(&VERTEX_MUT, 0, 2, gl::FLOAT, 0);

    program.draw_rect(1);
}

pub fn texture_map(texture: &TextureMap<String>, name: &str, pos: Vec2) {
    todo!()
}

#[cfg(test)]
mod test {
    use crate::{
        draws::Reanim,
        gl_unit::{texture::TextureMap, window::Window, GLcontext},
    };

    use super::{color, font::with_font};

    #[test]
    fn ui_color() {
        let mut window = Window::new(800, 600, "reanim test", false);
        let mut context = GLcontext::with(&mut window);
        window.window.show();
        while !window.update() {
            context.draw_option(&mut window, |_, window| {
                with_font(|font| {
                    font.draw(
                        "为什么不显示啊！",
                        window.window.get_size(),
                        0f32,
                        0f32,
                        16,
                        (1f32, 1f32, 1f32, 1f32),
                    );
                });

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

                unsafe {
                    VAO_STATIC.bind();
                    VERTEX_MUT.sub(&[-1f32, 1f32, 1f32, 1f32, 1f32, -1f32, -1f32, -1f32], 0);
                    PROGRAM2D_TWO.bind();
                    PROGRAM2D_TWO.put_matrix_name(&Mat4::IDENTITY, "project_mat");
                    PROGRAM2D_TWO.put_matrix_name(&Mat4::IDENTITY, "model_mat");
                    PROGRAM2D_TWO.draw_rect(1);
                }
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
