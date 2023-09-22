#version 450 core

in layout(location=0) vec3 position;
in layout(location=4) vec4 color;

out layout(location=4) vec4 out_color;

void main()
{
    out_color = color;
    vec3 inverted_position = vec3(-position[0], -position[1], position[2]);
    gl_Position = vec4(position, 1.0f);
}