#version 330
    in vec2 uv;
    uniform sampler2D text;
    uniform vec4 text_color;
    out vec4 color;
    void main(){
        vec4 sampled = vec4(1.0, 1.0, 1.0, texture(text,uv).r);

        color = text_color * sampled;
                // color = vec4(1,1,1,1);
    }

