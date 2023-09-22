#version 450 core

in layout(location=0) vec3 position;
in layout(location=4) vec4 color;
uniform layout(location=2) mat4x4 transformation_matrix;

out layout(location=4) vec4 out_color;

void main()
{
    out_color = color;
    float var = 0.5f;

    vec4 pos_vec4 = vec4(position, 1.0f);


    gl_Position = transformation_matrix * pos_vec4;


}

