#version 330
    in vec2 uv;
    uniform sampler2D image;
    out vec4 color;
    void main(){
        color = texture(image,uv);
        // color = vec4(1,1,1,1);
    }
