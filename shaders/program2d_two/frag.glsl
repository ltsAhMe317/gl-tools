#version 330
    in vec2 tex_uv;
    uniform sampler2D image;
    out vec4 color;
    void main(){
        color = texture(image,tex_uv);
    }
