#version 330 core

uniform sampler2D back_texture;
uniform sampler2D front_texture;

in vec2 v_texcoord;
out vec4 frag_color;

void main() {
    vec4 back_color = texture(back_texture, vec2(v_texcoord.x, 1.0 - v_texcoord.y));
    vec4 front_color = texture(front_texture, vec2(v_texcoord.x, 1.0 - v_texcoord.y));
    
    frag_color = mix(back_color, front_color, front_color.a);
}