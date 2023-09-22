#version 450 core

in layout(location=0) vec3 position;
in layout(location=4) vec4 color;
uniform layout(location=2) float oscillating_uniform;

out layout(location=4) vec4 out_color;

void main()
{
    out_color = color;
    float var = 0.5f;
    mat4x4 super_special_matrix = {
    {1.0f,0.0f,0.0f,0.0f},
    {0.0f,1.0f,0.0f,oscillating_uniform},
    {0.0f,0.0f,1.0f,0.0f},
    {0.0f,0.0f,0.0f,1.0f}};
    vec4 pos_vec4 = vec4(position, 1.0f);


    gl_Position = super_special_matrix * pos_vec4;


}

