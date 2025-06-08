use core::panic;
use std::{
    collections::HashMap,
    ffi::c_void,
    fs::{self},
    hash::Hash,
    ops::Deref,
    path::Path,
    ptr::null,
};

use gl::types::{GLenum, GLint, GLsizei, GLuint};
use glam::{vec2, Mat4, Vec2};
use guillotiere::*;
use image::{DynamicImage, EncodableLayout, ImageBuffer, Rgba};

use std::fmt::{Debug, Formatter};

use crate::{TEX_VERTEX_STATIC, TEX_VERTEX_YFLIP_STATIC, VAO_MUT, VERTEX_MUT};

use super::{FrameBuffer, PROGRAM2D_TWO};

const TEXTURE_MAP_MAX: i32 = 4000;

#[derive(Clone, Copy, Debug)]
pub struct UVindex {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}
impl UVindex {
    pub const fn get_uv(&self) -> [f32; 8] {
        [
            self.x,
            self.y + self.h,
            self.x + self.w,
            self.y + self.h,
            self.x + self.w,
            self.y,
            self.x,
            self.y,
        ]
    }

    pub const fn get_pixel_size(&self) -> (f32, f32) {
        (
            self.w * TEXTURE_MAP_MAX as f32,
            self.h * TEXTURE_MAP_MAX as f32,
        )
    }
}

pub struct TextureMap<T: Hash + Eq> {
    allocator: AtlasAllocator,
    frame: FrameBuffer,
    index: HashMap<T, UVindex>,
}
impl TextureMap<String> {
    pub fn new_dir(path: &Path) -> TextureMap<String> {
        let read = fs::read_dir(path).expect("failed read dir");
        let mut texs = Vec::new();
        for file in read {
            match file {
                Ok(file) => {
                    if file.file_type().unwrap().is_file() {
                        let file_name_bind = file.file_name();
                        let file_name = file_name_bind.to_str().unwrap();
                        let file_name = file_name[..file_name.rfind('.').unwrap()].to_string();
                        let texture = TextureWrapper(Texture2D::load_path(
                            file.path().as_path(),
                            TextureParm::new(),
                        ));
                        texs.push((file_name, texture));
                    }
                }
                Err(err) => println!("failed read dir"),
            }
        }
        let mut map: TextureMap<String> = TextureMap::<String>::new();
        map.add(texs, true);
        map
    }
}
impl<T: Hash + Eq> Default for TextureMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Hash + Eq> TextureMap<T> {
    pub fn new() -> Self {
        let rect_map = AtlasAllocator::new(Size::new(TEXTURE_MAP_MAX, TEXTURE_MAP_MAX));
        let uv_list: HashMap<T, UVindex> = HashMap::new();
        let texture = TextureWrapper(Texture2D::with_size(
            TEXTURE_MAP_MAX as u32,
            TEXTURE_MAP_MAX as u32,
            TextureType::RGBA8,
            TextureParm::new(),
        ));
        let mut frame = FrameBuffer::new();
        frame.link_texture(texture, gl::COLOR_ATTACHMENT0);
        FrameBuffer::unbind();
        Self {
            index: uv_list,
            frame,
            allocator: rect_map,
        }
    }
    pub fn add<Tex: AsRef<Texture2D>>(
        &mut self,
        vec: Vec<(T, Tex)>,
        y_flip: bool,
    ) -> Result<(), &'static str> {
        if vec.is_empty() {
            return Err("textures is empty");
        }

        self.frame.bind(gl::FRAMEBUFFER);
        self.frame.view_port();
        let program = PROGRAM2D_TWO.deref();
        program.bind();
        program.put_matrix_name(
            &Mat4::orthographic_rh_gl(0f32, 1f32, 0f32, 1f32, 1f32, -1f32),
            "project_mat",
        );

        program.put_matrix_name(&Mat4::IDENTITY, "model_mat");
        program.put_texture(0, program.get_uniform("image"));

        unsafe {
            // VAO_MUT.as_ref().unwrap().bind();
            if y_flip {
                VAO_MUT.with(&TEX_VERTEX_YFLIP_STATIC, 1, 2, gl::FLOAT, 0);
            } else {
                VAO_MUT.with(&TEX_VERTEX_STATIC, 1, 2, gl::FLOAT, 0);
            }
            VAO_MUT.with(&VERTEX_MUT, 0, 2, gl::FLOAT, 0);
        }

        let mut uv_list = HashMap::new();
        for (name, texture) in vec.into_iter() {
            let texture = texture.as_ref();

            let rect = match self
                .allocator
                .allocate(Size::new(texture.w as i32, texture.h as i32))
            {
                Some(rect) => rect.rectangle,
                None => {
                    return Err("can not allocate");
                }
            };
            let uv = UVindex {
                x: rect.min.x as f32 / TEXTURE_MAP_MAX as f32,
                y: rect.min.y as f32 / TEXTURE_MAP_MAX as f32,
                w: rect.width() as f32 / TEXTURE_MAP_MAX as f32,
                h: rect.height() as f32 / TEXTURE_MAP_MAX as f32,
            };

            texture.bind_unit(0);

            VERTEX_MUT.sub(
                &[
                    uv.x,
                    uv.y + uv.h,
                    uv.x + uv.w,
                    uv.y + uv.h,
                    uv.x + uv.w,
                    uv.y,
                    uv.x,
                    uv.y,
                ],
                0,
            );

            program.draw_rect(1);
            uv_list.insert(name, uv);
        }
        FrameBuffer::unbind();
        self.index.extend(uv_list);
        Ok(())
    }

    pub fn clear(&mut self) {
        self.allocator.clear();
        self.index.clear();
    }

    pub fn get_uv(&self, name: &T) -> Option<UVindex> {
        Some(*self.index.get(name)?)
    }

    pub fn get_tex(&self) -> &Texture2D {
        self.frame.texture.as_ref().unwrap()
    }
}
#[cfg(test)]
mod test {
    use std::{
        fs::{self},
        path::Path,
    };

    use glam::Mat4;

    use crate::{
        gl_unit::{
            texture::{Texture, TextureWrapper},
            window::Window,
            GLcontext, PROGRAM2D_ONE,
        },
        TEX_VERTEX_MUT, VAO_MUT, VERTEX_MUT,
    };

    use super::{Texture2D, TextureMap, TextureParm};

    #[test]
    fn texture_map() {
        let mut window = Window::new(800, 600, "test", false);
        let mut context = GLcontext::with(&mut window);

        let tex_list = {
            let mut map = Vec::new();
            // let read_dir = fs::read_dir(Path::new(
            //     "E:\\b\\pixiv\\jcm2\\#ロリ Rapidin 2021 vol.4 - jcm2的插画 - pixiv",
            // ))
            // .unwrap();
            let read_dir = fs::read_dir(Path::new(
                "E:\\RustroverProjects\\bone_static\\Res\\Texture",
            ))
            .unwrap();

            for file in read_dir {
                let file = match file {
                    Ok(file) => file,
                    Err(_) => continue, // 跳过错误项
                };

                // 确保不是目录
                if file.file_type().map(|ft| ft.is_dir()).unwrap_or(true) {
                    continue; // 如果是目录或无法获取文件类型，跳过
                }

                let file_name = file.file_name();
                let name = file_name.to_str().unwrap();
                let set_name = &name[..name.find(".").unwrap()];
                println!("load:{:?}", file.path());
                map.push((
                    set_name.to_string(),
                    TextureWrapper(Texture2D::load_path(
                        file.path().as_path(),
                        TextureParm::new(),
                    )),
                ));
            }
            map
        };
        unsafe {
            gl::Enable(gl::TEXTURE_2D);
        }
        let mut what = Vec::new();
        // let tex = Texture2D::load_path(
        //     Path::new("E:\\b\\pixiv\\啦啦啦的收藏 - pixiv\\26533633_p0.png"),
        //     TextureParm::new(),
        // );
        for (str, tex) in tex_list.iter() {
            what.push((str.to_string(), tex.as_ref()));
        }
        // let he = "hehe".to_string();
        // let what = vec![(&he, &tex)];

        println!("load map...");
        let mut map = TextureMap::new();
        map.add(tex_list, true);
        // map.frame
        //     .texture
        //     .as_ref()
        //     .unwrap()
        //     .get_image()
        //     .save(Path::new("test map.png"))
        //     .unwrap();
        window.view_port();
        window.window.show();

        unsafe {
            VAO_MUT.with(&VERTEX_MUT, 0, 2, gl::FLOAT, 0);
            VAO_MUT.with(&TEX_VERTEX_MUT, 1, 2, gl::FLOAT, 0);
        }
        while !window.update() {
            context.draw_option(&mut window, |context, window| {
                let program = &PROGRAM2D_ONE;
                program.bind();
                program.put_matrix_name(&Mat4::IDENTITY, "project_mat");
                program.put_matrix_name(&Mat4::IDENTITY, "model_mat");
                program.put_texture(0, program.get_uniform("image"));
                map.get_tex().bind_unit(0);

                unsafe {
                    VERTEX_MUT.sub(&[-1f32, 1f32, 1f32, 1f32, 1f32, -1f32, -1f32, -1f32], 0);

                    let index = map.get_uv(&"GINO".to_string()).unwrap();
                    TEX_VERTEX_MUT.sub(
                        &[
                            index.x,
                            index.y + index.h,
                            index.x + index.w,
                            index.y + index.h,
                            index.x + index.w,
                            index.y,
                            index.x,
                            index.y,
                        ],
                        0,
                    );
                }

                program.draw_rect(1);
            });
        }
    }
}

pub struct TextureWrapper<T: Texture>(pub T);
impl<T: Texture> TextureWrapper<T> {
    pub const fn new(texture: T) -> Self {
        Self(texture)
    }
}
impl<T: Texture> Drop for TextureWrapper<T> {
    fn drop(&mut self) {
        self.0.delete();
    }
}
impl<T: Texture> AsRef<T> for TextureWrapper<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
impl<T: Texture> Deref for TextureWrapper<T> {
    fn deref(&self) -> &Self::Target {
        &self.0
    }

    type Target = T;
}

pub trait Texture {
    fn send_to_texture(&self);
    fn bind_unit(&self, id: i32);
    fn send_date<T>(&self, date_mode: GLenum, date_type: GLenum, date: Vec<T>);
    fn delete(&self);
}

#[derive(Clone)]
pub struct Texture1D {
    pub texture: GLuint,
    pub size: u32,
}
impl Texture for Texture1D {
    fn send_to_texture(&self) {
        unsafe { gl::BindTexture(gl::TEXTURE_1D, self.texture) }
    }
    fn bind_unit(&self, id: i32) {
        bind_texture_unit(id);
        self.send_to_texture();
    }

    fn send_date<T>(&self, date_mode: GLenum, date_type: GLenum, date: Vec<T>) {
        self.send_to_texture();
        unsafe {
            gl::TexSubImage1D(
                gl::TEXTURE_1D,
                0,
                0,
                date.len() as GLsizei,
                date_mode,
                date_type,
                date.as_ptr() as *const c_void,
            );
        }
    }
    fn delete(&self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture as *const GLuint);
        }
    }
}
impl Texture1D {
    pub fn new_size(size: u32) -> Self {
        Self::new::<u8>(null(), TextureType::RED8, size, BASE_PARM)
    }

    pub fn load<T>(raw: Option<&[T]>, mode: TextureType, size: u32, parm: TextureParm) -> Self {
        if let Some(raw) = raw {
            Self::new(raw.as_ptr(), mode, size, parm);
        }
        Self::new(null::<T>(), mode, size, parm)
    }

    fn new<T>(raw: *const T, mode: TextureType, size: u32, parm: TextureParm) -> Self {
        let mut id: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut id as *mut u32);
            gl::BindTexture(gl::TEXTURE_1D, id);
            texture_parm(gl::TEXTURE_1D, parm);

            //(internal fmt,type)
            let type_: (GLenum, GLenum) = mode.as_gl();
            gl::TexImage1D(
                gl::TEXTURE_1D,
                0,
                type_.0 as i32,
                size as i32,
                0,
                type_.0,
                type_.1,
                raw as *const c_void,
            );
        }
        Self { texture: id, size }
    }
}

impl Debug for Texture2D {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "w:{},h:{},id:{}", self.w, self.h, self.texture)
    }
}
#[derive(Clone)]
pub struct Texture2D {
    pub texture: GLuint,
    pub w: u32,
    pub h: u32,
}

impl Texture for Texture2D {
    fn send_to_texture(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
        }
    }

    fn bind_unit(&self, id: i32) {
        bind_texture_unit(id);
        self.send_to_texture();
    }

    fn send_date<T>(&self, date_mode: GLenum, date_type: GLenum, date: Vec<T>) {
        todo!()
    }

    fn delete(&self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture as *const GLuint);
        }
    }
}
impl Texture2D {
    pub fn get_image(&self) -> DynamicImage {
        self.send_to_texture();
        unsafe {
            let mut image_date: Vec<u8> = Vec::with_capacity((self.w * self.h * 4) as usize);
            image_date.set_len((self.w * self.h * 4) as usize);
            gl::GetTexImage(
                gl::TEXTURE_2D,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                image_date.as_mut_ptr() as *mut c_void,
            );
            let image_buffer: ImageBuffer<Rgba<u8>, _> =
                ImageBuffer::from_raw(self.w, self.h, image_date).unwrap();
            DynamicImage::ImageRgba8(image_buffer)
        }
    }

    pub unsafe fn new<T>(
        raw: *const T,
        mode: TextureType,
        w: u32,
        h: u32,
        parm: TextureParm,
    ) -> Self {
        let mut id: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut id as *mut u32);
            gl::BindTexture(gl::TEXTURE_2D, id);

            texture_parm(gl::TEXTURE_2D, parm);
            //(internal fmt,type)
            let type_: (GLenum, GLenum) = mode.as_gl();
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                type_.0 as GLint,
                w as GLsizei,
                h as GLsizei,
                0,
                type_.0,
                type_.1,
                raw as *const c_void,
            );
        }
        Self { texture: id, w, h }
    }

    pub fn load<T>(
        raw: Option<&[T]>,
        mode: TextureType,
        w: u32,
        h: u32,
        parm: TextureParm,
    ) -> Self {
        if let Some(raw) = raw {
            unsafe {
                return Self::new(raw.as_ptr(), mode, w, h, parm);
            }
        }
        unsafe { Self::new(null::<T>(), mode, w, h, parm) }
    }
    pub fn load_path(path: &Path, parm: TextureParm) -> Self {
        Self::load_image(&image::open(path).unwrap(), parm)
    }
    pub fn load_image(image: &DynamicImage, parm: TextureParm) -> Self {
        match image {
            DynamicImage::ImageLuma8(_) => {
                panic!("Texture2D Luma8 not support")
            }
            DynamicImage::ImageLumaA8(_) => {
                panic!("Texture2D LumaA8 not support")
            }
            DynamicImage::ImageLuma16(_) => {
                panic!("Texture2D Luma16 not support")
            }
            DynamicImage::ImageLumaA16(_) => {
                panic!("Texture2D LumaA16 not support")
            }

            DynamicImage::ImageRgb8(image_buffer) => Self::load(
                Some(image_buffer.as_bytes()),
                TextureType::RGB8,
                image_buffer.width(),
                image_buffer.height(),
                parm,
            ),
            DynamicImage::ImageRgba8(image_buffer) => Self::load(
                Some(image_buffer.as_bytes()),
                TextureType::RGBA8,
                image_buffer.width(),
                image_buffer.height(),
                parm,
            ),
            DynamicImage::ImageRgb16(image_buffer) => Self::load(
                Some(image_buffer.as_bytes()),
                TextureType::RGB16,
                image_buffer.width(),
                image_buffer.height(),
                parm,
            ),
            DynamicImage::ImageRgba16(image_buffer) => Self::load(
                Some(image_buffer.as_bytes()),
                TextureType::RGBA16,
                image_buffer.width(),
                image_buffer.height(),
                parm,
            ),
            DynamicImage::ImageRgb32F(image_buffer) => Self::load(
                Some(image_buffer.as_bytes()),
                TextureType::RGB32,
                image_buffer.width(),
                image_buffer.height(),
                parm,
            ),
            DynamicImage::ImageRgba32F(image_buffer) => Self::load(
                Some(image_buffer.as_bytes()),
                TextureType::RGBA32,
                image_buffer.width(),
                image_buffer.height(),
                parm,
            ),
            _ => {
                panic!("Texture2D load error,what is that?!");
            }
        }
    }

    pub fn vec2(&self) -> Vec2 {
        vec2(self.w as f32, self.h as f32)
    }
    pub fn with_size(w: u32, h: u32, type_: TextureType, fmt: TextureParm) -> Self {
        Self::load::<u8>(None, type_, w, h, fmt)
    }

    pub fn unbind() {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

#[derive(Clone, Copy)]
pub enum TextureType {
    RGBA8,
    RGB8,
    RED8,
    RGBA16,
    RGB16,
    RED16,
    RGBA32,
    RGB32,
    RED32,
}

//fmt,type
impl TextureType {
    pub const fn as_gl(&self) -> (GLenum, GLenum) {
        match self {
            TextureType::RGBA8 => (gl::RGBA, gl::UNSIGNED_BYTE),
            TextureType::RGB8 => (gl::RGB, gl::UNSIGNED_BYTE),
            TextureType::RED8 => (gl::RED, gl::UNSIGNED_BYTE),
            TextureType::RGBA16 => (gl::RGBA, gl::UNSIGNED_SHORT),
            TextureType::RGB16 => (gl::RGB, gl::UNSIGNED_SHORT),
            TextureType::RED16 => (gl::RED, gl::UNSIGNED_SHORT),
            TextureType::RGBA32 => (gl::RGBA, gl::FLOAT),
            TextureType::RGB32 => (gl::RGB, gl::FLOAT),
            TextureType::RED32 => (gl::RED, gl::FLOAT),
        }
    }
}

#[derive(Clone, Copy)]
pub struct TextureParm {
    min_filter: GLenum,
    mag_filter: GLenum,
    wrap_s: GLenum,
    wrap_t: GLenum,

    once_size: i32,
}
impl Default for TextureParm {
    fn default() -> Self {
        Self::new()
    }
}

impl TextureParm {
    pub const fn min_filter(&self, value: GLenum) -> Self {
        let mut edit = *self;
        edit.min_filter = value;
        edit
    }

    pub const fn mag_filter(&self, value: GLenum) -> Self {
        let mut edit = *self;
        edit.mag_filter = value;
        edit
    }

    pub const fn warp_s(&self, value: GLenum) -> Self {
        let mut edit = *self;
        edit.wrap_s = value;
        edit
    }

    pub const fn warp_t(&self, value: GLenum) -> Self {
        let mut edit = *self;
        edit.wrap_t = value;
        edit
    }
    pub const fn once_size(&self, value: i32) -> Self {
        let mut edit = *self;
        edit.once_size = value;
        edit
    }
    pub const fn new() -> Self {
        Self {
            min_filter: gl::NEAREST,
            mag_filter: gl::NEAREST,
            wrap_s: gl::CLAMP_TO_BORDER,
            wrap_t: gl::CLAMP_TO_BORDER,
            once_size: 4,
        }
    }
}

pub static BASE_PARM: TextureParm = TextureParm::new();

pub fn texture_parm(target: GLenum, parm: TextureParm) {
    unsafe {
        gl::TexParameteri(target, gl::TEXTURE_MIN_FILTER, parm.min_filter as GLint);
        gl::TexParameteri(target, gl::TEXTURE_MAG_FILTER, parm.mag_filter as GLint);
        gl::TexParameteri(target, gl::TEXTURE_WRAP_S, parm.wrap_s as GLint);
        gl::TexParameteri(target, gl::TEXTURE_WRAP_T, parm.wrap_t as GLint);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, parm.once_size);
    }
}

// pub fn base_parm(target: GLenum) {
//     unsafe {
//         gl::TexParameteri(target, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
//         gl::TexParameteri(target, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
//         gl::TexParameteri(target, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as GLint);
//         gl::TexParameteri(target, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as GLint);
//         // gl::TexParameteri(target, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
//         // gl::TexParameteri(target, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
//     }
// }
// pub fn mipmap_parm(target: GLenum) {
//     unsafe {
//         gl::TexParameteri(target, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
//         gl::TexParameteri(target, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
//         gl::TexParameteri(
//             target,
//             gl::TEXTURE_MIN_FILTER,
//             gl::LINEAR_MIPMAP_LINEAR as GLint,
//         );
//         gl::TexParameteri(target, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
//     }
//}
const fn texture(id: i32) -> GLenum {
    match id {
        0 => gl::TEXTURE0,
        1 => gl::TEXTURE1,
        2 => gl::TEXTURE2,
        3 => gl::TEXTURE3,
        4 => gl::TEXTURE4,
        5 => gl::TEXTURE5,
        6 => gl::TEXTURE6,
        7 => gl::TEXTURE7,
        8 => gl::TEXTURE8,
        9 => gl::TEXTURE9,
        10 => gl::TEXTURE10,
        11 => gl::TEXTURE11,
        12 => gl::TEXTURE12,
        13 => gl::TEXTURE13,
        14 => gl::TEXTURE14,
        15 => gl::TEXTURE15,
        16 => gl::TEXTURE16,
        17 => gl::TEXTURE17,
        _ => panic!("out texture unit"),
    }
}
fn bind_texture_unit(id: i32) {
    unsafe {
        gl::ActiveTexture(texture(id));
    }
}
