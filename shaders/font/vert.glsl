    #version 330
    layout (location = 0) in vec4 vert;
    out vec2 uv;
    uniform mat4 project_mat;
    uniform mat4 model_mat;
    void main(){
        gl_Position = project_mat*model_mat * vec4(vert.xy,0,1);
        uv = vert.zw;
    }

