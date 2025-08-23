use std::{collections::HashMap, sync::LazyLock};

use glam::{vec3, Mat4, Quat, Vec3, Vec4};
use gltf::{Node, Primitive, animation::util::ReadOutputs};

use crate::{
    Buffer, BufferConst, BufferObject,
    draws::model::Model,
    gl_unit::{
        VertexArray,
        define::{BufferTarget, BufferUsage, DrawMode, VertexArrayAttribPointerGen},
        program::Program,
        texture::Texture,
    },
};

struct Material {
    texture: Option<usize>,
    color: [f32; 4],
}
impl Material {
    pub fn new(material: gltf::Material) -> Self {
        let pbr = material.pbr_metallic_roughness();
        let texture = pbr.base_color_texture().map(|tex| tex.texture().index());
        let color = pbr.base_color_factor();
        Self { texture, color }
    }
}
#[allow(dead_code)]
struct PrimitiveData {
    weight_buffer: Option<BufferConst<f32>>,
    joint_buffer: Option<BufferConst<f32>>,
    vertex_buffer: BufferConst<f32>,
    uv_buffer: BufferConst<f32>,
    vao: VertexArray,
    ebo: BufferConst<u32>,
    draw_mode: DrawMode,
    material: Material,
}
impl PrimitiveData {
    pub fn new(model: &Model, prim: Primitive) -> Self {
        let reader = prim.reader(|index| Some(&model.data[index.index()]));
        const PRIM_CAP: usize = 2048;
        let mut vertex_list: Vec<f32> = Vec::with_capacity(PRIM_CAP);
        let mut uv_list: Vec<f32> = Vec::with_capacity(PRIM_CAP);
        let mut indices_list: Vec<u32> = Vec::with_capacity(PRIM_CAP);
        if let (Some(vertex), Some(uv), Some(indices)) = (
            reader.read_positions(),
            reader.read_tex_coords(0),
            reader.read_indices(),
        ) {
            for vertex in vertex {
                vertex_list.extend_from_slice(&vertex);
            }
            for uv in uv.into_f32() {
                uv_list.extend_from_slice(&uv);
            }
            for index in indices.into_u32() {
                indices_list.push(index);
            }
        }
        let vertex_buffer =
            BufferConst::new(BufferTarget::Vertex, &vertex_list, BufferUsage::Static);
        let uv_buffer = BufferConst::new(BufferTarget::Vertex, &uv_list, BufferUsage::Static);
        let indices_buffer =
            BufferConst::new(BufferTarget::Element, &indices_list, BufferUsage::Static);
        let mut joint_list: Vec<f32> = Vec::with_capacity(PRIM_CAP);
        let mut weight_list: Vec<f32> = Vec::with_capacity(PRIM_CAP);
        let mut joint_buffer = None;
        let mut weight_buffer = None;
        if let (Some(joints), Some(weight)) = (reader.read_joints(0), reader.read_weights(0)) {
            for (joint, weight) in joints
                .into_u16()
                .map(|value| value.map(|value| value as f32))
                .zip(weight.into_f32())
            {
                joint_list.extend_from_slice(&joint);
                weight_list.extend_from_slice(&weight);
            }
            joint_buffer = Some(BufferConst::new(
                BufferTarget::Vertex,
                &joint_list,
                BufferUsage::Static,
            ));
            weight_buffer = Some(BufferConst::new(
                BufferTarget::Vertex,
                &weight_list,
                BufferUsage::Static,
            ));
        }

        let mut vao = VertexArray::new();
        vao.bind(|vao| {
            vao.bind_pointer(
                &vertex_buffer,
                VertexArrayAttribPointerGen::new::<f32>(0, 3),
            );
            vao.bind_pointer(&uv_buffer, VertexArrayAttribPointerGen::new::<f32>(1, 2));
            if let (Some(joint), Some(weight)) = (&joint_buffer, &weight_buffer) {
                vao.bind_pointer(joint, VertexArrayAttribPointerGen::new::<f32>(3, 4));
                vao.bind_pointer(weight, VertexArrayAttribPointerGen::new::<f32>(4, 4));
            }
        });
        vao.element_bind(&indices_buffer);

        Self {
            weight_buffer,
            joint_buffer,
            vertex_buffer,
            uv_buffer,
            vao,
            ebo: indices_buffer,
            draw_mode: DrawMode::from_gl(prim.mode().as_gl_enum()),
            material: Material::new(prim.material()),
        }
    }
}
pub struct Mesh<'a> {
    transfrom: Mat4,
    primitives: Vec<PrimitiveData>,
    joints_mat: Option<Vec<Mat4>>,
    parent: Node<'a>,
}

fn meshes<'a>(node: Node<'a>, model: &'a Model) -> Vec<Mesh<'a>> {
    let mut mesh_list = Vec::new();
    if let Some(mesh) = node.mesh() {
        let mut prim_vec = Vec::new();
        for prim in mesh.primitives() {
            let prim = PrimitiveData::new(model, prim);
            prim_vec.push(prim);
            println!("prim new!");
        }
        mesh_list.push(Mesh {
            transfrom: model.global_mat(&node),
            primitives: prim_vec,
            joints_mat: model.joint_mat(&node),
            parent: node.clone(),
        });
    }
    mesh_list
}

const MODEL_PROGRAM_VERT: &str = include_str!("../../../shaders/model/vert.glsl");
const MODEL_PROGRAM_FRAG: &str = include_str!("../../../shaders/model/frag.glsl");
pub static MODEL_PROGRAM: LazyLock<Program> =
    LazyLock::new(|| Program::basic_new(MODEL_PROGRAM_VERT, MODEL_PROGRAM_FRAG, None));

pub struct Player<'a> {
    meshes: Vec<Mesh<'a>>,
    model: &'a Model,
    time: f32,
}
impl<'a> Player<'a> {
    pub fn new(model: &'a Model) -> Self {
        let mut mesh_list = Vec::new();
        model
            .document
            .nodes()
            .map(|node| {
                mesh_list.extend(meshes(node, model));
            })
            .for_each(drop);
        Self {
            meshes: mesh_list,
            model,
            time: 0f32,
        }
    }
    pub fn draw(&self) {
        let program = &MODEL_PROGRAM;
        program.bind();
        program.put_texture(0, program.get_uniform("material_texture"));
        for mesh in self.meshes.iter() {
            if let Some(joint_mat) = &mesh.joints_mat {
                program.put_matrix_vec(&joint_mat, program.get_uniform("joint_mats"));
            }
            program.put_bool(program.get_uniform("is_skin"), mesh.joints_mat.is_some());
            program.put_matrix_name(mesh.transfrom, "mesh_mat");
            for prim in mesh.primitives.iter() {
                if let Some(texture) = &prim.material.texture {
                    self.model.texs[*texture].bind_unit(0);
                }
                program.put_vec4(prim.material.color, program.get_uniform("material_color"));
                program.put_bool(
                    program.get_uniform("is_material_texture"),
                    prim.material.texture.is_some(),
                );
                prim.vao.bind(|vao| {
                    vao.draw_element(prim.draw_mode, 0, prim.ebo.count() as i32);
                });
            }
        }
    }
    pub fn time(&mut self, value: f32) {
        self.time = value;
    }
    pub fn time_add(&mut self, value: f32) {
        self.time += value;
    }
    pub fn update_animation(&mut self) {
        let change = self.change_all();
        for mesh in self.meshes.iter_mut() {
            mesh.transfrom = self.model.global_mat_change(&mesh.parent, &change);
            mesh.joints_mat = self.model.joint_mat_change(&mesh.parent,&change);
        }
    }

    fn change_all(&self) -> HashMap<usize, Mat4> {
        let mut change = HashMap::new();
        for anim in self.model.document.animations() {
            for channel in anim.channels() {
                
                let target = channel.target();
                let target_id  = target.node().index();
                let reader = channel.reader(|id| Some(&self.model.data[id.index()]));
                let input = reader.read_inputs().unwrap();
                let output_mat =  output_mat(input,reader.read_outputs().unwrap(),self.time);
                *change.entry(target_id).or_insert(output_mat) *= output_mat;
            }
        }
        change
    }
}

fn output_mat(input: gltf::accessor::Iter<f32>, output: ReadOutputs, timer: f32) -> Mat4 {
    let mut start_time = 0f32;
    let mut start_index = 0;
    let mut end_time = 0f32;
    let mut end_index = 0;
    for (count, const_time) in input.enumerate() {
        if timer >= const_time {
            start_time = const_time;
            start_index = count;
        }
        if const_time >= timer {
            end_time = const_time;
            end_index = count;
            break;
        }
    }
    let total = end_time - start_time;
    let asp: (f32, f32) = ((timer - start_time) / total, (end_time - timer) / total);

    match output {
        ReadOutputs::Translations(iter) => {
            let mut start = Vec3::default();
            let mut end = Vec3::default();
            for (count, value) in iter.enumerate() {
                if count == start_index {
                    start = vec3(value[0], value[1], value[2]);
                }
                if count == end_index {
                    end = vec3(value[0], value[1], value[2]);
                    break;
                }
            }

            start *= asp.0;
            end *= asp.1;
            let result = (start + end) / 2f32;
            Mat4::from_translation(result)
        }
        ReadOutputs::Rotations(rotations) => {
            let mut start = Vec4::default();
            let mut end = Vec4::default();
            for (count, value) in rotations.into_f32().enumerate() {
                if count == start_index {
                    start = Vec4::from_array(value);
                }
                if count == end_index {
                    end = Vec4::from_array(value);
                    break;
                }
            }

            start *= asp.0;
            end *= asp.1;
            let result = (start + end) / 2f32;
            Mat4::from_quat(Quat::from_xyzw(result.x, result.y, result.z, result.w))
        }
        ReadOutputs::Scales(iter) => {
            let mut start = Vec3::default();
            let mut end = Vec3::default();
            for (count, value) in iter.enumerate() {
                if count == start_index {
                    start = vec3(value[0], value[1], value[2]);
                }
                if count == end_index {
                    end = vec3(value[0], value[1], value[2]);
                    break;
                }
            }

            start *= asp.0;
            end *= asp.1;
            let result = (start + end) / 2f32;
            Mat4::from_scale(result)
        }
        ReadOutputs::MorphTargetWeights(morph_target_weights) => todo!(),
    }
}
