#version 460
    layout (location = 0) in vec2 vert;
    uniform mat4 project;
    void main(){
        gl_Position = project*vec4(vert,0,1);
    }    

