use core::panic;
use std::{ffi::c_void, marker::PhantomData, ops::Deref, ptr::null, sync::LazyLock};

use gl::types::{GLenum, GLuint};

use super::{
    VertexArray,
    define::{BufferTarget, BufferUsage, TypeGL, VertexArrayAttribPointerGen},
};
//do not edit
// down left
pub static TEX_VERTEX_YFLIP_STATIC: LazyLock<BufferConst<f32>> = LazyLock::new(|| {
    BufferConst::new(
        BufferTarget::Vertex,
        &[0f32, 0f32, 1f32, 0f32, 1f32, 1f32, 0f32, 1f32],
        BufferUsage::Static,
    )
});
pub static TEX_VERTEX_STATIC: LazyLock<BufferConst<f32>> = LazyLock::new(|| {
    BufferConst::new(
        BufferTarget::Vertex,
        &[0f32, 1f32, 1f32, 1f32, 1f32, 0f32, 0f32, 0f32],
        BufferUsage::Static,
    )
});

pub static VAO_STATIC: LazyLock<VertexArray> = LazyLock::new(|| {
    let vao = VertexArray::new();
    vao.bind(|vao| {
        vao.bind_pointer(
            VERTEX_MUT.deref(),
            VertexArrayAttribPointerGen::new::<f32>(0, 2),
        );
        vao.bind_pointer(
            TEX_VERTEX_STATIC.deref(),
            VertexArrayAttribPointerGen::new::<f32>(1, 2),
        );
    });
    vao
});

//mutable
pub const VERTEX_BIG: usize = 2 * 4096;
pub static VERTEX_BIG_MUT: LazyLock<BufferConst<f32>> = LazyLock::new(|| {
    BufferConst::new(
        BufferTarget::Vertex,
        &[0f32; VERTEX_BIG],
        BufferUsage::Dynamic,
    )
});
pub static VERTEX_MUT: LazyLock<BufferConst<f32>> = LazyLock::new(|| {
    BufferConst::new(
        BufferTarget::Vertex,
        &[0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32],
        BufferUsage::Dynamic,
    )
});
pub static TEX_VERTEX_MUT: LazyLock<BufferConst<f32>> = LazyLock::new(|| {
    BufferConst::new(
        BufferTarget::Vertex,
        &[0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32],
        BufferUsage::Dynamic,
    )
});
pub static VAO_MUT: LazyLock<VertexArray> = LazyLock::new(|| VertexArray::new());

pub trait Buffer {
    fn type_as_gl(&self) -> GLenum;
    fn target(&self) -> BufferTarget;
    fn id(&self) -> GLuint;
    fn count(&self) -> usize;
    fn bind_target(&self) {
        bind_buffer(self.target(), self.id());
    }
    fn unbind_target(&self) {
        bind_buffer(self.target(), 0);
    }
}
fn bind_buffer(target: BufferTarget, id: GLuint) {
    unsafe {
        gl::BindBuffer(target.as_gl(), id);
    }
}

pub struct BufferObject {
    pub target: BufferTarget,
    id: GLuint,
    len: usize,
    type_const: GLenum,
}
impl Buffer for BufferObject {
    fn target(&self) -> BufferTarget {
        self.target
    }

    fn id(&self) -> GLuint {
        self.id
    }

    fn count(&self) -> usize {
        self.len
    }

    fn type_as_gl(&self) -> GLenum {
        self.type_const
    }
}

pub struct BufferConst<T>
where
    T: TypeGL,
{
    pub target: BufferTarget,
    id: GLuint,
    len: usize,
    type_const: PhantomData<T>,
}

impl<T: TypeGL + 'static> BufferConst<T> {
    pub unsafe fn new_raw(
        target: BufferTarget,
        point: *const c_void,
        len: usize,
        usage: BufferUsage,
    ) -> Self {
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
            bind_buffer(target, id);
            gl::BufferData(
                target.as_gl(),
                (len * size_of::<T>()) as isize,
                point,
                usage.as_gl(),
            );
        }
        bind_buffer(target, 0);
        Self {
            target,
            id,
            len,
            type_const: PhantomData,
        }
    }
    pub fn new(target: BufferTarget, data: &[T], usage: BufferUsage) -> Self {
          unsafe { Self::new_raw(target, data.as_ptr() as *const c_void, data.len(), usage) } 
    }
    pub fn from_iter(
        target: BufferTarget,
        data: impl Iterator<Item = T>,
        usage: BufferUsage,
    ) -> Self {
        let mut vec = Vec::new();
        for var in data {
            vec.push(var);
        }
        Self::new(target, &vec, usage)
    }
    pub fn new_null(target: BufferTarget, len: usize, usage: BufferUsage) -> Self {
        unsafe { Self::new_raw(target, null(), len, usage) }
    }
    pub fn sub_data(&self, data: &[T], offset: usize) {
        if data.len() > self.count() {
            panic!("[sub data err]data's len > buffer");
        }
        self.bind_target();
        unsafe {
            gl::BufferSubData(
                self.target().as_gl(),
                offset as isize,
                std::mem::size_of_val(data) as isize,
                data.as_ptr() as *const c_void,
            );
        }
    }

    pub fn buffer_object(self) -> BufferObject {
        let value = BufferObject {
            target: self.target,
            id: self.id,
            len: self.len,
            type_const: T::as_gl(),
        };
        std::mem::forget(self);
        value
    }
}
impl<T: TypeGL> Buffer for BufferConst<T> {
    fn target(&self) -> BufferTarget {
        self.target
    }

    fn id(&self) -> GLuint {
        self.id
    }

    fn count(&self) -> usize {
        self.len
    }

    fn type_as_gl(&self) -> GLenum {
        T::as_gl()
    }
}
impl<T> Drop for BufferConst<T>
where
    T: TypeGL,
{
    fn drop(&mut self) {
        println!("{:?} buffer:{} leave", self.target, self.id);
        unsafe {
            self.unbind_target();
            gl::DeleteBuffers(1, &self.id as *const GLuint);
        }
    }
}

#[test]
fn size() {
    println!("{}", size_of::<u16>())
}
