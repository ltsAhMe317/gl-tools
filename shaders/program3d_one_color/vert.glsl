    #version 330
    layout (location = 0) in vec3 vert;
    uniform mat4 project_mat;
    uniform mat4 model_mat;
    void main(){    
        gl_Position = project_mat * model_mat * vec4(vert,1);
    }

