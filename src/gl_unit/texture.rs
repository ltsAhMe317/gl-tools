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

use crate::{
    gl_unit::define::DrawMode, TEX_VERTEX_STATIC, TEX_VERTEX_YFLIP_STATIC, VAO_MUT, VERTEX_MUT,
};

use super::define::{self, Filter, TextureParm, TextureType, VertexArrayAttribPointerGen};
use super::{program::PROGRAM2D_TWO, ConstBlend, FrameBuffer};
const TEXTURE_MAP_SPLIT: i32 = 1;

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

    pub fn get_pixel_size<T:Hash+Eq>(&self,map:&TextureMap<T>) -> (f32, f32) {
        (
            self.w * map.allocator.size().width as f32,
            self.h * map.allocator.size().width as f32,
        )
    }
}
pub struct TextureMap<T: Hash + Eq> {
    allocator: AtlasAllocator,
    frame: FrameBuffer,
    index: HashMap<T, UVindex>,
}
impl TextureMap<String> {
    pub fn new_files(path: impl AsRef<Path>, w: i32, h: i32) -> TextureMap<String> {
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
                            define::TextureParm::new(),
                        ));
                        texs.push((file_name, texture));
                    }
                }
                Err(err) => println!("failed read file {}", err),
            }
        }
        let mut map: TextureMap<String> = TextureMap::<String>::new(w, h);
        map.add(texs, true).unwrap();
        map
    }
}

impl<T: Hash + Eq> TextureMap<T> {
    pub fn new(w: i32, h: i32) -> Self {
        let rect_map = AtlasAllocator::new(Size::new(w, h));
        let uv_list: HashMap<T, UVindex> = HashMap::new();
        let texture = TextureWrapper(Texture2D::with_size(
            w as u32,
            h as u32,
            TextureType::RGBA8,
            define::TextureParm::new()
                .min_filter(Filter::Linear)
                .mag_filter(Filter::Linear),
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
    pub fn add(
        &mut self,
        vec: Vec<(T, TextureWrapper<Texture2D>)>,
        y_flip: bool,
    ) -> Result<(), &'static str> {
        if vec.is_empty() {
            return Ok(());
        }
        crate::gl_unit::const_blend(ConstBlend::SrcOnly);
        self.frame.bind(gl::FRAMEBUFFER);
        self.frame.view_port();
        let program = PROGRAM2D_TWO.deref();
        program.bind();
        program.put_matrix_name(
            Mat4::orthographic_rh_gl(0f32, 1f32, 0f32, 1f32, 1f32, -1f32),
            "project_mat",
        );

        program.put_matrix_name(Mat4::IDENTITY, "model_mat");
        program.put_texture(0, program.get_uniform("image"));
        VAO_MUT.bind(|vao| {
            vao.bind_pointer(
                if y_flip {
                    TEX_VERTEX_YFLIP_STATIC.deref()
                } else {
                    TEX_VERTEX_STATIC.deref()
                },
                VertexArrayAttribPointerGen::new::<f32>(1, 2),
            );
            vao.bind_pointer(
                VERTEX_MUT.deref(),
                VertexArrayAttribPointerGen::new::<f32>(0, 2),
            );
        });
        let mut uv_list = HashMap::new();
        for (name, texture) in vec.into_iter() {
            println!("{},{}", texture.w, texture.h);
            let texture = texture.as_ref();
            let uv;
            if texture.w == 0 || texture.h == 0 {
                uv = UVindex {
                    x: 0f32,
                    y: 0f32,
                    w: 0f32,
                    h: 0f32,
                }
            } else {
                let rect = match self.allocator.allocate(Size::new(
                    texture.w as i32 + TEXTURE_MAP_SPLIT,
                    texture.h as i32 + TEXTURE_MAP_SPLIT,
                )) {
                    Some(rect) => rect.rectangle,
                    None => {
                        return Err("can not allocate");
                    }
                };
                let size =self.allocator.size();
                uv = UVindex {
                    x: rect.min.x as f32 / size.width as f32,
                    y: rect.min.y as f32 / size.height as f32,
                    w: texture.w as f32 / size.width as f32,
                    h: texture.h as f32 / size.height as f32,
                };

                VERTEX_MUT.sub_data(
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
                texture.bind_unit(0);
                // println!("vao bind:{}",crate::gl_unit::debug::now_vao_id());
                
                VAO_MUT.bind(|vao|{vao.draw_arrays(DrawMode::Quads, 0, 4);});
            }
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
        ops::Deref,
        path::Path,
    };

    use glam::Mat4;

    use crate::{
        gl_unit::{
            define::{DrawMode, TextureParm, VertexArrayAttribPointerGen},
            program::PROGRAM2D_ONE,
            texture::{Texture, TextureWrapper},
            window::Window,
            GLcontext,
        },
        TEX_VERTEX_MUT, VAO_MUT, VERTEX_MUT,
    };

    use super::{Texture2D, TextureMap};

    #[test]
    fn texture_map() {
        let mut window = Window::new(800, 600, "test", false);
        let mut context = GLcontext::with(&mut window);

        let tex_list = {
            let mut map = Vec::new();
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
        for (str, tex) in tex_list.iter() {
            what.push((str.to_string(), tex.as_ref()));
        }
        // let he = "hehe".to_string();
        // let what = vec![(&he, &tex)];

        println!("load map...");
        let mut map = TextureMap::new(4000, 4000);
        map.add(tex_list, true).unwrap();
        // map.frame
        //     .texture
        //     .as_ref()
        //     .unwrap()
        //     .get_image()
        //     .save(Path::new("test map.png"))
        //     .unwrap();
        window.view_port();
        window.window.show();

        VAO_MUT.bind_pointer(
            VERTEX_MUT.deref(),
            VertexArrayAttribPointerGen::new::<f32>(0, 2),
        );
        VAO_MUT.bind_pointer(
            TEX_VERTEX_MUT.deref(),
            VertexArrayAttribPointerGen::new::<f32>(1, 2),
        );

        while !window.update() {
            context.draw_option(&mut window, |_, _| {
                let program = &PROGRAM2D_ONE;
                program.bind();
                program.put_matrix_name(Mat4::IDENTITY, "project_mat");
                program.put_matrix_name(Mat4::IDENTITY, "model_mat");
                program.put_texture(0, program.get_uniform("image"));
                map.get_tex().bind_unit(0);

                VERTEX_MUT.sub_data(&[-1f32, 1f32, 1f32, 1f32, 1f32, -1f32, -1f32, -1f32], 0);

                let index = map.get_uv(&"GINO".to_string()).unwrap();
                TEX_VERTEX_MUT.sub_data(
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
                VAO_MUT.draw_arrays(DrawMode::Quads, 0, 4);
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
    fn bind_unit(&self, id: u32) {
        active_texture_unit(define::Texture::Unit(id));
        self.send_to_texture();
    }
    fn send_date<T>(&self, type_: TextureType, x: i32, y: i32, w: i32, h: i32, date: &[T]);
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

    fn delete(&self) {
        println!("Texture1D:{} leave", self.texture);
        unsafe {
            gl::DeleteTextures(1, &self.texture as *const GLuint);
        }
    }

    fn send_date<T>(&self, type_: TextureType, x: i32, y: i32, w: i32, h: i32, date: &[T]) {
        self.send_to_texture();
        let (format, type_) = type_.as_gl();
        unsafe {
            gl::TexSubImage2D(
                gl::TEXTURE_1D,
                0,
                x,
                y,
                w,
                h,
                format,
                type_,
                date.as_ptr() as *const c_void,
            );
        }
    }
}
impl Texture1D {
    pub fn new_size(size: u32) -> Self {
        Self::new::<u8>(null(), TextureType::RED8, size, TextureParm::new())
    }

    pub fn load<T>(raw: Option<&[T]>, mode: TextureType, size: u32, parm: TextureParm) -> Self {
        if let Some(raw) = raw {
            Self::new(raw.as_ptr(), mode, size, parm)
        } else {
            Self::new(null::<T>(), mode, size, parm)
        }
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

    fn send_date<T>(&self, type_: TextureType, x: i32, y: i32, w: i32, h: i32, date: &[T]) {
        self.send_to_texture();
        let (inter, real) = type_.as_gl();
        unsafe {
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                x,
                y,
                w,
                h,
                inter,
                real,
                date.as_ptr() as *const c_void,
            );
        }
    }

    fn delete(&self) {
        println!("Texture1D:{} leave", self.texture);
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

pub fn texture_parm(target: GLenum, parm: TextureParm) {
    unsafe {
        gl::TexParameteri(
            target,
            gl::TEXTURE_MIN_FILTER,
            parm.min_filter.as_gl() as GLint,
        );
        gl::TexParameteri(
            target,
            gl::TEXTURE_MAG_FILTER,
            parm.mag_filter.as_gl() as GLint,
        );
        gl::TexParameteri(target, gl::TEXTURE_WRAP_S, parm.wrap_s.as_gl() as GLint);
        gl::TexParameteri(target, gl::TEXTURE_WRAP_T, parm.wrap_t.as_gl() as GLint);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, parm.once_load_size);
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
fn active_texture_unit(id: define::Texture) {
    unsafe {
        gl::ActiveTexture(id.as_gl());
    }
}
