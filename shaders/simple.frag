#version 450 core

out vec4 color;

uniform layout(location=2) vec4 uniform_color_input;

in layout(location=4) vec4 buffer_color;

void main()
{

color = buffer_color;
}