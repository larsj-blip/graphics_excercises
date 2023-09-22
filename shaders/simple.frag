#version 450 core

out vec4 color;


in layout(location=4) vec4 buffer_color;

void main()
{

color = buffer_color;
}