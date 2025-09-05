pub mod model;
pub mod video;
use crate::gl_unit::define::{DrawMode, VertexArrayAttribPointerGen};
use crate::gl_unit::program::{Program, PROGRAM2D_ONE};
use crate::gl_unit::texture::{Texture, TextureMap, UVindex};
use crate::gl_unit::window::Window;
use crate::gl_unit::{self, ConstBlend};
use crate::{VAO_MUT, VERTEX_BIG_MUT};
use core::fmt::{Debug, Formatter};
use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3, Vec4Swizzles};


// use rusty_spine::{
//     AnimationState, AnimationStateData, Atlas, AttachmentType, Physics, Skeleton, SkeletonBinary,
//     SkeletonData, SkeletonJson,
// };

use std::collections::HashMap;
use std::fs;
use std::ops::{Deref, Range};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use xml::reader::XmlEvent;

pub fn window_ort(window_size: (i32, i32)) -> Mat4 {
    let (w, h) = window_size;
    let (w, h) = (w as f32 / 2f32, h as f32 / 2f32);
    Mat4::orthographic_rh_gl(-w, w, -h, h, -1f32, 1f32)
}
pub fn window_ort_asp(window_size: (i32, i32)) -> Mat4 {
    let (w, h) = window_size;
    let asp = w as f32 / h as f32;
    Mat4::orthographic_rh_gl(-asp, asp, -1f32, 1f32, 2f32, -2f32)
}

pub trait Camera {
    fn as_mat(&self) -> Mat4;
}
pub struct Camera2D {
    pub location: Vec2,
}
impl Camera2D {
    pub fn new() -> Self {
        Self {
            location: vec2(0f32, 0f32),
        }
    }
}
impl Camera for Camera2D {
    fn as_mat(&self) -> Mat4 {
        Mat4::from_translation(self.location.extend(0f32))
    }
}
pub fn vec_from_rad(pitch:f32,yaw:f32)->Vec3{
 Vec3 {
            x: yaw.sin() * pitch.cos(),
            y: pitch.sin(),
            z: yaw.cos() * pitch.cos(),
        }.normalize()
}
pub struct Camera3D {
    pub location: Vec3,
    pub look_at:Vec3,
    pub fov:f32,

    asp:f32
}
impl Camera3D {
    pub fn new(window:&Window) -> Self {
        let (w,h) = window.window.get_size();
        Self{
            location:vec3(0f32, 0f32, 0f32),
            fov: 70f32,
            asp:w as f32/h as f32,
            look_at: vec3(0f32,0f32,1f32),
        }
    }
     pub fn look_rad(&mut self, pitch: f32,yaw: f32,) {
        self.look_at = self.location + vec_from_rad(pitch,yaw);
    }
    pub fn go_vec(&mut self,vec:Vec3,range:f32){
        self.location += vec.normalize() * range;
    }
}
impl Camera for Camera3D {
    fn as_mat(&self) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov.to_radians(),self.asp, 1f32, -1f32) * Mat4::look_at_rh(self.location, self.look_at, vec3(0f32, 1f32, 0f32))
    }
}

// static mut SPINE_VECTOR: [f32; 2000] = [0f32; 2000];

// static mut SPINE_TEX_CROOD: [f32; 2000] = [0f32; 2000];

// static mut SPINE_VERTEX_BUFFER: Lazy<VertexBuffer<f32>> =
//     Lazy::new(|| VertexBuffer::new_array(&unsafe { SPINE_VECTOR.to_vec() }, gl::DYNAMIC_DRAW));
// static mut SPINE_CROOD_BUFFER: Lazy<VertexBuffer<f32>> =
//     Lazy::new(|| VertexBuffer::new_array(&unsafe { SPINE_TEX_CROOD.to_vec() }, gl::DYNAMIC_DRAW));
// static mut SPINE_VAO: Lazy<VertexArray> = Lazy::new(|| unsafe {
//     let vao = VertexArray::new();
//     vao.with(&SPINE_VERTEX_BUFFER, 0, 2, gl::FLOAT, 0);
//     vao.with(&SPINE_CROOD_BUFFER, 1, 2, gl::FLOAT, 0);
//     vao
// });
// static mut SPINE_MESH_VERTEX: [f32; 2000] = [0f32; 2000];

// pub struct SpinePlay {
//     pub skeleton: Skeleton,
//     pub state: AnimationState,
// }

// impl SpinePlay {
//     pub fn update(&mut self, delta: f32) {
//         self.state.update(delta);
//         self.state.apply(&mut self.skeleton);
//         self.skeleton.update(delta);
//         self.skeleton.update_world_transform(Physics::Update);
//     }
//     pub fn get_anim_name(&self) -> Option<String> {
//         self.state
//             .get_current(0)
//             .as_ref()
//             .map(|track| track.animation().name().to_string())
//     }
//     pub fn check_state(&self, name: &str) -> bool {
//         match self.get_anim_name() {
//             None => false,
//             Some(now_name) => now_name.eq(name),
//         }
//     }
// }

// pub static SPINE_OFFSET: f32 = 0.000001f32;
// pub struct Spine {
//     pub img_data: Texture2D,
//     pub sk_data: Arc<SkeletonData>,
//     pub anim_data: Arc<AnimationStateData>,
// }
// impl Spine {
//     pub fn json(dir: &Path) -> Self {
//         let dir_path = dir.to_str().unwrap();
//         let dir_name = &dir_path[dir_path.rfind('\\').unwrap()..];
//         let atlas = Arc::new(
//             Atlas::new_from_file(dir_path.to_string().add(dir_name).add(".atlas").as_str())
//                 .unwrap(),
//         );
//         // let mut loader = SkeletonBinary::new(atlas);
//         // let sk_data = loader.read_skeleton_data_file(dir_path.to_string().add(dir_name).add(".skel").as_str()).expect("wtf");
//         let loader = SkeletonJson::new(atlas);
//         let sk_data = Arc::new(
//             loader
//                 .read_skeleton_data_file(dir_path.to_string().add(dir_name).add(".json").as_str())
//                 .expect("wtf"),
//         );
//         let anim_data = Arc::new(AnimationStateData::new(sk_data.clone()));
//         // anim.set_animation_by_name(0,"saihong1",true);
//         Spine {
//             img_data: Texture2D::load_path(
//                 Path::new(&dir_path.to_string().add(dir_name).add(".png")),
//                 TextureParm::new(),
//             ),
//             sk_data,
//             anim_data,
//         }
//     }
//     pub fn code(dir: &Path) -> Self {
//         let dir_path = dir.to_str().unwrap();
//         let dir_name = &dir_path[dir_path.rfind('\\').unwrap()..];
//         let atlas = Arc::new(
//             Atlas::new_from_file(dir_path.to_string().add(dir_name).add(".atlas").as_str())
//                 .unwrap(),
//         );
//         // let mut loader = SkeletonBinary::new(atlas);
//         // let sk_data = loader.read_skeleton_data_file(dir_path.to_string().add(dir_name).add(".skel").as_str()).expect("wtf");
//         let loader = SkeletonBinary::new(atlas);
//         let sk_data = loader
//             .read_skeleton_data_file(dir_path.to_string().add(dir_name).add(".skel").as_str())
//             .expect("wtf");

//         let sk_data_arc = Arc::new(sk_data);
//         let anim_data = Arc::new(AnimationStateData::new(sk_data_arc.clone()));
//         // anim.set_animation_by_name(0,"saihong1",true);
//         Spine {
//             img_data: Texture2D::load_path(
//                 Path::new(&dir_path.to_string().add(dir_name).add(".png")),
//                 TextureParm::new(),
//             ),
//             sk_data: sk_data_arc,
//             anim_data,
//         }
//     }

//     pub fn new_play(&self) -> SpinePlay {
//         SpinePlay {
//             skeleton: Skeleton::new(self.sk_data.clone()),
//             state: AnimationState::new(self.anim_data.clone()),
//         }
//     }

//     pub fn get_anim(&self) -> AnimationState {
//         let anim_data = AnimationStateData::new(self.sk_data.clone());
//         AnimationState::new(Arc::from(anim_data))
//     }

//     pub fn render(
//         &self,
//         play: &SpinePlay,
//         is_mask: bool,
//         camera: &Camera,
//         program: &Program,
//         tranform: Mat4,
//     ) {
//         todo!();
//         program.bind();
//         let base_mat = tranform * camera.ort;
//         let mut num = 0;
//         let is_shadow = program.get_uniform("is_shadow");
//         let is_mask_index = program.get_uniform("is_mask");
//         if !is_mask {
//             program.put_bool(is_mask_index, false);
//         }
//         self.img_data.bind_unit(0);
//         program.put_texture(0, program.get_uniform("image"));
//         for mut object in play.skeleton.draw_order() {
//             let mut tex_color: Vec<f32> = Vec::new();
//             match object.attachment() {
//                 None => {}
//                 Some(mut attach) => {
//                     program.put_matrix(
//                         &(Mat4::from_translation(vec3(0f32, 0f32, SPINE_OFFSET * num as f32))
//                             * base_mat),
//                         program.get_uniform("model_mat"),
//                     );
//                     let (solt, attach) = attach.unwrap_parent_child();
//                     match attach.attachment_type() {
//                         AttachmentType::Region => unsafe {
//                             let attach_region = attach.as_region().unwrap();
//                             let uvs = attach_region.uvs();
//                             SPINE_TEX_CROOD[0..uvs.len()].copy_from_slice(&uvs);
//                             attach_region.compute_world_vertices(solt, &mut SPINE_VECTOR, 0, 2);
//                             let wdf = attach_region
//                                 .renderer_object()
//                                 .get_atlas_region()
//                                 .unwrap()
//                                 .page()
//                                 .renderer_object();
//                             SPINE_VERTEX_BUFFER.sub(&SPINE_VECTOR, 0);
//                             SPINE_CROOD_BUFFER.sub(&SPINE_TEX_CROOD, 0);
//                             SPINE_VAO.bind();
//                             program.draw_rect(1);
//                         },
//                         AttachmentType::BoundingBox => {
//                             println!("bounding")
//                         }
//                         AttachmentType::Mesh => unsafe {
//                             let attach_mesh = attach.as_mesh().unwrap();
//                             let max_mesh_len = attach_mesh.world_vertices_length();
//                             if max_mesh_len > SPINE_MESH_VERTEX.len() as i32 {
//                                 continue;
//                             } else {
//                                 attach_mesh.compute_world_vertices(
//                                     solt,
//                                     0,
//                                     attach_mesh.world_vertices_length(),
//                                     &mut SPINE_MESH_VERTEX,
//                                     0,
//                                     2,
//                                 );
//                                 let uvs_list = attach_mesh.uvs();

//                                 let uvs_list = attach_mesh.uvs();
//                                 for i in 0..attach_mesh.triangles_count() {
//                                     let better_index = (i * 2) as usize;
//                                     let index =
//                                         (*attach_mesh.triangles().offset(i as isize) << 1) as usize;
//                                     SPINE_VECTOR[better_index] = SPINE_MESH_VERTEX[index];
//                                     SPINE_VECTOR[better_index + 1] = SPINE_MESH_VERTEX[index + 1];

//                                     SPINE_TEX_CROOD[better_index] = *uvs_list.add(index);
//                                     SPINE_TEX_CROOD[better_index + 1] = *uvs_list.add(index + 1);
//                                 }
//                                 SPINE_VERTEX_BUFFER.sub(&SPINE_VECTOR, 0);
//                                 SPINE_CROOD_BUFFER.sub(&SPINE_TEX_CROOD, 0);
//                                 SPINE_VAO.bind();

//                                 gl::DrawArrays(gl::TRIANGLES, 0, attach_mesh.triangles_count());
//                             }
//                         },
//                         AttachmentType::LinkedMesh => {
//                             println!("linkedMesh")
//                         }
//                         AttachmentType::Path => {
//                             println!("Path")
//                         }
//                         AttachmentType::Point => {
//                             println!("Point")
//                         }
//                         AttachmentType::Clipping => {
//                             println!("what")
//                         }
//                         AttachmentType::Unknown => {
//                             println!("not")
//                         }
//                     }
//                 }
//             }
//             num += 1;
//         }
//         program.put_bool(is_mask_index, true);
//     }
//     // pub fn render_base(&self,is_depth:bool,ort:Mat4, camera:&Mat4, transform:&Mat4){
//     //     unsafe { self.render(is_depth,ort, &SPINE_PROGRAM,transform); }
//     // }
// }

#[derive(Clone, Copy)]
pub struct Tick {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub sx: Option<f32>,
    pub sy: Option<f32>,
    pub kx: Option<f32>,
    pub ky: Option<f32>,
    pub alpha: Option<f32>,
    pub texture: Option<UVindex>,
}
impl Debug for Tick {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "T<").unwrap();
        if let Some(value) = self.x {
            write!(f, "x:{} ", value).unwrap();
        }
        if let Some(value) = self.y {
            write!(f, "y:{} ", value).unwrap();
        }
        if let Some(value) = self.sx {
            write!(f, "sx:{} ", value).unwrap();
        }
        if let Some(value) = self.sy {
            write!(f, "sy:{} ", value).unwrap();
        }
        if let Some(value) = self.alpha {
            write!(f, "a:{} ", value).unwrap();
        }
        if let Some(value) = &self.texture {
            write!(f, "texture {:?}", value).unwrap();
        }
        write!(f, ">")
    }
}
impl Tick {
    pub const fn mix(&self, tick: &Tick, f: f32) -> Self {
        Tick {
            x: Self::liner_option(self.x, tick.x, f),
            y: Self::liner_option(self.y, tick.y, f),
            sx: Self::liner_option(self.sx, tick.sx, f),
            sy: Self::liner_option(self.sy, tick.sy, f),
            kx: Self::liner_option(self.kx, tick.kx, f),
            ky: Self::liner_option(self.ky, tick.ky, f),
            alpha: Self::liner_option(self.alpha, tick.alpha, f),
            texture: match tick.texture {
                Some(uv) => Some(uv),
                None => self.texture,
            },
        }
    }

    const fn liner_option(a: Option<f32>, b: Option<f32>, f: f32) -> Option<f32> {
        match (a, b) {
            (Some(av), Some(bv)) => Some(Self::liner(av, bv, f)),
            (Some(av), None) => Some(av),
            (None, Some(bv)) => Some(bv),
            _ => None,
        }
    }

    const fn liner(start: f32, end: f32, fraction: f32) -> f32 {
        start + (end - start) * fraction
    }

    pub const fn update(&self, tick: &Tick) -> Self {
        let mut clone = *self;
        if let Some(value) = tick.x {
            clone.x = Some(value);
        }
        if let Some(value) = tick.y {
            clone.y = Some(value);
        }
        if let Some(value) = tick.sx {
            clone.sx = Some(value);
        }
        if let Some(value) = tick.sy {
            clone.sy = Some(value);
        }
        if let Some(value) = tick.kx {
            clone.kx = Some(value);
        }
        if let Some(value) = tick.ky {
            clone.ky = Some(value);
        }
        if let Some(value) = tick.alpha {
            clone.alpha = Some(value);
        }
        if let Some(value) = tick.texture {
            clone.texture = Some(value);
        }
        clone
    }
    pub const fn default_none() -> Self {
        Self {
            x: None,
            y: None,
            sx: None,
            sy: None,
            kx: None,
            ky: None,
            alpha: None,
            texture: None,
        }
    }
    pub const fn default() -> Self {
        Self {
            x: Some(0.0),
            y: Some(0.0),
            sx: Some(1.0),
            sy: Some(1.0),
            kx: Some(0.0),
            ky: Some(0.0),
            alpha: Some(1.0),
            texture: None,
        }
    }
}
#[cfg(test)]
pub mod test {
    use std::path::Path;

    use glam::Mat4;

    use crate::gl_unit::{texture::TextureMap, window::Window, GLcontext};

    use super::Reanim;

    #[test]
    fn reanim() {
        let mut window = Window::new(800, 600, "reanim test", false);
        let mut context = GLcontext::with(&mut window);

        let tex_map = TextureMap::new_files("./test_res/reanim_texture/", 800, 800);
        let reanim = Reanim::from_file(Path::new("./test_res/reanim/Coin_silver.reanim"), &tex_map);
        let mut play = reanim.make_player();
        play.add_anim(0, "loop", super::PlayMode::Loop);
        window.window.show();
        while !window.update() {
            context.draw_option(&mut window, |_, window| {
                play.render(window.window.get_size(), &tex_map, Mat4::IDENTITY);
            });
            play.update(window.delta_count.delta as f32);
        }
    }
}

pub struct Reanim {
    pub date: Arc<ReanimData>,
}
impl Reanim {
    pub fn from_file(path: &Path, textures: &TextureMap<String>) -> Self {
        Self {
            date: Arc::new(ReanimData::new(
                fs::read_to_string(path).unwrap().as_str(),
                textures,
            )),
        }
    }
    pub fn new(xml: &str, textures: &TextureMap<String>) -> Self {
        Self {
            date: Arc::new(ReanimData::new(xml, textures)),
        }
    }
    pub fn make_player(&self) -> ReanimPlayer {
        let mut player = ReanimPlayer {
            date: self.date.clone(),
            tracks: Vec::with_capacity(self.date.tracks.len()),
            anims_delta: HashMap::with_capacity(self.date.anim.len()),
            anim_queue: array_init::array_init(|_| Vec::new()),
        };
        for _ in self.date.tracks.iter() {
            player.tracks.push(Tick::default());
        }
        player.anims_delta.insert("loop".to_string(), 0f32);

        for (name, _) in self.date.anim.iter() {
            player.anims_delta.insert(
                name.to_string(),
                self.date.anim.get(name).unwrap().start as f32 / self.date.fps,
            );
        }

        player.track_update(0);
        player
    }
}
pub struct ReanimData {
    fps: f32,
    len: usize,
    pub tracks: Vec<(String, Vec<Tick>)>,
    pub anim: HashMap<String, Range<usize>>,
}
impl Debug for ReanimData {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "")
    }
}
impl ReanimData {
    pub fn from_file(path: &Path, textures: &TextureMap<String>) -> Self {
        Self::new(fs::read_to_string(path).unwrap().as_str(), textures)
    }
    pub fn new(xml: &str, textures: &TextureMap<String>) -> Self {
        let mut xml = xml::EventReader::from_str(xml);
        let mut anims = HashMap::new();
        let mut tracks = Vec::new();
        let mut len = 0;
        let mut fps: f32 = 0.0;

        let mut is_anim = false;
        let mut now_tick: usize = 0;
        let mut temp_tick = Tick::default_none();
        let mut temp_track = None;
        let mut temp_track_name = None;
        let mut temp_frame: Vec<(usize, bool)> = Vec::new();
        loop {
            match xml.next().unwrap() {
                XmlEvent::StartElement {
                    name,
                    attributes: _,
                    namespace: _,
                } => {
                    match name.to_string().as_str() {
                        "fps" => {
                            if let Ok(XmlEvent::Characters(str)) = xml.next() {
                                fps = f32::from_str(&str).unwrap()
                            }
                        }
                        "track" => {
                            temp_track = Some(Vec::new());
                            now_tick = 0;
                        }
                        "name" => {
                            if let Ok(XmlEvent::Characters(str)) = xml.next() {
                                is_anim = false;
                                if str.len() > 5 && str[..5].eq("anim_") {
                                    is_anim = true;
                                    temp_track_name = Some(str[5..].to_string());
                                    temp_frame.clear();
                                } else {
                                    temp_track_name = Some(str);
                                }
                            }
                        }
                        "t" => {
                            temp_tick = Tick::default_none();
                            now_tick += 1;
                        }

                        "x" => {
                            if let Ok(XmlEvent::Characters(str)) = xml.next() {
                                temp_tick.x = Some(f32::from_str(&str).unwrap());
                            }
                        }
                        "y" => {
                            if let Ok(XmlEvent::Characters(str)) = xml.next() {
                                temp_tick.y = Some(f32::from_str(&str).unwrap());
                            }
                        }
                        "sx" => {
                            if let Ok(XmlEvent::Characters(str)) = xml.next() {
                                temp_tick.sx = Some(f32::from_str(&str).unwrap());
                            }
                        }
                        "sy" => {
                            if let Ok(XmlEvent::Characters(str)) = xml.next() {
                                temp_tick.sy = Some(f32::from_str(&str).unwrap());
                            }
                        }
                        "kx" => {
                            if let Ok(XmlEvent::Characters(str)) = xml.next() {
                                temp_tick.kx = Some(f32::from_str(&str).unwrap());
                            }
                        }
                        "ky" => {
                            if let Ok(XmlEvent::Characters(str)) = xml.next() {
                                temp_tick.ky = Some(f32::from_str(&str).unwrap());
                            }
                        }

                        "a" => {
                            if let Ok(XmlEvent::Characters(str)) = xml.next() {
                                temp_tick.alpha = Some(f32::from_str(&str).unwrap());
                            }
                        }
                        "i" => {
                            if let Ok(XmlEvent::Characters(str)) = xml.next() {
                                temp_tick.texture =
                                    Some(textures.get_uv(&str).unwrap_or_else(|| {
                                        panic!("reanim init not find uv:{}", str)
                                    }));
                            }
                        }

                        "f" => {
                            if let Ok(XmlEvent::Characters(str)) = xml.next() {
                                if str.eq("0") {
                                    //true
                                    temp_frame.push((now_tick, true));
                                } else {
                                    //false
                                    temp_frame.push((now_tick, false));
                                }
                            }
                        }
                        &_ => {}
                    }
                }
                XmlEvent::EndElement { name } => {
                    match name.to_string().as_str() {
                        "track" => {
                            len = temp_track.as_ref().unwrap().len();
                            if !is_anim {
                                //push track
                                tracks.push((
                                    temp_track_name.take().unwrap(),
                                    temp_track.take().unwrap(),
                                ));
                            } else {
                                //push anim
                                let mut start = 1;
                                let mut end = len + 1;
                                for (index, value) in temp_frame.iter().cloned() {
                                    if value {
                                        start = index;
                                    } else if index > start {
                                        end = index;
                                    }
                                }
                                anims.insert(temp_track_name.take().unwrap(), start - 1..end - 1);
                                is_anim = false;
                            }
                        }
                        "t" => temp_track.as_mut().unwrap().push(temp_tick),
                        &_ => {}
                    }
                }
                XmlEvent::EndDocument => {
                    break;
                }
                _ => {}
            }
        }
        anims.insert("loop".to_string(), 0..len);
        Self {
            anim: anims,
            fps,
            len,
            tracks,
        }
    }
    pub fn get_anim_range(&self, name: &str) -> Range<usize> {
        self.anim.get(name).unwrap().clone()
    }
}

fn mat4_skew(kx: f32, ky: f32, sx: f32, sy: f32) -> Mat4 {
    Mat4 {
        x_axis: vec4(sx * kx.cos(), -kx.sin(), 0.0, 0.0),
        y_axis: vec4(ky.sin(), sy * ky.cos(), 0.0, 0.0),
        z_axis: vec4(0.0, 0.0, 1.0, 0.0),
        w_axis: vec4(0.0, 0.0, 0.0, 1.0),
    }
}
#[derive(Clone, Copy)]
pub enum PlayMode {
    Never,
    Loop,
    Speed(f32),
    Delay(f32),
    DelayLoop(f32),
    DelayLoopSpeed(f32, f32),
    Once(usize),
}

pub struct AnimState {
    name: String,
    mode: PlayMode,
}

pub struct ReanimPlayer {
    pub date: Arc<ReanimData>,
    tracks: Vec<Tick>,
    pub anims_delta: HashMap<String, f32>,
    pub anim_queue: [Vec<AnimState>; 32],
}
unsafe impl Send for ReanimPlayer {}
unsafe impl Sync for ReanimPlayer {}
impl ReanimPlayer {
    pub fn render_program(
        &self,
        window_size: (i32, i32),
        tex_map: &TextureMap<String>,
        program: &Program,
        mat4: Mat4,
    ) {
        //still need fix alpha

        let mut vertexs = Vec::new();
        for tick in self.tracks.iter() {
            if let Some(texture) = tick.texture.as_ref() {
                let mat = mat4_skew(
                    (tick.kx.unwrap()).to_radians(),
                    tick.ky.unwrap().to_radians(),
                    tick.sx.unwrap(),
                    tick.sy.unwrap(),
                );
                let vert = {
                    let (w, h) = texture.get_pixel_size(tex_map);
                    let vert_org = [vec2(0f32, h), vec2(w, h), vec2(w, 0f32), vec2(0f32, 0f32)];
                    vert_org.map(|vert| {
                        (mat * vert.extend(0f32).extend(1f32)).xy()
                            + vec2(tick.x.unwrap(), tick.y.unwrap())
                    })
                };
                let uv = texture.get_uv();
                vertexs.extend_from_slice(&[
                    vert[0].x, vert[0].y, uv[0], uv[1], vert[1].x, vert[1].y, uv[2], uv[3],
                    vert[2].x, vert[2].y, uv[4], uv[5], vert[3].x, vert[3].y, uv[6], uv[7],
                ]);
            }
        }
        tex_map.get_tex().bind_unit(0);
        //render
        gl_unit::const_blend(ConstBlend::Normal);
        program.bind();
        VAO_MUT.bind(|vao|{
        vao.bind_pointer(
            VERTEX_BIG_MUT.deref(),
            VertexArrayAttribPointerGen::new::<f32>(0, 4),
        );
        VERTEX_BIG_MUT.sub_data(&vertexs, 0);
        program.put_texture(0, program.get_uniform("image"));
        program.put_matrix(mat4, program.get_uniform("model_mat"));
        let (w, h) = window_size;
        let (w, h) = (w as f32 / 2f32, h as f32 / 2f32);
        program.put_matrix_name(
            Mat4::orthographic_rh_gl(-w, w, -h, h, 1f32, -1f32),
            "project_mat",
        );
        
        vao.draw_arrays(DrawMode::Quads,0,vertexs.len() as i32 / 16);});
    }
    pub fn render(&self, window_size: (i32, i32), tex_map: &TextureMap<String>, mat: Mat4) {
        self.render_program(window_size, tex_map, &PROGRAM2D_ONE, mat);
    }
    pub fn update(&mut self, delta: f32) {
        // let mut some = TextOptions::default();
        // some.color = Color::from_rgb(0,0,0);
        // some.size = 20f32;
        // message(|message|{
        //     message.strings.clear();
        //     for track in self.tracks.iter() {
        //         message.send(&format!("{:?}",track),Some(some));
        //     }
        // });
        for index in 0..self.anim_queue.len() {
            if !self.anim_queue.get(index).unwrap().is_empty() {
                let name = self
                    .anim_queue
                    .get(index)
                    .unwrap()
                    .first()
                    .unwrap()
                    .name
                    .clone();
                match &mut self
                    .anim_queue
                    .get_mut(index)
                    .unwrap()
                    .get_mut(0)
                    .unwrap()
                    .mode
                    .clone()
                {
                    PlayMode::Loop => {
                        self.update_anim(&name, delta, true);
                    }
                    PlayMode::Once(count) => {
                        let bool = self.update_anim(&name, delta, false);
                        if bool {
                            if *count <= 0 {
                                self.anim_queue.get_mut(index).unwrap().remove(0);
                                *self.anims_delta.get_mut(&name).unwrap() =
                                    self.date.get_anim_range(&name).start as f32 / self.date.fps;
                            } else {
                                *count -= 1;
                            }
                        }
                    }
                    PlayMode::Delay(delay) => {
                        if *delay > 0f32 {
                            if let PlayMode::Delay(delay) = &mut self
                                .anim_queue
                                .get_mut(index)
                                .unwrap()
                                .get_mut(0)
                                .unwrap()
                                .mode
                            {
                                *delay -= delta;
                            }
                        } else if self.update_anim(&name, delta, false) {
                            self.anim_queue.get_mut(index).unwrap().remove(0);
                        }
                    }
                    PlayMode::Speed(speed) => {
                        self.update_anim(&name, delta * *speed, true);
                    }
                    PlayMode::DelayLoop(delay) => {
                        if *delay > 0f32 {
                            if let PlayMode::DelayLoop(delay) = &mut self
                                .anim_queue
                                .get_mut(index)
                                .unwrap()
                                .get_mut(0)
                                .unwrap()
                                .mode
                            {
                                *delay -= delta;
                            }
                        } else {
                            self.update_anim(&name, delta, false);
                        }
                    }
                    PlayMode::DelayLoopSpeed(delay, speed) => {
                        if *delay > 0f32 {
                            if let PlayMode::DelayLoopSpeed(delay, _) = &mut self
                                .anim_queue
                                .get_mut(index)
                                .unwrap()
                                .get_mut(0)
                                .unwrap()
                                .mode
                            {
                                *delay -= delta;
                            }
                        } else {
                            self.update_anim(&name, delta * *speed, true);
                        }
                    }
                    PlayMode::Never => {
                        self.update_anim(&name, delta, false);
                    }
                }
            }
        }
    }
    pub fn rewind_ticks(&mut self) {
        for tick in self.tracks.iter_mut() {
            *tick = Tick::default();
        }
    }
    pub fn clean_anim(&mut self, track: usize) {
        self.anim_queue[track].clear();
    }
    pub fn clean_anim_all(&mut self) {
        for num in 0..self.anim_queue.len() {
            self.clean_anim(num);
        }
    }

    fn update_anim(&mut self, anim_name: &str, delta: f32, next: bool) -> bool {
        let mut is_end = false;
        let range = self.date.get_anim_range(anim_name);

        let time = self.anims_delta.get_mut(anim_name).unwrap();
        let start_time = range.start as f32 / self.date.fps;
        if *time < start_time {
            *time = start_time;
        }
        *time += delta;
        let index = (*time * self.date.fps) as usize;
        if !next {
            if index >= range.end - 1 {
                *time = start_time;
                is_end = true;
            }

            if !is_end {
                let time_fps = *time * self.date.fps;
                let time_index = time_fps as usize;
                self.track_update_line(time_index, time_index + 1, time_fps - time_index as f32);
            }
        } else {
            if index >= range.end {
                *time = start_time;
                is_end = true;
            }

            if !is_end {
                let time_fps = *time * self.date.fps;
                let time_index = time_fps as usize;
                let time_index_next = {
                    if index == range.end - 1 {
                        range.start
                    } else {
                        index + 1
                    }
                };
                self.track_update_line(time_index, time_index_next, time_fps - time_index as f32);
            }
        }

        is_end
    }
    pub fn anim_time(&self, anim_name: &str) -> f32 {
        *self.anims_delta.get(anim_name).unwrap()
    }
    pub fn anim_name(&self, index: usize) -> Option<&str> {
        Some(&self.anim_queue.get(index).unwrap().first()?.name)
    }
    pub fn set_anim_time(&mut self, anim_name: &str, time: f32) {
        *self.anims_delta.get_mut(anim_name).unwrap() = time;
    }
    pub fn get_tick(&self, anim_name: &str) -> Option<&Tick> {
        let mut count = 0;
        for (track_name, _) in self.date.tracks.iter() {
            if track_name.eq(anim_name) {
                return self.tracks.get(count);
            }
            count += 1;
        }
        None
    }

    pub fn add_anim(&mut self, index: usize, name: &str, mode: PlayMode) {
        self.anim_queue.get_mut(index).unwrap().push(AnimState {
            name: name.to_string(),
            mode,
        })
    }
    pub fn set_anim(&mut self, index: usize, name: &str, mode: PlayMode) {
        self.anim_queue.get_mut(index).unwrap().clear();
        self.add_anim(index, name, mode);
    }
    pub fn track_update_line(&mut self, tick: usize, tick2: usize, f: f32) {
        // println!("{} to {},f:{}",tick,tick2,f);
        let mut count = 0;
        for track in self.tracks.iter_mut() {
            let date = &self.date.tracks.get(count).unwrap().1;
            if let (Some(next), Some(next2)) = (date.get(tick), date.get(tick2)) {
                *track = track.update(&next.mix(next2, f));
            } else {
                panic!("not found:{},{}", tick, tick2);
            }
            count += 1;
        }
    }
    fn track_update(&mut self, tick: usize) {
        let mut count = 0;
        for track in self.tracks.iter_mut() {
            let date = &self.date.tracks.get(count).unwrap().1;
            *track = track.update(date.get(tick).unwrap());
            count += 1;
        }
    }

    pub fn iter_anim(&mut self, anim_name: &str) {
        for i in self.date.get_anim_range(anim_name) {
            self.track_update(i);
        }
    }
    pub fn iter_all(&mut self) {
        for i in 0..self.date.len {
            self.track_update(i);
        }
    }
}

// struct VedioTimer {
//     count: f32,
// }
// impl VedioTimer {
//     pub const fn new() -> Self {
//         Self { count: 0f32 }
//     }
//     pub const fn update(&mut self, fps: f32, delta: f32) -> bool {
//         let need_time = 1f32 / fps;
//         if self.count >= need_time {
//             self.count -= need_time;
//             return true;
//         }
//         false
//     }
// }
// pub struct Video {
//     fps: f32,
//     pub play: usize,
//     texs: Vec<TextureWrapper<Texture2D>>,
//     timer: VedioTimer,
// }

// impl Video {
//     const fn new() -> Self {
//         Self {
//             texs: Vec::new(),
//             timer: VedioTimer::new(),
//             fps: 0f32,
//             play: 0,
//         }
//     }

//     pub fn new_path(source: &str) -> Self {

//         let ffmpeg = FfmpegContext::builder().input(Input::from(source)).build()
//         let mut decoder = Decoder::new(source).expect("failed to create decoder");
//         let mut room = Self::new();
//         room.fps = decoder.frame_rate();
//         for frame in decoder.decode_raw_iter() {
//             let frame = frame.unwrap();
//             let (w, h) = (frame.width(), frame.height());
//             let texture = unsafe {
//                 Texture2D::new(
//                     frame.as_ptr() as *const u8,
//                     TextureType::RGB8,
//                     w,
//                     h,
//                     TextureParm,
//                 )
//             };
//             room.texs.push(TextureWrapper(texture));
//         }
//         room
//     }
//     pub const fn update(&mut self, delta: f32) {
//         if self.timer.update(self.fps, delta) {
//             self.play += 1;
//         }
//     }

//     pub fn get(&self) -> &Texture2D {
//         self.texs.get(self.play).unwrap().as_ref()
//     }
// }
