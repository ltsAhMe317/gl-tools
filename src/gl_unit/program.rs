use std::{
    ffi::{c_char, CStr, CString},
    fs,
    path::Path,
    sync::LazyLock,
};

use gl::types::{GLenum, GLint, GLsizei, GLuint};
use glam::{Mat3, Mat4};
use json::JsonValue;

const PROGRAM2D_VERT_TWO: &str = include_str!("../../shaders/program2d_two/vert.glsl");
const PROGRAM2D_FRAG_TWO: &str = include_str!("../../shaders/program2d_two/frag.glsl");

pub static PROGRAM2D_TWO: LazyLock<Program> =
    LazyLock::new(|| Program::basic_new(PROGRAM2D_VERT_TWO, PROGRAM2D_FRAG_TWO, None));

const PROGRAM2D_VERT_ONE: &str = include_str!("../../shaders/program2d_one/vert.glsl");
const PROGRAM2D_FRAG_ONE: &str = include_str!("../../shaders/program2d_one/frag.glsl");

pub static PROGRAM2D_ONE: LazyLock<Program> =
    LazyLock::new(|| Program::basic_new(PROGRAM2D_VERT_ONE, PROGRAM2D_FRAG_ONE, None));

pub struct Shader {
    shader_id: GLuint,
}

impl Shader {
    pub fn load_file(path: &Path, type_: GLenum) -> Self {
        Self::load(&fs::read_to_string(path).unwrap(), type_)
    }
    pub fn load(code: &str, type_: GLenum) -> Self {
        let id;
        let code = std::ffi::CString::new(code).unwrap();
        unsafe {
            id = gl::CreateShader(type_);
            gl::ShaderSource(
                id,
                1,
                &code.as_ptr() as *const *const c_char,
                std::ptr::null(),
            );
            gl::CompileShader(id);
        }
        Shader { shader_id: id }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.shader_id) }
    }
}
#[derive(Clone)]
pub struct Program {
    program_id: GLuint,
    uniform: JsonValue,
}

impl Program {
    pub fn get_uniform(&self, name: &str) -> GLint {
        unsafe {
            let id = gl::GetUniformLocation(self.program_id, CString::new(name).unwrap().as_ptr());
            if id == -1 {
                println!("can't find {} in program", name)
            }
            id
        }
    }
    pub fn load(path: &Path) -> Self {
        let json = json::parse(&fs::read_to_string(path).unwrap()).unwrap();
        let path_str = path.to_str().unwrap();
        let path = &path_str[..path_str.rfind("\\").unwrap() + 1];
        Self::basic_new(
            fs::read_to_string(Path::new(
                &(path.to_string() + (json["vert"].as_str().unwrap()) + ".vert"),
            ))
            .ok()
            .as_ref()
            .unwrap(),
            fs::read_to_string(Path::new(
                &(path.to_string() + json["frag"].as_str().unwrap() + ".frag"),
            ))
            .ok()
            .as_ref()
            .unwrap(),
            Some(json["uniforms"].clone()),
        )
    }
    pub fn basic_new(vert: &str, frag: &str, uniform: Option<JsonValue>) -> Self {
        unsafe {
            let program_id = gl::CreateProgram();

            //vert
            gl::AttachShader(program_id, Shader::load(vert, gl::VERTEX_SHADER).shader_id);

            //frag
            gl::AttachShader(
                program_id,
                Shader::load(frag, gl::FRAGMENT_SHADER).shader_id,
            );

            let mut len: GLsizei = 1;
            let mut code: Vec<c_char> = Vec::with_capacity(1000);
            gl::LinkProgram(program_id);
            gl::GetProgramInfoLog(program_id, 1000, &mut len, code.as_mut_ptr());
            let cstr = CStr::from_ptr(code.as_ptr());
            let str_slice = cstr.to_str().expect("Failed to convert CStr to &str");
            if !str_slice.is_empty() {
                panic!("Program err: {}", str_slice);
            }

            Program {
                program_id,
                uniform: uniform.unwrap_or(JsonValue::Null),
            }
        }
    }
    pub fn put_bool(&self, id: GLint, bool: bool) {
        let num = match bool {
            true => 1,
            false => 0,
        };
        unsafe {
            gl::Uniform1i(id, num);
        }
    }
    
    pub fn put_matrix3(&self, date: &Mat3, id: GLint) {
        unsafe {
            gl::UniformMatrix3fv(id, 1, gl::FALSE, date.as_ref().as_ptr());
        }
    }
    pub fn put_matrix(&self, date: &Mat4, id: GLint) {
        unsafe {
            gl::UniformMatrix4fv(id, 1, gl::FALSE, date.as_ref().as_ptr());
        }
    }
    pub fn put_matrix_name(&self, date: &Mat4, name: &str) {
        self.put_matrix(date, self.get_uniform(name));
    }
    pub fn put_vec4(&self, date: [f32; 4], id: GLint) {
        unsafe { gl::Uniform4f(id, date[0], date[1], date[2], date[3]) }
    }
    pub fn put_one(&self, date: f32, id: GLint) {
        unsafe {
            gl::Uniform1f(id, date);
        }
    }
    pub fn put_vec(&self, date: &[f32], id: GLint) {
        unsafe {
            gl::Uniform1fv(id, date.len() as GLsizei, date.as_ptr());
        }
    }
    pub fn put_f32(&self, name: &str, date: &Vec<f32>) {
        let id = self.get_uniform(name);
        match date.len() {
            1 => self.put_one(date[0], id),

            4 => self.put_vec4([date[0], date[1], date[2], date[3]], id),
            _ => {
                self.put_vec(date, id);
            }
        }
    }
    pub fn put_texture(&self, tex_unit: GLint, id: GLint) {
        unsafe {
            gl::Uniform1i(id, tex_unit);
        }
    }
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program_id);
            // gl::Enable(gl::BLEND);
            // gl::BlendEquation(Self::get_enum(self.blend_func["func"].as_str().unwrap()));
            // let src = Self::get_enum(self.blend_func["srcrgb"].as_str().unwrap());
            // let dst = Self::get_enum(self.blend_func["dstrgb"].as_str().unwrap());
            // gl::BlendFunc(src, dst);
        }
    }
    pub fn load_uniform(&self) {
        self.bind();
        for i in 0..self.uniform.len() {
            let mut temp = self.uniform[i].clone();
            let name = temp["name"].as_str().unwrap().to_string();
            let uniform_type = temp["type"].as_str().unwrap().to_string();
            let value_count = temp["count"].as_usize().unwrap();
            let mut value = Vec::with_capacity(value_count);
            let vec = temp["values"].take();
            for j in 0..value_count {
                if let Some(num) = vec[j].as_f32() {
                    value.push(num);
                }
            }
            unsafe {
                let name_id = self.get_uniform(&name);
                match uniform_type.as_str() {
                    "vec4" => {
                        let vec = [
                            vec[0].as_f32().unwrap(),
                            vec[1].as_f32().unwrap(),
                            vec[2].as_f32().unwrap(),
                            vec[3].as_f32().unwrap(),
                        ];
                        self.put_vec4(vec, name_id)
                    }

                    "float" => self.put_one(value[0], name_id),
                    "matrix4x4" => {
                        self.put_matrix(
                            &Mat4::from_cols_array(&(value.as_slice().try_into().unwrap())),
                            name_id,
                        );
                        // gl::UniformMatrix4fv(name_id, 1, gl::FALSE, value.as_ptr());
                    }
                    "matrix3x3" => {
                        gl::UniformMatrix3fv(name_id, 1, gl::FALSE, value.as_ptr());
                    }
                    "bool" => {
                        let some;
                        if vec[0].as_str().unwrap().eq("true") {
                            some = 1;
                        } else {
                            some = 0;
                        }
                        gl::Uniform1i(name_id, some);
                    }
                    &_ => {}
                }
            }
        }
    }
    fn get_enum(str: &str) -> GLenum {
        match str {
            "add" => gl::FUNC_ADD,
            "sub" => gl::FUNC_SUBTRACT,
            "reverse" => gl::FUNC_REVERSE_SUBTRACT,
            "one" => gl::ONE,
            "zero" => gl::ZERO,
            &_ => gl::ZERO,
        }
    }
}
