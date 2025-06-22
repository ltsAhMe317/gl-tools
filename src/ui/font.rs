use core::panic;
use std::fs;
use std::sync::{LazyLock, Mutex};

use freetype::face::LoadFlag;
use freetype::{self as ft, Bitmap, GlyphSlot};
use glam::{vec3, Mat4};
use std::collections::HashMap;

use crate::draws::window_ort;
use crate::gl_unit::define::{TextureParm, TextureType, VertexArrayAttribPointerGen};
use crate::gl_unit::program::Program;
use crate::gl_unit::texture::Texture2D;
use crate::gl_unit::texture::{Texture, TextureMap, TextureWrapper};
use crate::gl_unit::{self, view_port};
use crate::{VAO_MUT, VERTEX_BIG_MUT};
use std::path::Path;

static FT_LIB: LazyLock<ft::Library> = LazyLock::new(|| ft::Library::init().unwrap());

pub const FONT_SIZE_AUTO: u32 = 0;

const FT_TEXTURE_H: u32 = 32;

#[derive(Clone, Copy)]
struct Character {
    bearing: (i32, i32),
    advance: i64,
}
impl Character {
    pub fn new(char: usize, font: &Font) -> (Self, TextureWrapper<Texture2D>) {
        font.load_char(char);
        let texture = {
            let img = font.bitmap();
            let buffer = img.buffer();

            Texture2D::load(
                Some(buffer),
                TextureType::RED8,
                img.width() as u32,
                img.rows() as u32,
                TextureParm::new().once_load_size(1),
            )
        };
        (
            Self {
                bearing: {
                    let glyph = font.glyph();
                    (glyph.bitmap_left(), glyph.bitmap_top())
                },
                advance: font.advance().x,
            },
            TextureWrapper(texture),
        )
    }
}

const FT_PROGRAM_VERT: &str = "
    #version 330
    layout (location = 0) in vec4 vert;
    out vec2 uv;
    uniform mat4 project_mat;
    uniform mat4 model_mat;
    void main(){
        gl_Position = project_mat*model_mat * vec4(vert.xy,0,1);
        uv = vert.zw;
    }
";
const FT_PROGRAM_FRAG: &str = "
    #version 330
    in vec2 uv;
    uniform sampler2D text;
    uniform vec4 text_color;
    out vec4 color;
    void main(){
        vec4 sampled = vec4(1.0, 1.0, 1.0, texture(text,uv).r);

        color = text_color * sampled;
                // color = vec4(1,1,1,1);
    }
";

static FT_PROGRAM: LazyLock<Program> =
    LazyLock::new(|| Program::basic_new(FT_PROGRAM_VERT, FT_PROGRAM_FRAG, None));

pub struct FontBound {
    pub x_start: f32,
    pub x_end: f32,
    pub x_end_advance: f32,
    pub y_bottom: f32,
    pub y_top: f32,
}

unsafe impl Sync for Font {}
unsafe impl Send for Font {}
pub struct Font {
    font_date: freetype::Face,
    char_tex: TextureMap<usize>,
    characters: HashMap<usize, Character>,
}

impl Font {
    pub fn new_raw(raw: Vec<u8>, index: isize) -> Self {
        let font = Self {
            font_date: FT_LIB
                .new_memory_face(raw, index)
                .expect("new font from raw error!"),
            char_tex: TextureMap::new(1024, 1024),
            characters: HashMap::new(),
        };
        font.set_size(FONT_SIZE_AUTO, FT_TEXTURE_H);
        font
    }
    pub fn new_file(path: &Path, index: isize) -> Self {
        // let font = Self {
        //     font_date: FT_LIB
        //         .new_face(path, index)
        //         .expect("new font from file error!"),
        //     char_tex: TextureMap::new(4000, 4000),
        //     characters: HashMap::new(),
        // };
        // font.set_size(FONT_SIZE_AUTO, FT_TEXTURE_H);
        // font
        Self::new_raw(fs::read(path).unwrap(), index)
    }

    fn set_size(&self, w: u32, h: u32) {
        self.font_date
            .set_pixel_sizes(w, h)
            .expect("font set size error!");
    }

    fn load_char(&self, char: usize) {
        match self.font_date.load_char(char, LoadFlag::RENDER) {
            Ok(_) => (),
            Err(error) => {
                panic!("Font load char {} err! {}", char, error);
            }
        }
    }

    fn glyph(&self) -> &GlyphSlot {
        self.font_date.glyph()
    }
    fn bitmap(&self) -> Bitmap {
        self.glyph().bitmap()
    }
    fn advance(&self) -> freetype::ffi::FT_Vector {
        self.glyph().advance()
    }

    fn get_char(&mut self, str: &str) -> Vec<Character> {
        let mut ready_map_list = Vec::new();

        let mut vec = Vec::with_capacity(str.len());
        for char in str.chars() {
            let char = char as usize;
            match self.characters.get(&char) {
                Some(char) => {
                    vec.push(*char);
                }
                None => {
                    let (charater, tex) = Character::new(char, self);
                    self.characters.insert(char, charater);
                    ready_map_list.push((char, tex));

                    vec.push(charater);
                }
            }
        }
        //draw in tex_map

        if self.char_tex.add(ready_map_list, true).is_err() {
            self.characters.clear();
            self.char_tex.clear();
            return self.get_char(str);
        }

        vec
    }

    pub fn size(&mut self, str: &str, scale: i32) -> usize {
        let scale = scale as f32 / FT_TEXTURE_H as f32 * 2f32;

        let mut x = 0f32;

        let chars = self.get_char(str.as_ref());
        for char in chars.iter() {
            x += (char.advance >> 6) as f32 * scale;
        }
        x as usize
    }

    pub fn draw_as_texture(&mut self) -> Texture2D {
        todo!()
    }

    pub fn draw(
        &mut self,
        str: &str,
        window_size: (i32, i32),
        x: f32,
        y: f32,
        scale: i32,
        color: (f32, f32, f32, f32),
    ) {
        let char_len = str.chars().count();
        let scale = scale as f32 / FT_TEXTURE_H as f32 * 2f32;

        let mut x_count = 0f32;

        let mut vertex: Vec<f32> = Vec::with_capacity(char_len * 4 * 4);

        let chars = self.get_char(str);
        for (index, char) in str.chars().zip(chars.iter()) {
            let uv = self.char_tex.get_uv(&(index as usize)).unwrap();

            let tex_size = uv.get_pixel_size();

            let gl_x = char.bearing.0 as f32 * scale + x_count;

            let gl_y = -((tex_size.1 as i32 - char.bearing.1) as f32) * scale;

            let w = tex_size.0 * scale;
            let h = tex_size.1 * scale;

            let uv = uv.get_uv();
            vertex.extend_from_slice(&[
                gl_x,
                gl_y + h,
                uv[0],
                uv[1],
                gl_x + w,
                gl_y + h,
                uv[2],
                uv[3],
                gl_x + w,
                gl_y,
                uv[4],
                uv[5],
                gl_x,
                gl_y,
                uv[6],
                uv[7],
            ]);

            x_count += (char.advance >> 6) as f32 * scale;
        }
        let (window_w, window_h) = window_size;
        view_port(0, 0, window_w, window_h);
        gl_unit::const_blend(gl_unit::ConstBlend::Normal);
        FT_PROGRAM.put_texture(0, FT_PROGRAM.get_uniform("text"));
        self.char_tex.get_tex().bind_unit(0);
        FT_PROGRAM.bind();
        FT_PROGRAM.put_matrix_name(&(window_ort(window_size)), "project_mat");
        FT_PROGRAM.put_matrix_name(&Mat4::from_translation(vec3(x, y, 0f32)), "model_mat");
        FT_PROGRAM.put_vec4(
            [color.0, color.1, color.2, color.3],
            FT_PROGRAM.get_uniform("text_color"),
        );

        VAO_MUT.bind_set(
            &VERTEX_BIG_MUT,
            VertexArrayAttribPointerGen::new::<f32>(0, 4),
        );

        VERTEX_BIG_MUT.sub_data(&vertex, 0);
        FT_PROGRAM.draw_rect(str.chars().count() as i32);
    }
}
#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::gl_unit::{window::Window, GLcontext};

    use super::Font;

    #[test]
    fn font() {
        let mut window = Window::new(800, 600, "test font", false);
        let mut context = GLcontext::with(&mut window);
        window.window.show();

        let mut font = Font::new_file(Path::new("./font.otf"), 0);

        while !window.update() {
            context.draw_option(&mut window, |_, window| {
                font.draw(
                    "hello? this is a test:) 牛逼",
                    window.window.get_size(),
                    -400f32,
                    0f32,
                    10,
                    (1f32, 0f32, 1f32, 1f32),
                );

                // let mut y = -300;
                // while y < 300 {
                //     font.draw(
                //         "hello? this is a test:)",
                //         size.0,
                //         size.1,
                //         -400f32,
                //         y as f32,
                //         10,
                //         (1f32, 0f32, 1f32, 1f32),
                //     );
                //     y += 5;
                // }
                font.draw(
                    &format!("fps:{}", window.delta_count.fps()),
                    window.window.get_size(),
                    0f32,
                    0f32,
                    25,
                    (1f32, 1f32, 1f32, 1f32),
                );

                // font.draw("cao cao zhongyu",size.0,size.1,-400f32, -300f32, 300, (0f32,1f32,1f32,1f32));
            });
        }
    }
}
