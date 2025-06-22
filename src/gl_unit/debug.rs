use std::ffi::c_void;

use super::define::BufferTarget;

pub fn now_vao_id() -> i32 {
    unsafe {
        let mut value = 0i32;
        gl::GetIntegerv(gl::VERTEX_ARRAY_BINDING, &mut value);
        value
    }
}
pub fn buffer_size(target: BufferTarget) -> i32 {
    unsafe {
        let mut value = 0i32;
        gl::GetBufferParameteriv(target.as_gl(), gl::BUFFER_SIZE, &mut value);
        value
    }
}
pub fn buffer_data(target: BufferTarget) -> Vec<f32> {
    let size = buffer_size(target);
    let mut vec = vec![0f32; size as usize];
    unsafe {
        gl::GetBufferSubData(
            target.as_gl(),
            0,
            size as isize,
            vec.as_mut_ptr() as *mut c_void,
        );
    }
    vec
}
