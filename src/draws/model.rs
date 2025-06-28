use core::panic;
use std::{fs, path::Path, sync::LazyLock};

use glam::Mat4;
use gltf::{buffer::Data, Document, Node};

use crate::{
    gl_unit::{
        define::{
            BufferTarget, BufferUsage, DrawMode, TextureParm, TextureType,
            VertexArrayAttribPointerGen,
        },
        program::Program,
        texture::{Texture, Texture2D, TextureWrapper},
        VertexArray,
    },
    Buffer, BufferConst, BufferObject,
};

fn document_mesh(document: &Document, buffers: &Vec<Data>) -> Vec<Mesh> {
    let mut vec = Vec::new();
    for node in document.nodes() {
        vec.extend(node_next_mesh(buffers, &node, Mat4::IDENTITY));
    }
    vec
}
fn node_next_mesh(buffers: &Vec<Data>, node: &Node, transfrom: Mat4) -> Vec<Mesh> {
    let mut collect_mesh = Vec::new();
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

            if let (Some(pos), Some(uv),Some(normal)) = (reader.read_positions(), reader.read_tex_coords(0),reader.read_normals()) {
                for (vec, uv) in pos.into_iter().zip(uv.into_u8().into_iter()) {
                    vertex_list.extend(vec);
                    uv_list.extend(uv);
                }
                for normal in normal{
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
            let normal_buffer = BufferConst::new(BufferTarget::Vertex, &normal_list, BufferUsage::Static);

            let mut vertex_array = VertexArray::new();
            vertex_array.bind(|vao| {
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
                
            });

            vertex_array.element_bind(&element);

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

        collect_mesh.push(Mesh {
            transfrom: transfrom_done,
            primitives: datas,
        });
    } else {
        for child in node.children() {
            collect_mesh.extend(node_next_mesh(buffers, &child, transfrom_done));
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

pub struct Model {
    mashes: Vec<Mesh>,
    texs: Vec<TextureWrapper<Texture2D>>,
}
impl Model {
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
                    gltf::image::Format::R8G8 | gltf::image::Format::R16G16 => panic!("not impl"),
                    gltf::image::Format::R8G8B8 => TextureType::RGB8,
                    gltf::image::Format::R8G8B8A8 => TextureType::RGBA8,
                    gltf::image::Format::R16 => TextureType::RED16,
                    gltf::image::Format::R16G16B16 => TextureType::RGB16,
                    gltf::image::Format::R16G16B16A16 => TextureType::RGBA16,
                    gltf::image::Format::R32G32B32FLOAT => TextureType::RGB32,
                    gltf::image::Format::R32G32B32A32FLOAT => TextureType::RGBA32,
                };

                TextureWrapper(Texture2D::load(
                    Some(data.pixels.as_slice()),
                    tex_type,
                    data.width,
                    data.height,
                    TextureParm::new(),
                ))
            })
            .collect::<Vec<TextureWrapper<Texture2D>>>();

        Self {
            mashes: document_mesh(&document, &buffers),
            texs: textures,
        }
    }
    pub fn draw(&self, program: &Program) {
        program.bind();
        for mash in self.mashes.iter() {
            program.put_matrix_name(&mash.transfrom, "mesh_mat");
            program.put_texture(0, program.get_uniform("material_texture"));
            for primitive in mash.primitives.iter() {
                if let Some(tex_index) = &primitive.material.texture {
                    program.put_bool(program.get_uniform("is_material_texture"), true);
                    self.texs.get(*tex_index).unwrap().bind_unit(0);
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
const MODEL_PROGRAM_VERT:&str = include_str!("../../shaders/model/vert.glsl");
const MODEL_PROGRAM_FRAG:&str = include_str!("../../shaders/model/frag.glsl");
pub static MODEL_PROGRAM:LazyLock<Program> = LazyLock::new(||{
    Program::basic_new(MODEL_PROGRAM_VERT, MODEL_PROGRAM_FRAG, None)
});
#[cfg(test)]
mod tests {
    use std::path::Path;

    use glam::Mat4;
    use glfw::Action;

    use crate::{
        draws::{model::MODEL_PROGRAM, vec_from_rad, Camera, Camera3D}, gl_unit::{depth_test, polygon_mode, program::Program, window::Window, GLcontext}, ui::font::Font
    };

    use super::Model;

    #[test]
    pub fn model() {
        let mut window = Window::new(1280, 720, "test model", false);
        let mut context = GLcontext::with(&mut window);
        window.window.show();
        let mut look:(f32,f32) = (0f32,0f32);
        let mut camera = Camera3D::new(&window);
        let mut font = Font::new_file(Path::new("./font.otf"), 0);
        let model = Model::from_path("test.glb");
        depth_test(true);
        println!("loaded");
        while !window.update() {
            
            let mut look_vec = vec_from_rad( 0f32,look.0.to_radians())*window.delta_count.delta as f32;
            let look_vec_right = vec_from_rad( 0f32,(look.0-90f32).to_radians()) * window.delta_count.delta as f32;
            let delta = window.delta_count.delta as f32;
            if window.window.get_key(glfw::Key::W) == Action::Press{                
                camera.go_vec( look_vec,delta);
            }
if window.window.get_key(glfw::Key::A) == Action::Press{
                camera.go_vec( -look_vec_right,delta);
            }
if window.window.get_key(glfw::Key::D) == Action::Press{
                camera.go_vec( look_vec_right,delta);
            }
if window.window.get_key(glfw::Key::S) == Action::Press{
                camera.go_vec(-look_vec,delta);
            }
            if window.window.get_key(glfw::Key::Space) == Action::Press{
                camera.location.y += window.delta_count.delta as f32;
            }
            let delta = window.delta_count.delta as f32 * 40f32;
if window.window.get_key(glfw::Key::Up) == Action::Press{
                look.1 += delta;
            }
if window.window.get_key(glfw::Key::Down) == Action::Press{
                look.1-= delta;
            }
if window.window.get_key(glfw::Key::Right) == Action::Press{
                look.0 -= delta;
            }
if window.window.get_key(glfw::Key::Left) == Action::Press{
                look.0 += delta;
            }
            look.1 =look.1.max(-80f32).min(80f32);
            look.0 = look.0%360f32;
            camera.look_rad(look.1.to_radians(), look.0.to_radians());
            context.draw_option(&mut window, |_, window| {
                MODEL_PROGRAM.bind();
                let (w, h) = window.window.get_size();
                MODEL_PROGRAM.put_matrix_name(
                    &camera.as_mat(),
                    "project_mat",
                );
                // MODEL_PROGRAM.put_matrix_name(
                //     &(Mat4::from_translation(vec3(
                //         window.delta_count.time_count.sin() as f32,
                //         0f32,
                //         0f32,
                //     )) * Mat4::from_rotation_y(window.delta_count.time_count as f32)),
                //     "model_mat",
                // );
                MODEL_PROGRAM.put_matrix_name(&Mat4::IDENTITY, "model_mat");
                polygon_mode(
                    crate::gl_unit::define::Face::Front,
                    crate::gl_unit::define::PolygonMode::Fill,
                );
                polygon_mode(
                    crate::gl_unit::define::Face::Back,
                    crate::gl_unit::define::PolygonMode::Line(3f32),
                );

                model.draw(&MODEL_PROGRAM);
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
