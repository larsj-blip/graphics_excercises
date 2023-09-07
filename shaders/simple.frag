#version 450 core

out vec4 color;
uniform layout(location=2) vec4 color_input;
void main()
{
    color = vec4(0.0f, 0.0f, 0.0f, 0.0f) + color_input;
}