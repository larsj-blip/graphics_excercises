// Uncomment these following global attributes to silence most warnings of "low" interest:
/*
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
*/
extern crate nalgebra_glm as glm;

use rand::prelude::*;
use std::{mem, ptr, os::raw::c_void};
use std::thread;
use std::sync::{Mutex, Arc, RwLock};
use gl::types::{GLfloat, GLsizei};

mod shader;
mod util;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;
use crate::shader::Shader;

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  pointer_to_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

fn raw_pointer_to_mutable_array<T>(val: &[T]) -> *mut GLfloat {
    &val[0] as *const T as *mut gl::types::GLfloat
}

// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()


const VERTEX_VAO_INDEX: gl::types::GLuint = 0;

const COLOR_VAO_INDEX: gl::types::GLuint = 4;

const INDEX_UNIFORM_FRAGMENT_SHADER_COLOR: gl::types::GLint = 2;


// == // Generate your VAO here
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>, colors: &Vec<f32>) -> u32 {
    let infer_stride_from_rank_and_datatype: gl::types::GLsizei = 0;
    let create_single_object: gl::types::GLsizei = 1;
    const IGNORED_INITIAL_VALUE: u32 = 0;
    // * Generate a VAO and bind it

    let ignored_initial_value = 0;
    let mut vertex_array_object_id: u32 = ignored_initial_value;
    gl::GenVertexArrays(create_single_object, &mut vertex_array_object_id);
    gl::BindVertexArray(vertex_array_object_id);

    // * Generate a VBO and bind it
    let mut vertex_buffer_id: u32 = ignored_initial_value;
    gl::GenBuffers(create_single_object, &mut vertex_buffer_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_id);

    // * Fill it with data
    let size_of_vertex_array_in_bytes = byte_size_of_array(&vertices[..]);
    gl::BufferData(gl::ARRAY_BUFFER,
                   size_of_vertex_array_in_bytes,
                   pointer_to_array(&vertices[..]),
                   gl::STATIC_DRAW,
    );

    let rank_of_vertex_array = 3;
    let stride = infer_stride_from_rank_and_datatype;
    gl::VertexAttribPointer(
        VERTEX_VAO_INDEX,
        rank_of_vertex_array,
        gl::FLOAT,
        gl::FALSE,
        stride,
        ptr::null(),
    );

    gl::EnableVertexAttribArray(VERTEX_VAO_INDEX);

    // * Generate a IBO and bind it

    let mut color_buffer_object_id = ignored_initial_value;

    gl::GenBuffers(create_single_object, &mut color_buffer_object_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, color_buffer_object_id);
    // * Fill it with data

    let size_of_color_vector_in_bytes = byte_size_of_array(&colors);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        size_of_color_vector_in_bytes,
        pointer_to_array(&&colors[..]), gl::STATIC_DRAW,
    );

    let rank_of_color_array = 4;
    let stride = infer_stride_from_rank_and_datatype;
    gl::VertexAttribPointer(
        COLOR_VAO_INDEX,
        rank_of_color_array,
        gl::FLOAT,
        gl::FALSE,
        stride,
        ptr::null(),
    );
    gl::EnableVertexAttribArray(COLOR_VAO_INDEX);

    // * Generate a IBO and bind it

    let mut index_buffer_object_id = 0;
    gl::GenBuffers(create_single_object, &mut index_buffer_object_id);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer_object_id);
    // * Fill it with data

    let size_of_index_vector_in_bytes = byte_size_of_array(&indices);
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        size_of_index_vector_in_bytes,
        pointer_to_array(&&indices[..]), gl::STATIC_DRAW,
    );
    // * Return the ID of the VAO
    return vertex_array_object_id;
}


fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(INITIAL_SCREEN_W, INITIAL_SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Set up shared tuple for tracking changes to the window size
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers.
        // This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        // == // Set up your VAO around here

        let vertices = vec![
                            -0.8, -0.8, 1.2,
                            0.8, 0.8, -1.2,
                            0.0, 0.4, 0.0,
        ];
        let triangles = vec![0, 1, 2,
        ];
        let mut colors :Vec<f32> = Vec::new();
        let amount_of_vertices = vertices.len()/3;
        for _ in 0..amount_of_vertices{
            let mut color_vector = create_color_vector();
            colors.append(&mut color_vector);
        }
        let vao_1 = unsafe {
            create_vao(&vertices, &triangles, &colors)
        };

        // == // Set up your shaders here

        // Basic usage of shader helper:
        // The example code below creates a 'shader' object.
        // It which contains the field `.program_id` and the method `.activate()`.
        // The `.` in the path is relative to `Cargo.toml`.
        // This snippet is not enough to do the exercise, and will need to be modified (outside
        // of just using the correct path), but it only needs to be called once


        let path_to_fragment_shader = "./shaders/simple.frag";
        let path_to_vertex_shader = "./shaders/simple.vert";
        let shader_program: Shader =
            unsafe {
                let program = shader::ShaderBuilder::new()
                    .attach_file(path_to_vertex_shader)
                    .attach_file(path_to_fragment_shader)
                    .link();

                program.activate();
                program
            };

        let initial_uniform = unsafe {
            gl::Uniform4f(INDEX_UNIFORM_FRAGMENT_SHADER_COLOR, 0.0, 0.0, 0.0, 0.0);
        };

        // Used to demonstrate keyboard handling for exercise 2.
        let mut _arbitrary_number = 0.0; // feel free to remove


        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut previous_frame_time = first_frame_time;
        loop {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
            previous_frame_time = now;

            // Handle resize events
            if let Ok(mut new_size) = window_size.lock() {
                if new_size.2 {
                    context.resize(glutin::dpi::PhysicalSize::new(new_size.0, new_size.1));
                    window_aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
                    (*new_size).2 = false;
                    println!("Window was resized to {}x{}", new_size.0, new_size.1);
                    unsafe { gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32); }
                }
            }

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        // The `VirtualKeyCode` enum is defined here:
                        //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html

                        VirtualKeyCode::A => {
                            _arbitrary_number += delta_time;
                        }
                        VirtualKeyCode::D => {
                            _arbitrary_number -= delta_time;
                        }


                        // default handler:
                        _ => {}
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {

                // == // Optionally access the accumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }

            // == // Please compute camera transforms here (exercise 2 & 3)


            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky, full opacity
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);


                // == // Issue the necessary gl:: commands to draw your scene here
                gl::BindVertexArray(vao_1);
                let size_of_indices_vector = triangles.len() as gl::types::GLsizei;
                gl::DrawElements(
                    gl::TRIANGLES,
                    size_of_indices_vector,
                    gl::UNSIGNED_INT,
                    0 as *const _,
                );
            }


            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
        }
    });


    // == //
    // == // From here on down there are only internals.
    // == //


    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events are initially handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::Resized(physical_size), .. } => {
                println!("New window size received: {}x{}", physical_size.width, physical_size.height);
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, ..
                }, ..
            } => {
                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        }
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => { *control_flow = ControlFlow::Exit; }
                    Q => { *control_flow = ControlFlow::Exit; }
                    _ => {}
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => {}
        }
    });
}

fn create_color_vector() -> Vec<f32> {
    let mut iterations = 0;
    let mut color_vector: Vec<f32> = Vec::new();
    let length_of_color_vector = 4;
    while iterations < length_of_color_vector {
        let entry: f32 = rand::thread_rng().gen_range(0..10) as f32 / 10.0;
        color_vector.push(entry);
        iterations += 1;
    }
    color_vector[3] = 1.0;
    return color_vector
}
fn update_colors(current_color: &[f32; 4]) -> [f32; 4] {
    let new_color_x = change_color_by_weighted_amount(current_color[0], 0.01);
    let new_color_y = change_color_by_weighted_amount(current_color[1], 0.005);
    let new_color_z = change_color_by_weighted_amount(current_color[2], 0.0025);
    let new_color_w = change_color_by_weighted_amount(current_color[3], 0.00125);

    let new_color_array: [f32; 4] =  [new_color_x, new_color_y, new_color_z, new_color_w];
    return new_color_array;
}

fn change_color_by_weighted_amount(color_to_be_updated: f32, weight: f32) -> f32 {
    let mut intermediate_value = color_to_be_updated + weight;
    if intermediate_value >= 1.0 {
        return 0.0;
    }
    return intermediate_value;
}

