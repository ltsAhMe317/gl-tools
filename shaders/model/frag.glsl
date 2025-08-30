    
#version 330 core

// 输入变量
in vec3 frag_pos;
in vec3 normal;
in vec2 uv;

// 输出颜色
out vec4 color_out;

// 材质属性
uniform bool is_material_texture;
uniform vec4 material_color;
uniform sampler2D material_texture;

// 光源属性
uniform vec3 lightPos = vec3(0.0, 50.0, 0.0);  // 光源位置
uniform vec3 lightColor = vec3(1.0, 1.0, 1.0); // 光源颜色
uniform vec3 ambientColor = vec3(0.5, 0.5, 0.5); // 环境光颜色

void main()
{

    // 1. 获取基础材质颜色
    vec4 baseColor;
    if (is_material_texture) {
        baseColor = texture(material_texture, uv);
    } else {
        baseColor = material_color;
    }
    
    // 2. 法线处理
    vec3 norm = normalize(normal);
    
    // 3. 光照方向计算（从表面指向光源）
    vec3 lightDir = normalize(lightPos - frag_pos);
    
    // 4. 漫反射计算
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;
    
    // 5. 环境光计算
    vec3 ambient = ambientColor;
    
    // 6. 最终颜色合成
    // vec3 result = (ambient + diffuse) * baseColor.rgb;
    
    // 7. 输出颜色（保留原始alpha）
    // color_out = vec4(result, baseColor.a);
    color_out = baseColor;
}
