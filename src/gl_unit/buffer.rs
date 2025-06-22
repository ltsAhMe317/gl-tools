use core::panic;
use std::{ffi::c_void, marker::PhantomData, sync::LazyLock};

use gl::types::{GLenum, GLuint};

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
    vao
});

//mutable
pub const VERTEX_BIG: usize = 2 * 4096;
pub static VERTEX_BIG_MUT: LazyLock<Buffer<f32>> = LazyLock::new(|| {
    Buffer::new(
        BufferTarget::Vertex,
        &[0f32; VERTEX_BIG],
        BufferUsage::Dynamic,
    )
});
pub static VERTEX_MUT: LazyLock<Buffer<f32>> = LazyLock::new(|| {
    Buffer::new(
        BufferTarget::Vertex,
        &[0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32],
        BufferUsage::Dynamic,
    )
});
pub static TEX_VERTEX_MUT: LazyLock<Buffer<f32>> = LazyLock::new(|| {
    Buffer::new(
        BufferTarget::Vertex,
        &[0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32],
        BufferUsage::Dynamic,
    )
});
pub static VAO_MUT: LazyLock<VertexArray> = LazyLock::new(|| VertexArray::new());

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
    size: usize,
    type_const: PhantomData<T>,
}
impl<T: TypeGL> Buffer<T> {
    pub unsafe fn new_raw(
        target: BufferTarget,
        point: *const c_void,
        len: usize,
        usage: BufferUsage,
    ) -> Self {
        let mut id = 0;

        unsafe {
            gl::GenBuffers(1, &mut id as *mut GLuint);
        }

        let this = Self {
            target,
            id,
            size: len,
            type_const: PhantomData::default(),
        };
        this.bind_target();
        unsafe {
            gl::BufferData(
                this.target.as_gl(),
                (len * size_of::<T>()) as isize,
                point,
                usage.as_gl(),
            );
        }
        flush();
        this.unbind_target();
        this
    }
    pub fn new(target: BufferTarget, data: &[T], usage: BufferUsage) -> Self {
        let (point, len) = (data.as_ptr(), data.len());
        unsafe { Self::new_raw(target, point as *const c_void, len, usage) }
    }
    pub fn sub_data(&self, data: &[T], offset: usize) {
        if data.len() > self.size {
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
        unsafe {
            self.unbind_target();
            gl::DeleteBuffers(1, &self.id as *const GLuint);
        }
    }
}
