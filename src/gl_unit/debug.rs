
pub fn check_vao_state() {
    unsafe {
        // 检查当前绑定的 VAO
        let mut current_vao: gl::types::GLint = 0;
        gl::GetIntegerv(gl::VERTEX_ARRAY_BINDING, &mut current_vao);
        println!("\n--- VAO 状态检查 ---");
        println!("当前绑定的 VAO: {}", current_vao);

        if current_vao == 0 {
            println!("警告: 当前没有绑定任何 VAO");
            return;
        }

        // 检查 EBO 绑定状态
        let mut current_ebo: gl::types::GLint = 0;
        gl::GetIntegerv(gl::ELEMENT_ARRAY_BUFFER_BINDING, &mut current_ebo);
        println!("当前绑定的 EBO: {}", current_ebo);

        // 检查顶点属性绑定的 VBO
        let mut buffer_binding: gl::types::GLint = 0;
        gl::GetVertexAttribiv(
            0,  // 属性索引 0
            gl::VERTEX_ATTRIB_ARRAY_BUFFER_BINDING,
            &mut buffer_binding
        );
        println!("属性 0 绑定的 VBO: {}", buffer_binding);

        // 检查顶点属性是否启用
        let mut enabled: gl::types::GLint = 0;
        gl::GetVertexAttribiv(0, gl::VERTEX_ATTRIB_ARRAY_ENABLED, &mut enabled);
        println!("属性 0 是否启用: {}", enabled != 0);
    }
}
