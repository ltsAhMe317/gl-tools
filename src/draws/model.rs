use std::{collections::HashMap, fs,  path::Path, sync::LazyLock};

use glam::{Mat4, Vec3, vec3};
use gltf::{animation::{util::ReadOutputs,  }, buffer::Data, Animation, Document, Node};

use crate::{
    Buffer, BufferConst, BufferObject,
    gl_unit::{
        VertexArray,
        define::{
            BufferTarget, BufferUsage, DrawMode, TextureParm, TextureType,
            VertexArrayAttribPointerGen,
        },
        program::Program,
        texture::{Texture, Texture2D, TextureWrapper},
    },
};

fn document_mesh(document: &Document, buffers: &Vec<Data>) -> (HashMap<usize, Mesh>, AABB) {
    let mut aabb = AABB::new();
    let mut vec = HashMap::new();
    for node in document.nodes() {
        vec.extend(node_next_mesh(buffers, &node, Mat4::IDENTITY, &mut aabb));
    }
    (vec, aabb)
}

fn node_next_mesh(
    buffers: &Vec<Data>,
    node: &Node,
    transfrom: Mat4,
    aabb: &mut AABB,
) -> HashMap<usize, Mesh> {
    let mut collect_mesh = HashMap::new();
    let transfrom_done = transfrom
        * Mat4::from_cols_array(&unsafe { std::mem::transmute(node.transform().matrix()) });
    if let Some(mesh) = node.mesh() {
        let mut datas = Vec::new();
        for primitive in mesh.primitives() {
            primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_factor();
            let mut vertex_list = Vec::new();
            let mut uv_list = Vec::new();
            let mut normal_list = Vec::new();
            let element;
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            if let (Some(pos), Some(uv), Some(normal)) = (
                reader.read_positions(),
                reader.read_tex_coords(0),
                reader.read_normals(),
            ) {
                for (vec, uv) in pos.into_iter().zip(uv.into_u8().into_iter()) {
                    let xyz = vec3(vec[0], vec[1], vec[2]);
                    let xyz = transfrom_done.transform_point3(xyz);
                    aabb.update(xyz.x, xyz.y, xyz.z);

                    vertex_list.extend(vec);
                    uv_list.extend(uv);
                }
                for normal in normal {
                    normal_list.extend_from_slice(&normal);
                }
            }

            if let Some(indices) = reader.read_indices() {
                element = match indices {
                    gltf::mesh::util::ReadIndices::U8(iter) => {
                        BufferConst::from_iter(BufferTarget::Element, iter, BufferUsage::Static)
                            .buffer_object()
                    }
                    gltf::mesh::util::ReadIndices::U16(iter) => {
                        BufferConst::from_iter(BufferTarget::Element, iter, BufferUsage::Static)
                            .buffer_object()
                    }
                    gltf::mesh::util::ReadIndices::U32(iter) => {
                        BufferConst::from_iter(BufferTarget::Element, iter, BufferUsage::Static)
                            .buffer_object()
                    }
                }
            } else {
                element =
                    BufferConst::<u8>::new_null(BufferTarget::Element, 0, BufferUsage::Static)
                        .buffer_object();
            }
            let vertex_buffer =
                BufferConst::new(BufferTarget::Vertex, &vertex_list, BufferUsage::Static);
            let uv_buffer = BufferConst::new(BufferTarget::Vertex, &uv_list, BufferUsage::Static);
            let normal_buffer =
                BufferConst::new(BufferTarget::Vertex, &normal_list, BufferUsage::Static);

            let mut vertex_array = VertexArray::new();
            vertex_array.bind_mut(|vao| {
                vao.pointer(
                    &vertex_buffer,
                    VertexArrayAttribPointerGen::new::<f32>(0, 3),
                );
                vao.pointer(
                    &uv_buffer,
                    VertexArrayAttribPointerGen::new::<u8>(1, 2).is_normalized(true),
                );

                vao.pointer(
                    &normal_buffer,
                    VertexArrayAttribPointerGen::new::<f32>(2, 3),
                );
                vao.element_bind(&element);
            });

            let primitive = PrimitiveData {
                vertex_buffer,
                uv_buffer,
                vao: vertex_array,
                ebo: element,
                draw_mode: match primitive.mode() {
                    gltf::mesh::Mode::Points => DrawMode::Points,
                    gltf::mesh::Mode::Lines => DrawMode::Lines,
                    gltf::mesh::Mode::LineLoop => DrawMode::LineLoop,
                    gltf::mesh::Mode::LineStrip => DrawMode::LineStrip,
                    gltf::mesh::Mode::Triangles => DrawMode::Triangles,
                    gltf::mesh::Mode::TriangleStrip => DrawMode::TriangleStrip,
                    gltf::mesh::Mode::TriangleFan => DrawMode::TriangleFan,
                },
                material: {
                    let pbr = primitive.material().pbr_metallic_roughness();
                    Material {
                        texture: if let Some(tex) = pbr.base_color_texture() {
                            Some(tex.texture().index())
                        } else {
                            None
                        },
                        color: pbr.base_color_factor(),
                    }
                },
            };
            datas.push(primitive);
        }

        collect_mesh.insert(
            mesh.index(),
            Mesh {
                transfrom: transfrom_done,
                primitives: datas,
            },
        );
    } else {
        for child in node.children() {
            collect_mesh.extend(node_next_mesh(buffers, &child, transfrom_done, aabb));
        }
    }
    collect_mesh
}

struct Material {
    texture: Option<usize>,
    color: [f32; 4],
}
#[allow(dead_code)]
struct PrimitiveData {
    vertex_buffer: BufferConst<f32>,
    uv_buffer: BufferConst<u8>,
    vao: VertexArray,
    ebo: BufferObject,
    draw_mode: DrawMode,
    material: Material,
}
pub struct Mesh {
    transfrom: Mat4,
    primitives: Vec<PrimitiveData>,
}
#[derive(Clone, Copy, Debug)]
pub struct AABB {
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,
}
impl AABB {
    pub fn new() -> Self {
        Self {
            min_x: 0f32,
            min_y: 0f32,
            min_z: 0f32,
            max_x: 0f32,
            max_y: 0f32,
            max_z: 0f32,
        }
    }
    pub fn new_vec(min: Vec3, max: Vec3) -> Self {
        Self {
            min_x: min.x,
            min_y: min.y,
            min_z: min.z,
            max_x: max.x,
            max_y: max.y,
            max_z: max.z,
        }
    }
    pub fn min_pos(&self) -> Vec3 {
        vec3(self.min_x, self.min_y, self.min_z)
    }
    pub fn max_pos(&self) -> Vec3 {
        vec3(self.max_x, self.max_y, self.max_z)
    }
    pub fn transfrom(&self, mat: Mat4) -> Self {
        let min = self.min_pos();
        let max = self.max_pos();
        Self::new_vec(mat.transform_point3(min), mat.transform_point3(max))
    }
    fn update(&mut self, x: f32, y: f32, z: f32) {
        if x > self.max_x {
            self.max_x = x;
        }
        if y > self.max_y {
            self.max_y = y;
        }
        if z > self.max_z {
            self.max_z = z;
        }

        if x < self.min_x {
            self.min_x = x;
        }
        if y < self.min_y {
            self.min_y = y;
        }
        if z < self.min_z {
            self.min_z = z;
        }
    }
    pub fn as_vertexs(&self) -> [f32; 3 * 4 * 6] {
        let (min_x, min_y, min_z, max_x, max_y, max_z) = (
            self.min_x, self.min_y, self.min_z, self.max_x, self.max_y, self.max_z,
        );
        //痛苦面具
        //手打24个顶点66这是左边的      //2                  //3                     //4
        [
            min_x, min_y, min_z, min_x, max_y, min_z, min_x, max_y, max_z, min_x, min_y, max_z,
            //右边
            max_x, min_y, min_z, max_x, max_y, min_z, max_x, max_y, max_z, max_x, min_y, max_z,
            //上
            min_x, max_y, min_z, max_x, max_y, min_z, max_x, max_y, max_z, min_x, max_y, max_z,
            //下
            min_x, min_y, min_z, max_x, min_y, min_z, max_x, min_y, max_z, min_x, min_y, max_z,
            //后
            min_x, max_y, min_z, max_x, max_y, min_z, max_x, min_y, min_z, min_x, min_y, min_z,
            //前
            min_x, max_y, max_z, max_x, max_y, max_z, max_x, min_y, max_z, min_x, min_y, max_z,
        ]
    }
    pub fn is_touch(&self, other: &Self) -> bool {
        let x_touch = self.max_x > other.min_x && other.max_x > self.min_x;
        let y_touch = self.max_y > other.min_y && other.max_y > self.min_y;
        let z_touch = self.max_z > other.min_z && other.max_z > self.min_z;
        x_touch && y_touch && z_touch
    }
}
fn animation_update(
    last_mat: Mat4,
    mesh: &mut HashMap<usize, Mesh>,
    node: &Node,
    mut offset: HashMap<usize, Mat4>,
) {
    
            let mat = if let Some(mat) = offset.remove(&node.index()) {
                    mat
            }else{
                    let mat = node.transform().matrix();
                    Mat4::from_cols_array_2d(&mat)
            } * last_mat;
                
            for mesh_check in node.mesh() {
                    for (mesh_index, mesh) in mesh.iter_mut() {
                        if *mesh_index == mesh_check.index() {
                            mesh.transfrom = mat;
                            break;
                        }
                    }
            }
            for child in node.children().map(|child|{child as Node}){
                    animation_update(mat, mesh,&child, offset.clone());
            }
}



pub enum PlayMode{
    Normal,Repeat
}
pub struct ModelPlayer<'a> {
    pub aabb: AABB,
    mashes: HashMap<usize, Mesh>,
    time: f32,
    animation: Option<Animation<'a>>,
    pub play_mode:PlayMode
}

impl<'a> ModelPlayer<'a> {
    fn new( document: &'a Document, buffers: &Vec<Data>) -> Self {
        let animation = {
            
               if let Some(anim) = document
                    .animations()
                    // .filter(|animation| animation.name().unwrap().eq(anim_name))
                    .next(){
                Some(anim)
                }else{

                None}
                    
            
        };
        let (mashes, aabb) = document_mesh(document, buffers);
        Self {
            aabb,
            mashes,
            time: 0f32,
            animation: animation,
            play_mode: PlayMode::Repeat,
        }
    }
    pub fn time(&mut self, value: f32) {
        self.time = value;
    }
    pub fn time_add(&mut self,value:f32){
        self.time+=value;
    }
    pub fn load(&mut self, model: &Model) {
        if let Some(anim) = &self.animation {
            let mut offset = HashMap::new();
            for channel in anim.channels() {
                let target = channel.target();
                let reader =channel.reader(|buffer|{Some(&model.data[buffer.index()])});
                let time_list = reader.read_inputs().unwrap();
                let value_list =reader.read_outputs().unwrap();

                let mut start_time =0f32;
                let mut start_index=0;
                let mut end_time =0f32;
                let mut end_index=0;                
                for (count,const_time) in time_list.enumerate(){
                    if self.time>=const_time{
                        start_time = const_time;
                        start_index = count;
                    }
                    if const_time>=self.time{
                        end_time = const_time;
                        end_index = count;
                        break;
                    }
                }
                if start_time>end_time{
                    match self.play_mode{
                        PlayMode::Normal => {return;},
                        PlayMode::Repeat => {self.time=0f32;return;},
                    }
                }

                let total = end_time-start_time;
                let asp:(f32,f32) = ((self.time-start_time)/total,(end_time-self.time)/total); 

                               let mat=match value_list{
                    ReadOutputs::Translations(iter) => {
                        let mut start = Vec3::default();
                        let mut end = Vec3::default();
                        for (count,value) in iter.enumerate(){
                            if count == start_index{
                                start = vec3(value[0], value[1], value[2]);
                            }
                            if count == end_index{
                                end = vec3(value[0], value[1], value[2]);
                                break;
                            }
                        }
                        
                        start *= asp.0;
                        end *= asp.1;
                        let result = (start+end)/2f32;
                        Mat4::from_translation(result)
                    },
                    ReadOutputs::Rotations(rotations) => {
                        todo!()    
                    },
                    ReadOutputs::Scales(iter) => {
                        let mut start = Vec3::default();
                        let mut end = Vec3::default();
                        for (count,value) in iter.enumerate(){
                            if count == start_index{
                                start = vec3(value[0], value[1], value[2]);
                            }
                            if count == end_index{
                                end = vec3(value[0], value[1], value[2]);
                                break;
                            }
                        }
                        
                        start *= asp.0;
                        end *= asp.1;
                        let result = (start+end)/2f32;
                        Mat4::from_scale(result)
                    },
                    ReadOutputs::MorphTargetWeights(morph_target_weights) => todo!(),
                };         
                
                offset.insert(target.node().index(), mat);
            }
            animation_update(Mat4::IDENTITY, &mut self.mashes, &model.document.nodes().next().unwrap(),offset);
        }
    }
    pub fn draw(&self, model: &Model, program: &Program) {
        program.bind();
        for (_, mash) in self.mashes.iter() {
            program.put_matrix_name(mash.transfrom, "mesh_mat");
            program.put_texture(0, program.get_uniform("material_texture"));
            for primitive in mash.primitives.iter() {
                if let Some(tex_index) = &primitive.material.texture {
                    program.put_bool(program.get_uniform("is_material_texture"), true);
                    model.texs.get(*tex_index).unwrap().bind_unit(0);
                } else {
                    program.put_vec4(
                        primitive.material.color,
                        program.get_uniform("material_color"),
                    );
                    program.put_bool(program.get_uniform("is_material_texture"), false);
                }
                primitive.vao.bind(|vao| {
                    vao.draw_element(primitive.draw_mode, 0, primitive.ebo.len() as i32);
                });
            }
        }
    }
}
pub struct Model {
    data: Vec<Data>,
    document: Document,
    texs: Vec<TextureWrapper<Texture2D>>,
}

impl Model {
    pub fn player(&self) -> ModelPlayer<'_> {
        ModelPlayer::new(&self.document, &self.data)
    }
    pub fn from_path(p: impl AsRef<Path>) -> Self {
        Self::from_buffer(fs::read(p).unwrap())
    }
    pub fn from_buffer(buffer: impl AsRef<[u8]>) -> Self {
        let (document, buffers, images) = gltf::import_slice(buffer.as_ref()).unwrap();
        let textures = images
            .into_iter()
            .map(|data| {
                let tex_type = match data.format {
                    gltf::image::Format::R8 => TextureType::RED8,
                    gltf::image::Format::R8G8 | gltf::image::Format::R16G16 => todo!(),
                    gltf::image::Format::R8G8B8 => TextureType::RGB8,
                    gltf::image::Format::R8G8B8A8 => TextureType::RGBA8,
                    gltf::image::Format::R16 => TextureType::RED16,
                    gltf::image::Format::R16G16B16 => TextureType::RGB16,
                    gltf::image::Format::R16G16B16A16 => TextureType::RGBA16,
                    gltf::image::Format::R32G32B32FLOAT => TextureType::RGB32,
                    gltf::image::Format::R32G32B32A32FLOAT => TextureType::RGBA32,
                };
                dbg!(data.format);
                TextureWrapper(Texture2D::load(
                    Some(data.pixels.as_slice()),
                    tex_type,
                    data.width,
                    data.height,
                    if tex_type.eq(&TextureType::RGB8){TextureParm::new().once_load_size(1)}else{ TextureParm::new()},
                ))
            })
            .collect::<Vec<TextureWrapper<Texture2D>>>();
        Self {
            texs: textures,
            document,
            data: buffers,
        }
    }
}
const MODEL_PROGRAM_VERT: &str = include_str!("../../shaders/model/vert.glsl");
const MODEL_PROGRAM_FRAG: &str = include_str!("../../shaders/model/frag.glsl");
pub static MODEL_PROGRAM: LazyLock<Program> =
    LazyLock::new(|| Program::basic_new(MODEL_PROGRAM_VERT, MODEL_PROGRAM_FRAG, None));
#[cfg(test)]
mod tests {
    use std::path::Path;

    use glam::Mat4;
    use glfw::Action;

    use crate::{
        draws::{Camera, Camera3D, model::MODEL_PROGRAM, vec_from_rad},
        gl_unit::{GLcontext, depth_test, polygon_mode, window::Window},
        ui::font::Font,
    };

    use super::Model;

    #[test]
    pub fn model() {
        let mut window = Window::new(1280, 720, "test model", false);
        let mut context = GLcontext::with(&mut window);
        window.window.show();
        let mut look: (f32, f32) = (0f32, 0f32);
        let mut camera = Camera3D::new(&window);
        let mut font = Font::new_file(Path::new("./font.otf"), 0);
        let model_data = Model::from_path("cao.glb");
        let model = model_data.player();
        depth_test(true);
        println!("loaded");
        while !window.update() {
            let look_vec =
                vec_from_rad(0f32, look.0.to_radians()) * window.delta_count.delta as f32;
            let look_vec_yaw =
                vec_from_rad(0f32, (look.0 - 90f32).to_radians()) * window.delta_count.delta as f32;
            let delta = window.delta_count.delta as f32;
            if window.window.get_key(glfw::Key::W) == Action::Press {
                camera.go_vec(look_vec, delta);
            }
            if window.window.get_key(glfw::Key::A) == Action::Press {
                camera.go_vec(-look_vec_yaw, delta);
            }
            if window.window.get_key(glfw::Key::D) == Action::Press {
                camera.go_vec(look_vec_yaw, delta);
            }
            if window.window.get_key(glfw::Key::S) == Action::Press {
                camera.go_vec(-look_vec, delta);
            }
            if window.window.get_key(glfw::Key::Space) == Action::Press {
                camera.location.y += window.delta_count.delta as f32;
            }
            let delta = window.delta_count.delta as f32 * 40f32;
            if window.window.get_key(glfw::Key::Up) == Action::Press {
                look.1 += delta;
            }
            if window.window.get_key(glfw::Key::Down) == Action::Press {
                look.1 -= delta;
            }
            if window.window.get_key(glfw::Key::Right) == Action::Press {
                look.0 -= delta;
            }
            if window.window.get_key(glfw::Key::Left) == Action::Press {
                look.0 += delta;
            }
            look.1 = look.1.max(-80f32).min(80f32);
            look.0 = look.0 % 360f32;
            camera.look_rad(look.1.to_radians(), look.0.to_radians());
            context.draw_option(&mut window, |_, window| {
                MODEL_PROGRAM.bind();
                let (w, h) = window.window.get_size();
                MODEL_PROGRAM.put_matrix_name(camera.as_mat(), "project_mat");
                MODEL_PROGRAM.put_matrix_name(Mat4::IDENTITY, "model_mat");
                polygon_mode(
                    crate::gl_unit::define::Face::Front,
                    crate::gl_unit::define::PolygonMode::Fill,
                );
                polygon_mode(
                    crate::gl_unit::define::Face::Back,
                    crate::gl_unit::define::PolygonMode::Line(3f32),
                );

                model.draw(&model_data,&MODEL_PROGRAM);
                // polygon_mode(
                //     crate::gl_unit::define::Face::FrontAndBack,
                //     crate::gl_unit::define::PolygonMode::Line(1f32),
                // );
                // model.draw(&program);
                polygon_mode(
                    crate::gl_unit::define::Face::FrontAndBack,
                    crate::gl_unit::define::PolygonMode::Fill,
                );

                font.draw(
                    &format!("fps:{}", window.delta_count.fps() as i32),
                    window.window.get_size(),
                    w as f32 / 2f32 - 300f32,
                    h as f32 / 2f32 - 50f32,
                    25,
                    (1f32, 1f32, 1f32, 1f32),
                );
            });
        }
    }
}
