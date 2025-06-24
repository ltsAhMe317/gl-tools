use std::path::Path;

use rand::seq::IndexedRandom;

use crate::{
    gl_unit::{
        define::{BufferTarget, BufferUsage, DrawMode, VertexArrayAttribPointerGen},
        program::Program,
        VertexArray,
    },
    Buffer, BufferConst, BufferObject,
};

pub struct Mash {
    pub name: Option<String>,
    vbo: BufferConst<f32>,
    vao: VertexArray,
    ebo: BufferObject,
}

pub struct Model {
    mashes: Vec<Mash>,
}
impl Model {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let model = modelz::Model3D::load(path).unwrap();
        let mut mashes = Vec::new();
        for mash in model.meshes.into_iter() {
            println!("loading mash");
            let buffer = BufferConst::new(
                BufferTarget::Vertex,
                &{
                    let mut vec: Vec<f32> = Vec::with_capacity(mash.vertices.len());
                    for vertex in mash.vertices.iter() {
                        vec.extend_from_slice(&vertex.position);
                    }
                    vec
                },
                BufferUsage::Static,
            );
            let element_buffer = {
                if let Some(indices) = mash.indices {
                    match indices {
                        modelz::Indices::U8(items) => {
                            BufferConst::<u8>::new(BufferTarget::Element, items.as_slice(), BufferUsage::Static)
                                .buffer_object()
                        }
                        modelz::Indices::U16(items) => {

                            BufferConst::<u16>::new(BufferTarget::Element, items.as_slice(), BufferUsage::Static)
                                .buffer_object()
                        }
                        modelz::Indices::U32(items) => {

                            BufferConst::<u32>::new(BufferTarget::Element, items.as_slice(), BufferUsage::Static)
                                .buffer_object()
                        }
                    }
                } else {
                    BufferConst::<u8>::new_null(BufferTarget::Element, 0, BufferUsage::Static)
                        .buffer_object()
                }
            };
            let mut vao = VertexArray::new();
            
            vao.pointer(&buffer, VertexArrayAttribPointerGen::new::<f32>(0, 3));
            vao.element_bind(&element_buffer);
            
            mashes.push(Mash {
                name: mash.name,
                vbo: buffer,
                vao,
                ebo: element_buffer,
            });
        }

        Self { mashes }
    }
    pub fn draw(&self, program: &Program) {
        program.bind();
        for mash in self.mashes.iter() {
            mash.vao.bind(|vao|{
                vao
                .draw_element(DrawMode::Triangles, 0, mash.ebo.len() as i32);
            });
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

    void main(){
        gl_Position = project_mat*model_mat* vec4(vert,1);
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
        let model = Model::new("test.glb");
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
                        70f32.to_radians(),
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
                program.put_matrix_name(&(Mat4::from_translation(vec3(window.delta_count.time_count.sin() as f32, 0f32, 0f32))*Mat4::from_rotation_y(window.delta_count.time_count as f32)), "model_mat");
                model.draw(&program);
                polygon_mode(
                    crate::gl_unit::define::Face::FrontAndBack,
                    crate::gl_unit::define::PolygonMode::Fill,
                );

                font.draw(
                    &format!("fps:{}", window.delta_count.fps() as i32),
                    window.window.get_size(),
                    w as f32/2f32 -300f32,
                    h as f32/2f32-50f32,
                    25,
                    (1f32, 1f32, 1f32, 1f32),
                );
            });
        }
    }
}
