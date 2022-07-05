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
use std::time::*;
// use std::time::{Duration, Instant};
use std::convert::{TryInto};
use std::ops::{Add, Sub, AddAssign, SubAssign};

use gloo_console::log;
use std::f32::consts::PI;

const AMORTIZATION: f32 = 0.95;

// https://github.com/rust-lang/rust/issues/48564#issuecomment-698712971
// std::time invocation causes panic.  There is a comment linked above which solves this
// with the polyfillish stuff below.

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(std::time::Instant);

#[cfg(not(target_arch = "wasm32"))]
impl Instant {
    pub fn now() -> Self { Self(std::time::Instant::now()) }
    pub fn duration_since(&self, earlier: Instant) -> Duration { self.0.duration_since(earlier.0) }
    pub fn elapsed(&self) -> Duration { self.0.elapsed() }
    pub fn checked_add(&self, duration: Duration) -> Option<Self> { self.0.checked_add(duration).map(|i| Self(i)) }
    pub fn checked_sub(&self, duration: Duration) -> Option<Self> { self.0.checked_sub(duration).map(|i| Self(i)) }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = r#"
export function performance_now() {
  return performance.now();
}"#)]
extern "C" {
    fn performance_now() -> f64;
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(u64);

#[cfg(target_arch = "wasm32")]
impl Instant {
    pub fn now() -> Self { Self((performance_now() * 1000.0) as u64) }
    pub fn duration_since(&self, earlier: Instant) -> Duration { Duration::from_micros(self.0 - earlier.0) }
    pub fn elapsed(&self) -> Duration { Self::now().duration_since(*self) }
    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        match duration.as_micros().try_into() {
            Ok(duration) => self.0.checked_add(duration).map(|i| Self(i)),
            Err(_) => None,
        }
    }
    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        match duration.as_micros().try_into() {
            Ok(duration) => self.0.checked_sub(duration).map(|i| Self(i)),
            Err(_) => None,
        }
    }
}

impl Add<Duration> for Instant { type Output = Instant; fn add(self, other: Duration) -> Instant { self.checked_add(other).unwrap() } }
impl Sub<Duration> for Instant { type Output = Instant; fn sub(self, other: Duration) -> Instant { self.checked_sub(other).unwrap() } }
impl Sub<Instant>  for Instant { type Output = Duration; fn sub(self, other: Instant) -> Duration { self.duration_since(other) } }
impl AddAssign<Duration> for Instant { fn add_assign(&mut self, other: Duration) { *self = *self + other; } }
impl SubAssign<Duration> for Instant { fn sub_assign(&mut self, other: Duration) { *self = *self - other; } }


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

        let mut v_200 = Rc::new(RefCell::new(Vehicle_100 {
                position_dx: 0.3,
                position_dy: 0.3,
                vifo_theta: Rad(0.3),
                velocity_theta: Rad(0.3),
                velocity_scalar: 0.0,
                velocity_dx: 0.0,
                velocity_dy: 0.0,
        }));
        let v_200 = v_200.clone();
        let alias_v_200 = v_200.clone();


        let mut tv : Rc<RefCell<Vec<Rc<RefCell<Vehicle_100>>>>> = Rc::new(RefCell::new(vec![]));
        let tv = tv.clone();
        let torpedos_vec = tv.clone();
        // let alias_tv = torpedos_vec.clone();
        
        {
            let keypress_cb = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                // log!("keypress {#:?}", event.key_code());
                match event.key_code() {
                    39 => v_200.borrow_mut().vifo_theta -= Rad(0.3),
                    38 => {
                        // add velocity in the direction of vifo theta
                        // then sum the velocities much like with the torpedo firing.
                        // let vehicle_new_impulse_velocity_vector_scalar = 
                        let vniv_scalar = 0.08;
                        let vniv_theta = v_200.borrow().vifo_theta;

                        let vniv_dx = Rad::cos(vniv_theta) * vniv_scalar;
                        let vniv_dy = Rad::sin(vniv_theta) * vniv_scalar;
                        
                        // let vehicle_new_summed_velocity_dx = 
                        let vnsv_dx = vniv_dx + v_200.borrow().velocity_dx;
                        let vnsv_dy = vniv_dy + v_200.borrow().velocity_dy;

                        let vnsv_theta = Rad::atan(vnsv_dy / vnsv_dx);
                        // let vnsv_scalar = (vnsv_dx as f32) / (Rad::cos(Rad(vnsv_theta)) as f32);
                        let vnsv_scalar = vnsv_dx / Rad::cos(vnsv_theta);
                        let vnsv_scalar_2 = vnsv_dy / Rad::sin(vnsv_theta);
                        // // assert vnvs_scalar == vnsv_scalar_2;
                        v_200.borrow_mut().velocity_dx = vnsv_dx;
                        v_200.borrow_mut().velocity_dy = vnsv_dy;
                        v_200.borrow_mut().velocity_theta = vnsv_theta.into();
                        v_200.borrow_mut().velocity_scalar = vnsv_scalar;

                    },
                    37 => v_200.borrow_mut().vifo_theta += Rad(0.3),
                    32 => {
                        let ticv_scalar = 0.34;
                        // Inherit own charge impulse velocity vector theta from vehicle.
                        // let torpedo_internal_charge_vifo_theta = v_200.borrow().vifo_theta; 
                        let ticv_theta = v_200.borrow().vifo_theta;
                        // this is only true for the initial internal charge.
                        // Torpedos will not be flying where they are pointed in general.
                        // let torpedo_own_impulse_velocity_dx =
                        let ticv_dx = Rad::cos(ticv_theta) * ticv_scalar;
                        let ticv_dy = Rad::sin(ticv_theta) * ticv_scalar;
                        // let torpedo_summed_velocity_dx =
                        let tsv_dx = ticv_dx + v_200.borrow().velocity_dx;
                        let tsv_dy = ticv_dy + v_200.borrow().velocity_dy;

                        // let torpedo_summed_velocity_theta = Rad::atan(tsv_dy / tsv_dx);
                        let tsv_theta = Rad::atan(tsv_dy / tsv_dx);
                        let tsv_scalar = tsv_dx / Rad::cos(tsv_theta);
                        let tsv_scalar_2 = tsv_dy / Rad::sin(tsv_theta);
                        // assert tsv_scalar == tsv_scalar_2;

                        let mut torpedo = Vehicle_100 {
                            position_dx:  v_200.borrow().position_dx,
                            position_dy: v_200.borrow().position_dy,
                            vifo_theta: ticv_theta,
                            velocity_theta: tsv_theta,
                            velocity_scalar: tsv_scalar,
                            velocity_dx: tsv_dx,
                            velocity_dy: tsv_dy,
                        };
                        let torpedo_wrapped = Rc::new(RefCell::new(torpedo));
                        tv.borrow_mut().push(torpedo_wrapped);
                    }
                    _ => (),
                }

            }) as Box<dyn FnMut(KeyboardEvent)>);
            et_keys
                .add_event_listener_with_callback("keydown", keypress_cb.as_ref().unchecked_ref())
                .unwrap();
            keypress_cb.forget();

        }

        let vehicle_050_vertices: Vec<f32> = vec![
            0.034, 0.0, 
             -0.011, -0.011,
            -0.011, 0.011,
        ];

        let vehicle_100_vertices: Vec<f32> = vec![
            0.021, 0.0, 
             -0.008, -0.008,
            -0.008, 0.008,
        ];


        let torpedo_050_vertices: Vec<f32> = vec![
            0.012, 0.0,
            -0.007, -0.007, 
            -0.007, 0.007, 
        ];

        let torpedo_100_vertices: Vec<f32> = vec![
            0.013, 0.0,
            -0.005, -0.005, 
            -0.005, 0.005, 
        ];

        let vehicle_100_vertex_buffer = gl.create_buffer().unwrap();
        let vehicle_100_js_vertices = js_sys::Float32Array::from(vehicle_100_vertices.as_slice());
        // gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vehicle_100_vertex_buffer));
        // gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vehicle_100_js_vertices, GL::STATIC_DRAW);
        // let vehicle_100_vertices_position = gl.get_attrib_location(&vehicle_100_shader_program, "a_position") as u32;
        // gl.vertex_attrib_pointer_with_i32(vehicle_100_vertices_position, 2, GL::FLOAT, false, 0, 0);
        // gl.enable_vertex_attrib_array(vehicle_100_vertices_position);

        let torpedo_100_vertex_buffer = gl.create_buffer().unwrap();
        let torpedo_100_js_vertices = js_sys::Float32Array::from(torpedo_100_vertices.as_slice());
        // gl.bind_buffer(GL::ARRAY_BUFFER, Some(&torpedo_100_vertex_buffer));
        // gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &torpedo_100_js_vertices, GL::STATIC_DRAW);
        // let torpedo_100_vertices_position = gl.get_attrib_location(&torpedo_100_shader_program, "b_position") as u32;
        // gl.vertex_attrib_pointer_with_i32(torpedo_100_vertices_position, 2, GL::FLOAT, false, 0, 0);
        // gl.enable_vertex_attrib_array(torpedo_100_vertices_position);

        let time_location = gl.get_uniform_location(&vehicle_100_shader_program, "u_time");


        
        // let r1_location = gl.get_uniform_location(&shader_program, "r1");

        let v_200_pos_deltas_loc = gl.get_uniform_location(&vehicle_100_shader_program, "pos_deltas");

        let v_200_vifo_theta_loc = gl.get_uniform_location(&vehicle_100_shader_program, "vifo_theta");

        let t_200_pos_deltas_loc = gl.get_uniform_location(&torpedo_100_shader_program, "pos_deltas");

        let t_200_vifo_theta_loc = gl.get_uniform_location(&torpedo_100_shader_program, "vifo_theta");

        let timestamp = Instant::now();
        let mut cursor = timestamp.elapsed().as_millis();
        log!("cursor", cursor);


        let alias_tv = torpedos_vec.clone();

        let gl = gl.clone();
        let render_loop_closure = Rc::new(RefCell::new(None));
        let alias_rlc = render_loop_closure.clone();
        *alias_rlc.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vehicle_100_vertex_buffer));
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vehicle_100_js_vertices, GL::STATIC_DRAW);
            let vehicle_100_vertices_position = gl.get_attrib_location(&vehicle_100_shader_program, "a_position") as u32;
            gl.vertex_attrib_pointer_with_i32(vehicle_100_vertices_position, 2, GL::FLOAT, false, 0, 0);
            gl.enable_vertex_attrib_array(vehicle_100_vertices_position);

            let now = timestamp.elapsed().as_millis();
            let time_delta = now - cursor;
            cursor = now;

            let delta_scalar = (time_delta as f32) * 0.001; 
            // gl.vertex_attrib_pointer_with_i32(vehicle_100_vertices_position, 2, GL::FLOAT, false, 0, 0);
            // gl.enable_vertex_attrib_array(vehicle_100_vertices_position);

            gl.use_program(Some(&vehicle_100_shader_program));
            gl.clear_color(0.99, 0.99, 0.99, 1.0);
            gl.clear(GL::COLOR_BUFFER_BIT);

            let old_pos_dx = alias_v_200.borrow().position_dx;
            let additional_dx = alias_v_200.borrow().velocity_dx * (delta_scalar as f32);
            let mut new_pos_dx = old_pos_dx + additional_dx;
            if new_pos_dx < -1.0 {
                new_pos_dx = new_pos_dx + 2.0;
            }
            if new_pos_dx > 1.0 {
                new_pos_dx = new_pos_dx - 2.0;
            }
            alias_v_200.borrow_mut().position_dx = new_pos_dx;

            let old_pos_dy = alias_v_200.borrow().position_dy;
            let additional_dy = alias_v_200.borrow().velocity_dy * (delta_scalar as f32);
            let mut new_pos_dy = old_pos_dy + additional_dy;
            if new_pos_dy < -1.0 {
                new_pos_dy += 2.0;
            }
            if new_pos_dy > 1.0 {
                new_pos_dy -= 2.0;
            }
            alias_v_200.borrow_mut().position_dy = new_pos_dy; 

            gl.use_program(Some(&vehicle_100_shader_program));

            gl.uniform1f(time_location.as_ref(), 0.4 as f32);
            
            gl.uniform2f(v_200_pos_deltas_loc.as_ref(), new_pos_dx, new_pos_dy);
            gl.uniform1f(v_200_vifo_theta_loc.as_ref(), alias_v_200.borrow().vifo_theta.0);

            gl.draw_arrays(GL::TRIANGLES, 0, 6);


            gl.bind_buffer(GL::ARRAY_BUFFER, Some(&torpedo_100_vertex_buffer));
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &torpedo_100_js_vertices, GL::STATIC_DRAW);
            let torpedo_100_vertices_position = gl.get_attrib_location(&torpedo_100_shader_program, "b_position") as u32;
            gl.vertex_attrib_pointer_with_i32(torpedo_100_vertices_position, 2, GL::FLOAT, false, 0, 0);
            gl.enable_vertex_attrib_array(torpedo_100_vertices_position);



            gl.use_program(Some(&torpedo_100_shader_program));

            // gl.draw_arrays(GL::TRIANGLES, 0, 6);


       
            for torp in torpedos_vec.borrow().iter() {

                let old_pos_dx = torp.borrow().position_dx;
                let additional_dx = torp.borrow().velocity_dx * (delta_scalar as f32);
                let mut new_pos_dx = old_pos_dx + additional_dx;
                if new_pos_dx < -1.0 {
                    new_pos_dx = new_pos_dx + 2.0;
                }
                if new_pos_dx > 1.0 {
                    new_pos_dx = new_pos_dx - 2.0;
                }
                torp.borrow_mut().position_dx = new_pos_dx;
    
                let old_pos_dy = torp.borrow().position_dy;
                let additional_dy = torp.borrow().velocity_dy * (delta_scalar as f32);
                let mut new_pos_dy = old_pos_dy + additional_dy;
                if new_pos_dy < -1.0 {
                    new_pos_dy += 2.0;
                }
                if new_pos_dy > 1.0 {
                    new_pos_dy -= 2.0;
                }
                torp.borrow_mut().position_dy = new_pos_dy; 


                gl.uniform2f(t_200_pos_deltas_loc.as_ref(), new_pos_dx, new_pos_dy);
                gl.uniform1f(t_200_vifo_theta_loc.as_ref(), torp.borrow().vifo_theta.0);

                gl.draw_arrays(GL::TRIANGLES, 0, 6);
            }



            GameTwo::request_animation_frame(render_loop_closure.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));

        GameTwo::request_animation_frame(alias_rlc.borrow().as_ref().unwrap());
    }
}

struct Vehicle_100 {
    position_dx: f32, // raw displacement in x, y
    position_dy: f32,
    // vehicle_inertial_frame_orientation_theta: f32,
    vifo_theta: Rad<f32>,
    // polar description
    velocity_theta: Rad<f32>,
    velocity_scalar: f32,
    // redundant alternate description of velocity, cartesian
    velocity_dx: f32,
    velocity_dy: f32,
    
}

