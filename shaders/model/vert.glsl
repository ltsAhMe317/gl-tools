        #version 330
    layout (location = 0) in vec3 vert;
    layout (location = 1) in vec2 auv;
	layout (location = 2) in vec3 anormal;
    out vec2 uv;
	out vec3 normal;
	out vec3 frag_pos;
    uniform mat4 project_mat;
    uniform mat4 model_mat;
    uniform mat4 mesh_mat;
    void main(){
        uv = auv;
		normal = anormal;
        gl_Position = project_mat*model_mat* mesh_mat*vec4(vert,1);
		frag_pos = gl_Position.xyz;
    }
