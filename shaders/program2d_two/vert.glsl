
 #version 330
    layout (location = 0) in vec2 vert;
    layout (location = 1) in vec2 uv;
    out vec2 tex_uv;
    uniform mat4 project_mat;
    uniform mat4 model_mat;
    void main(){
        gl_Position = project_mat * model_mat * vec4(vert,0,1);
        tex_uv = uv;
    }
