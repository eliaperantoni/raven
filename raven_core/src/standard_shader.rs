use crate::shader::{Shader, ShaderComponent, ShaderComponentType};
use crate::Result;

const STANDARD_VERT_SHADER: &'static str = r"
#version 330 core
layout (location = 0) in vec3 pos_in;
layout (location = 1) in vec3 normal_in;
layout (location = 2) in vec2 uv_in;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec3 frag_pos;
out vec3 normal;
out vec2 uv;

void main() {
    gl_Position = projection * view * model * vec4(pos_in, 1.0);

    frag_pos = vec3(model * vec4(pos_in, 1.0));
    normal = transpose(inverse(mat3(model))) * normal_in;
    uv = uv_in;
}
";

const STANDARD_FRAG_SHADER: &'static str = r"
#version 330 core

in vec3 frag_pos;
in vec3 normal;
in vec2 uv;

out vec4 color;

uniform bool useSampler;
uniform sampler2D sampler;

void main() {
    if (useSampler) {
        color = texture(sampler, uv);
    } else {
        color = vec4(0.4, 0.4, 0.4, 1.0);
    }
}
";

pub fn get_standard_shader() -> Result<Shader> {
    Shader::new()
        .with_component(ShaderComponent::new(STANDARD_VERT_SHADER, ShaderComponentType::VERTEX)?)
        .with_component(ShaderComponent::new(STANDARD_FRAG_SHADER, ShaderComponentType::FRAGMENT)?)
        .build()
}
