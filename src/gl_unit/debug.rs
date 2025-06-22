pub fn now_vao_id() -> i32 {
    unsafe {
        let mut value = 0i32;
        gl::GetIntegerv(gl::VERTEX_ARRAY_BINDING, &mut value as *mut i32);
        value
    }
}
