use std::{ops::Deref, sync::LazyLock};

use super::{VertexArray, VertexBuffer};

//do not edit
// down left
pub static TEX_VERTEX_YFLIP_STATIC: LazyLock<VertexBuffer<f32>> = LazyLock::new(|| {
    VertexBuffer::new_array(
        &[0f32, 0f32, 1f32, 0f32, 1f32, 1f32, 0f32, 1f32],
        gl::STATIC_DRAW,
    )
});
pub static TEX_VERTEX_STATIC: LazyLock<VertexBuffer<f32>> = LazyLock::new(|| {
    VertexBuffer::new_array(
        &[0f32, 1f32, 1f32, 1f32, 1f32, 0f32, 0f32, 0f32],
        gl::STATIC_DRAW,
    )
});

pub static VAO_STATIC: LazyLock<VertexArray> = LazyLock::new(|| {
    let vao = VertexArray::new();
    vao.with(&*VERTEX_MUT, 0, 2, gl::FLOAT, 0);
    vao.with(&*TEX_VERTEX_STATIC, 1, 2, gl::FLOAT, 0);
    vao
});

//mutable
pub const VERTEX_BIG: usize = 2 * 4096;
pub static VERTEX_BIG_MUT: LazyLock<VertexBuffer<f32>> =
    LazyLock::new(|| VertexBuffer::new_array(&[0f32; VERTEX_BIG], gl::DYNAMIC_DRAW));
pub static VERTEX_MUT: LazyLock<VertexBuffer<f32>> = LazyLock::new(|| {
    VertexBuffer::new_array(
        &[0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32],
        gl::DYNAMIC_DRAW,
    )
});
pub static TEX_VERTEX_MUT: LazyLock<VertexBuffer<f32>> = LazyLock::new(|| {
    VertexBuffer::new_array(
        &[0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32],
        gl::DYNAMIC_DRAW,
    )
});
pub static VAO_MUT: LazyLock<VertexArray> = LazyLock::new(|| VertexArray::new());
