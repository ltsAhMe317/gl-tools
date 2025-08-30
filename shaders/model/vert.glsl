#version 330
uniform mat4 joint_mats[24];
layout(location = 0) in vec3 vert;
layout(location = 1) in vec2 auv;
layout(location = 2) in vec3 anormal;

uniform bool is_skin;
layout(location = 3) in vec4 joint;
layout(location = 4) in vec4 weight;

out vec2 uv;
out vec3 normal;
out vec3 frag_pos;
uniform mat4 project_mat;
uniform mat4 model_mat;
uniform mat4 mesh_mat;
void main() {
    uv = auv;
    normal = anormal;
    if (is_skin) {
        mat4 skin_mat = weight.x * joint_mats[int(joint.x)] +
                weight.y * joint_mats[int(joint.y)] +
                weight.z * joint_mats[int(joint.z)] +
                weight.w * joint_mats[int(joint.w)];
        vec4 world_pos = skin_mat * vec4(vert, 1);
        gl_Position = project_mat *  world_pos;
    } else {
        gl_Position = project_mat * model_mat * mesh_mat * vec4(vert, 1);
    }
    frag_pos = gl_Position.xyz;
}
