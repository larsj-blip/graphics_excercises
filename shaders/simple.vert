#version 450 core

in vec3 position;

void main()
{
    vec3 inverted_position = vec3(-position[0], -position[1], position[2]);
    gl_Position = vec4(inverted_position, 1.0f);
}