use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext as GL, 
    window, AngleInstancedArrays, KeyboardEvent,
    EventTarget, MouseEvent, WebGlBuffer, WebGlProgram,
    WebGlUniformLocation,
};
use yew::html::Scope;
use yew::{html, Component, Context, Html, NodeRef};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use cgmath::prelude::*;
use cgmath::Rad;

use std::cell::RefCell;
use std::rc::Rc;
use std::time;
use std::time::{Duration, Instant};

use gloo_console::log;
use std::f32::consts::PI;

const AMORTIZATION: f32 = 0.95;

pub enum Msg {}

pub struct GameTwo {
    gl: Option<Rc<GL>>,
    node_ref: NodeRef,
    canvas_width: i32,
    canvas_height: i32,
}

impl Component for GameTwo {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            gl: None,
            node_ref: NodeRef::default(),
            canvas_width: 2000,
            canvas_height: 2000,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <canvas width=2000 height=2000 ref={self.node_ref.clone()} />
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        self.render_gl(ctx.link());
    }
}

impl GameTwo {
    fn request_animation_frame(f: &Closure<dyn FnMut()>) {
        window().unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())
            .expect("should register `requestAnimationFrame` OK");
    }

    fn render_gl(&mut self, _link: &Scope<Self>) {
        let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        let gl: GL = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<GL>()
            .unwrap();
        
        let gl = Some(Rc::new(gl));
        let gl = gl.as_ref().expect("Error: GL Context not initialized.");

        let vehicle_100_vert_code = include_str!("../shaders/vehicle_100.vert");
        let torpedo_100_vert_code = include_str!("../shaders/torpedo_100.vert");

        let vehicle_100_vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
        let torpedo_100_vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
        
        gl.shader_source(&vehicle_100_vert_shader, vehicle_100_vert_code);
        gl.shader_source(&torpedo_100_vert_shader, torpedo_100_vert_code);

        gl.compile_shader(&vehicle_100_vert_shader);
        gl.compile_shader(&torpedo_100_vert_shader);

        let vehicle_100_vert_shader_log = gl.get_shader_info_log(&vehicle_100_vert_shader);
        let torpedo_100_vert_shader_log = gl.get_shader_info_log(&torpedo_100_vert_shader);
        
        log!("vehicle_100 shader log: ", vehicle_100_vert_shader_log);
        log!("torpedo_100 shader log: ", torpedo_100_vert_shader_log);

        let frag_code = include_str!("../shaders/basic.frag");
        let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
        gl.shader_source(&frag_shader, frag_code);
        gl.compile_shader(&frag_shader);
        let basic_frag_shader_log = gl.get_shader_info_log(&frag_shader);

        let vehicle_100_shader_program = gl.create_program().unwrap();
        gl.attach_shader(&vehicle_100_shader_program, &vehicle_100_vert_shader);
        gl.attach_shader(&vehicle_100_shader_program, &frag_shader);

        let torpedo_100_shader_program = gl.create_program().unwrap();
        gl.attach_shader(&torpedo_100_shader_program, &torpedo_100_vert_shader);
        gl.attach_shader(&torpedo_100_shader_program, &frag_shader);

        gl.link_program(&vehicle_100_shader_program);
        gl.link_program(&torpedo_100_shader_program);

        // why not just use the document exposed by web-sys?
        let document = web_sys::window().unwrap().document().unwrap();

        // let et_mouse: EventTarget = canvas.into();
        let et_keys : EventTarget = document.into(); 

        let mut v_200 = Vehicle_100 {
            dx: 0.3,
            dy: 0.3,
            vifo_theta: 0.3,
            velocity_theta: 0.3,
            velocity: 0.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
        };

        let t_100_vec : Rc<RefCell<Vec<Vehicle_100>>> = Rc::new(RefCell::new(vec![]));

        {
            let keypress_cb = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                // log!("keypress {#:?}", event.key_code());
                match event.key_code() {
                    39 => v_200.vifo_theta -= 0.1,
                    38 => {
                        // Don't directly mutate displacement, only adjust velocity 
                    },
                    37 => v_200.vifo_theta += 0.1,
                    32 => {
                        // log!("shoot torpedo");

                        // let { 
                        //     v_dx, v_dy, v_vifo_theta, vehicle_velocity_theta, vehicle_velocity, 
                        // } = *v_200.borrow();;

                        // Looking for a struct comprehension, may cost to keep borrowing (?)
                        // let vehicle_dx = v_200.borrow().dx;
                        // let vehicle_dy = v_200.borrow().dy;
                        // let vehicle_vifo_theta = v_200.borrow().vifo_theta;
                        // let vehicle_velocity_theta = v_200.borrow().velocity_theta;


                        // let torpedo = Vehicle_100 {
                        //     dx
                        // }
                        // t_100_vec.borrow_mut().push();
                    }
                    _ => (),
                }

            }) as Box<dyn FnMut(KeyboardEvent)>);
            et_keys
                .add_event_listener_with_callback("keydown", keypress_cb.as_ref().unchecked_ref())
                .unwrap();
            keypress_cb.forget();

        }

        let vehicle_100_vertices: Vec<f32> = vec![
            0.0, 0.034,
             -0.011, -0.011,
            0.011, -0.011,
        ];

        let torpedo_100_vertices: Vec<f32> = vec![
            0.0, 0.012,
            -0.007, -0.007,
            0.007, -0.007,
        ];

        let vehicle_100_vertex_buffer = gl.create_buffer().unwrap();
        let vehicle_100_js_vertices = js_sys::Float32Array::from(vehicle_100_vertices.as_slice());
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vehicle_100_vertex_buffer));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vehicle_100_js_vertices, GL::STATIC_DRAW);
        let vehicle_100_vertices_position = gl.get_attrib_location(&vehicle_100_shader_program, "a_position") as u32;
        gl.vertex_attrib_pointer_with_i32(vehicle_100_vertices_position, 2, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(vehicle_100_vertices_position);

        let torpedo_100_vertex_buffer = gl.create_buffer().unwrap();
        let torpedo_100_js_vertices = js_sys::Float32Array::from(torpedo_100_vertices.as_slice());
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&torpedo_100_vertex_buffer));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &torpedo_100_js_vertices, GL::STATIC_DRAW);
        let torpedo_100_vertices_position = gl.get_attrib_location(&torpedo_100_shader_program, "a_position") as u32;
        gl.vertex_attrib_pointer_with_i32(torpedo_100_vertices_position, 2, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(torpedo_100_vertices_position);

        let time_location = gl.get_uniform_location(&vehicle_100_shader_program, "u_time");

        let mut timestamp = Instant::now().elapsed().as_secs();







        let gl = gl.clone();
        let render_loop_closure = Rc::new(RefCell::new(None));
        let alias_rlc = render_loop_closure.clone();
        *alias_rlc.borrow_mut() = Some(Closure::wrap(Box::new(move || {

            let now = Instant::now().elapsed().as_secs();
            let time_delta = now - timestamp;
            timestamp = now;

            gl.use_program(Some(&vehicle_100_shader_program));
            gl.clear_color(0.18, 0.13, 0.12, 1.0);
            gl.clear(GL::COLOR_BUFFER_BIT);


            GameTwo::request_animation_frame(render_loop_closure.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));

        GameTwo::request_animation_frame(alias_rlc.borrow().as_ref().unwrap());
    }
}

struct Vehicle_100 {
    dx: f32, // raw displacement in x, y
    dy: f32,
    // vehicle_inertial_frame_orientation_theta: f32,
    vifo_theta: f32,
    // polar description
    velocity_theta: f32,
    velocity: f32,
    // redundant alternate description of velocity, cartesian
    velocity_x: f32,
    velocity_y: f32,
    
}