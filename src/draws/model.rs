use std::{fs, path::Path};

use gl::VertexArrayVertexBuffer;
use glam::Mat4;
use gltf::{buffer::Data, Document, Gltf, Node};
use rand::seq::IndexedRandom;

use crate::{
    gl_unit::{
        define::{BufferTarget, BufferUsage, DrawMode, VertexArrayAttribPointerGen},
        program::Program,
        VertexArray,
    },
    Buffer, BufferConst, BufferObject,
};


fn document_mesh(document:&Document,buffers:&Vec<Data>)->Vec<Mesh>{
    let mut vec = Vec::new();
    for node in document.nodes(){
        vec.extend(node_next_mesh(buffers, &node, Mat4::IDENTITY));
    }
    vec
}
fn node_next_mesh(buffers:&Vec<Data>,node:&Node,transfrom:Mat4)->Vec<Mesh>{
    let mut collect_mesh = Vec::new();
            let transfrom_done =  transfrom*Mat4::from_cols_array(&unsafe{std::mem::transmute(node.transform().matrix())});
    
    if let Some(mesh) = node.mesh(){
    let mut datas = Vec::new();
            for primitive in mesh.primitives() {
                
                let mut vertex_list = Vec::new();
                let element;
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                
                if let Some(pos) = reader.read_positions() {
                    for vec in pos {
                        vertex_list.extend(vec);
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
                let mut vertex_array = VertexArray::new();
                vertex_array.bind(|vao|{
                vao.pointer(
                    &vertex_buffer,
                    VertexArrayAttribPointerGen::new::<f32>(0, 3),
                );
                
                });
                vertex_array.element_bind(&element);

                let primitive = PrimitiveData {
                    vbo: vertex_buffer,
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
                };
                datas.push(primitive);
            }
            
            
        collect_mesh.push(Mesh{transfrom:transfrom_done,primitives:datas});
    }else {
         for child in node.children(){
            collect_mesh.extend(node_next_mesh(buffers, &child, transfrom_done));
        }
    }
    collect_mesh
}





pub struct PrimitiveData {
    vbo: BufferConst<f32>,
    vao: VertexArray,
    ebo: BufferObject,
    draw_mode: DrawMode,
}
pub struct Mesh {
    transfrom:Mat4,
    primitives: Vec<PrimitiveData>,
}

pub struct Model {
    mashes: Vec<Mesh>,
}
impl Model {
    pub fn from_path(p:impl AsRef<Path>)->Self{
        Self::from_buffer(fs::read(p).unwrap())
    }
    pub fn from_buffer(buffer: impl AsRef<[u8]>) -> Self {
        let (document, buffers, images) = gltf::import_slice(buffer.as_ref()).unwrap();


        
        Self { mashes: document_mesh(&document, &buffers) }
    }
    pub fn draw(&self, program: &Program) {
        program.bind();
        for mash in self.mashes.iter() {
            program.put_matrix_name(&mash.transfrom, "mesh_mat");
            for primitive in mash.primitives.iter() {
                
                primitive.vao.bind(|vao| {
                    vao.draw_element(primitive.draw_mode, 0, primitive.ebo.len() as i32);
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use glam::{vec3, Mat4};

    use crate::{
        gl_unit::{polygon_mode, program::Program, window::Window, GLcontext},
        ui::font::Font,
    };

    use super::Model;

    #[test]
    pub fn model() {
        let mut window = Window::new(1280, 720, "test model", false);
        let mut context = GLcontext::with(&mut window);
        window.window.show();

        let vert = "
            #version 330
    layout (location = 0) in vec3 vert;
    uniform mat4 project_mat;
    uniform mat4 model_mat;
    uniform mat4 mesh_mat;
    void main(){
        gl_Position = project_mat*model_mat* mesh_mat*vec4(vert,1);
    }
        ";
        let frag = "
            #version 330
    out vec4 color;
    void main(){
        color = vec4(1,1,1,1);
    }
";
        let program = Program::basic_new(vert, frag, None);
        program.bind();

        let mut font = Font::new_file(Path::new("./font.otf"), 0);
        let model = Model::from_path("test.glb");

        println!("loaded");
        while !window.update() {
            context.draw_option(&mut window, |_, window| {
                polygon_mode(
                    crate::gl_unit::define::Face::FrontAndBack,
                    crate::gl_unit::define::PolygonMode::Line(1f32),
                );
                program.bind();
                let (w, h) = window.window.get_size();
                program.put_matrix_name(
                    &(Mat4::perspective_rh_gl(
                        90f32.to_radians(),
                        w as f32 / h as f32,
                        0.01f32,
                        10f32,
                    ) * Mat4::look_to_rh(
                        vec3(0f32, 2f32, -3f32),
                        vec3(0f32, -1f32, 1f32),
                        vec3(0f32, 1f32, 0f32),
                    )),
                    "project_mat",
                );
                program.put_matrix_name(
                    &(Mat4::from_translation(vec3(
                        window.delta_count.time_count.sin() as f32,
                        0f32,
                        0f32,
                    )) * Mat4::from_rotation_y(window.delta_count.time_count as f32)),
                    "model_mat",
                );
                model.draw(&program);
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
