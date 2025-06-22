use core::panic;
use std::{ffi::c_void, marker::PhantomData, sync::LazyLock};

use gl::types::{GLenum, GLuint};

use crate::gl_unit::debug;

use super::{
    define::{BufferTarget, BufferUsage, VertexArrayAttribPointerGen},
    flush, VertexArray,
};
//do not edit
// down left
pub static TEX_VERTEX_YFLIP_STATIC: LazyLock<Buffer<f32>> = LazyLock::new(|| {
    Buffer::new(
        BufferTarget::Vertex,
        &[0f32, 0f32, 1f32, 0f32, 1f32, 1f32, 0f32, 1f32],
        BufferUsage::Static,
    )
});
pub static TEX_VERTEX_STATIC: LazyLock<Buffer<f32>> = LazyLock::new(|| {
    Buffer::new(
        BufferTarget::Vertex,
        &[0f32, 1f32, 1f32, 1f32, 1f32, 0f32, 0f32, 0f32],
        BufferUsage::Static,
    )
});

pub static VAO_STATIC: LazyLock<VertexArray> = LazyLock::new(|| {
    let vao = VertexArray::new();
    vao.bind_set(&*VERTEX_MUT, VertexArrayAttribPointerGen::new::<f32>(0, 2));
    vao.bind_set(
        &*TEX_VERTEX_STATIC,
        VertexArrayAttribPointerGen::new::<f32>(1, 2),
    );
    dbg!("VAO_STATIC LOAD");
    vao
});

//mutable
pub const VERTEX_BIG: usize = 2 * 4096;
pub static VERTEX_BIG_MUT: LazyLock<Buffer<f32>> = LazyLock::new(|| {
    dbg!("VERTEX_BIG_MUT LOAD");

    Buffer::new(
        BufferTarget::Vertex,
        &[0f32; VERTEX_BIG],
        BufferUsage::Dynamic,
    )
});
pub static VERTEX_MUT: LazyLock<Buffer<f32>> = LazyLock::new(|| {
    dbg!("VERTEX_MUT LOAD");

    Buffer::new(
        BufferTarget::Vertex,
        &[0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32],
        BufferUsage::Dynamic,
    )
});
pub static TEX_VERTEX_MUT: LazyLock<Buffer<f32>> = LazyLock::new(|| {
    dbg!("TEX_VERTEX_MUT LOAD");

    Buffer::new(
        BufferTarget::Vertex,
        &[0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32],
        BufferUsage::Dynamic,
    )
});
pub static VAO_MUT: LazyLock<VertexArray> = LazyLock::new(|| {
    dbg!("VAO_STATIC LOAD");

    VertexArray::new()
});

pub trait TypeGL {
    fn as_gl() -> GLenum;
}
impl TypeGL for f32 {
    fn as_gl() -> GLenum {
        gl::FLOAT
    }
}
impl TypeGL for i32 {
    fn as_gl() -> GLenum {
        gl::INT
    }
}
pub struct Buffer<T>
where
    T: TypeGL,
{
    pub target: BufferTarget,
    id: GLuint,
    size: isize,
    type_const: PhantomData<T>,
}
impl<T: TypeGL> Buffer<T> {
    pub unsafe fn new_raw(
        target: BufferTarget,
        point: *const c_void,
        len: isize,
        usage: BufferUsage,
    ) -> Self {
        let mut id = 0;

        gl::GenBuffers(1, &mut id);
        Self::bind(target, id);
        gl::BufferData(
            target.as_gl(),
            len * size_of::<T>() as isize,
            point,
            usage.as_gl(),
        );
        println!("now_size:{}", debug::buffer_size(target));
        println!("data:{:?}", debug::buffer_data(target));
        println!("create:{}", gl::GetError());
        Self::bind(target, 0);
        Self {
            target,
            id,
            size: len,
            type_const: PhantomData,
        }
    }
    pub fn new(target: BufferTarget, data: &[T], usage: BufferUsage) -> Self {
        let (point, len) = (data.as_ptr(), data.len());
        unsafe { Self::new_raw(target, point as *const c_void, len as isize, usage) }
    }
    pub fn sub_data(&self, data: &[T], offset: usize) {
        if data.len() as isize > self.size {
            panic!("[sub data err]data's len > buffer");
        }
        self.bind_target();
        unsafe {
            gl::BufferSubData(
                self.target.as_gl(),
                offset as isize,
                std::mem::size_of_val(data) as isize,
                data.as_ptr() as *const c_void,
            );
        }
    }
    pub fn bind_target(&self) {
        Self::bind(self.target, self.id);
    }
    pub fn unbind_target(&self) {
        Self::bind(self.target, 0);
    }
    fn bind(target: BufferTarget, id: GLuint) {
        unsafe {
            gl::BindBuffer(target.as_gl(), id);
        }
    }
}
impl<T> Drop for Buffer<T>
where
    T: TypeGL,
{
    fn drop(&mut self) {
        println!("i break:(");
        unsafe {
            self.unbind_target();
            gl::DeleteBuffers(1, &self.id as *const GLuint);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gl_unit::{
        define::{BufferTarget, BufferUsage, VertexArrayAttribPointerGen},
        program::Program,
        window::Window,
        GLcontext, VertexArray,
    };

    use super::Buffer;

    #[test]
    fn test_buffer() {
        let mut window = Window::new(800, 600, "test buffer", false);
        let mut gl_context = GLcontext::with(&mut window);
        window.window.show();
        let vert = "
#version 330
    layout (location = 0) in vec2 vert;
    
    void main(){
        gl_Position =  vec4(vert.xy,0,1);
    }
            
        ";
        let frag = "
#version 330
    out vec4 color;    
    void main(){
        color =  vec4(1,1,0,1);
    }
            
            
        ";
        let program = Program::basic_new(vert, frag, None);
        program.bind();
        let buffer = Buffer::new(
            BufferTarget::Vertex,
            &[0f32, 0f32, 0.5f32, 0f32, 0.5f32, 0.5f32, 0f32, 0.5f32],
            BufferUsage::Static,
        );
        let vao = VertexArray::new();

        while !window.update() {
            gl_context.draw_option(&mut window, |context, window| {
                vao.bind_set(&buffer, VertexArrayAttribPointerGen::new::<f32>(0, 2));

                program.draw_rect(1);
            });
        }
    }
}
