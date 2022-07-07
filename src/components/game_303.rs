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

use std::sync::{Arc, Mutex};

use cgmath::prelude::*;
use cgmath::Rad;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::*;
// use std::time::{Duration, Instant};
use std::convert::{TryInto};
use std::ops::{Add, Sub, AddAssign, SubAssign};
use std::collections::HashMap;

use gloo_console::log;
use std::f32::consts::PI;

const AMORTIZATION: f32 = 0.95;


const RESOLUTION : f32 = 8.0;
const SCALE : f32 = 0.08;
const HALF : f32 = SCALE / 2.0;
const STEP : f32 = SCALE / RESOLUTION;

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

pub struct GameThree {
    node_ref: Arc<NodeRef>,
}

impl Component for GameThree {
    type Message = Msg;
    type Properties = ();
    fn create(_ctx: &Context<Self>) -> Self {
        let node_ref = Arc::new(NodeRef::default());
        Self {
            node_ref: node_ref,
        }
    }
    fn view(&self, _ctx: &Context<Self>) -> Html {
 
        html! {
            <canvas width=2000 height=2000 ref={(*self.node_ref).clone()} />
        }
    }
    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {   
        let alt_ref = (*self.node_ref).clone();
        render_game(alt_ref);
    }
}

fn render_game
(
    node_ref: NodeRef,
)
{
    let canvas = node_ref.cast::<HtmlCanvasElement>().unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let gl: GL = canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<GL>()
        .unwrap();
    let gl : Arc<GL> = Arc::new(gl);

    let (
        player_vertex_buffer,
        player_js_vertices,
        player_shader_program,
        player_vertices_position,
        player_pos_deltas_loc,
        player_vifo_theta_loc,
        time_location,
    ) = setup_shaders(gl.clone()).unwrap();

    let (
        torp_vertex_buffer,
        torp_js_vertices,
        torp_shader_program,
        torp_vertices_position,
        torp_pos_deltas_loc,
        torp_vifo_theta_loc,
        time_location_2,
    ) = setup_torp_shaders(gl.clone()).unwrap();

    let game_state = create_game_state().unwrap();

    set_player_one_events(game_state.clone());
    set_player_two_events(game_state.clone());

    // let game_state = game_state.clone();
    let mut cursor = game_state.lock().unwrap().start_time.elapsed().as_millis();

    let render_loop_closure = Rc::new(RefCell::new(None));
    let alias_rlc = render_loop_closure.clone();
    *alias_rlc.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let now = game_state.lock().unwrap().start_time.elapsed().as_millis();
        let time_delta = now - cursor;
        cursor = now;

        gl.clear_color(0.99, 0.99, 0.99, 1.0);
        gl.clear(GL::COLOR_BUFFER_BIT);

        let game_state = update_game_state(time_delta, game_state.clone()).unwrap();
        draw_players(
            gl.clone(),
            game_state.clone(),
            player_vertex_buffer.clone(),
            player_js_vertices.clone(),
            player_shader_program.clone(),
            player_vertices_position.clone(),
            time_location.clone(),
            player_pos_deltas_loc.clone(),
            player_vifo_theta_loc.clone(),
        );

        draw_torps(
            gl.clone(),
            game_state.clone(),
            torp_vertex_buffer.clone(),
            torp_js_vertices.clone(),
            torp_shader_program.clone(),
            torp_vertices_position.clone(),
            time_location_2.clone(),
            torp_pos_deltas_loc.clone(),
            torp_vifo_theta_loc.clone(),
        );

        request_animation_frame(render_loop_closure.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(alias_rlc.borrow().as_ref().unwrap());

}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn setup_torp_shaders
<'a>
(
    gl: Arc<GL>,
)
-> Result<(
    Arc<WebGlBuffer>, // torp vertex buffer
    Arc<js_sys::Float32Array>, // torp_js_vertices,
    Arc<web_sys::WebGlProgram>, // torp_shader_program,
    Arc<u32>, // torp_vertices_position
    Arc<WebGlUniformLocation>, //torp_pos_deltas_loc
    Arc<WebGlUniformLocation>, // torp_vifo_theta_loc
    Arc::<WebGlUniformLocation>, // time_location
), &'a str>
{
    let torpedo_100_vertices: Vec<f32> = vec![
        0.007, 0.0,
        -0.0038, -0.0038, 
        -0.0038, 0.0038, 
    ];

    let torpedo_100_vert_code = include_str!("../shaders/torpedo_100.vert");
    let torpedo_100_vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
    gl.shader_source(&torpedo_100_vert_shader, torpedo_100_vert_code);
    gl.compile_shader(&torpedo_100_vert_shader);
    let torpedo_100_vert_shader_log = gl.get_shader_info_log(&torpedo_100_vert_shader);
    log!("torpedo_100 shader log: ", torpedo_100_vert_shader_log);

    let frag_code = include_str!("../shaders/basic.frag");
    let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
    gl.shader_source(&frag_shader, frag_code);
    gl.compile_shader(&frag_shader);
    let basic_frag_shader_log = gl.get_shader_info_log(&frag_shader);

    let torp_shader_program = gl.create_program().unwrap();
    gl.attach_shader(&torp_shader_program, &torpedo_100_vert_shader);
    gl.attach_shader(&torp_shader_program, &frag_shader);
    gl.link_program(&torp_shader_program);

    let time_location = gl.get_uniform_location(&torp_shader_program, "u_time");

    let torp_vertex_buffer = gl.create_buffer().unwrap();
    let torp_js_vertices = js_sys::Float32Array::from(torpedo_100_vertices.as_slice());
    let torp_pos_deltas_loc = gl.get_uniform_location(&torp_shader_program, "pos_deltas");
    let torp_vifo_theta_loc =  gl.get_uniform_location(&torp_shader_program, "vifo_theta");
    let torp_vertices_position = gl.get_attrib_location(&torp_shader_program, "b_position") as u32;

    let frag_code = include_str!("../shaders/basic.frag");
    let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
    gl.shader_source(&frag_shader, frag_code);
    gl.compile_shader(&frag_shader);
    let basic_frag_shader_log = gl.get_shader_info_log(&frag_shader);

    Ok((
        Arc::new(torp_vertex_buffer),
        Arc::new(torp_js_vertices),
        Arc::new(torp_shader_program),
        Arc::new(torp_vertices_position),
        Arc::new(torp_pos_deltas_loc.unwrap()),
        Arc::new(torp_vifo_theta_loc.unwrap()),
        Arc::new(time_location.unwrap()),
    ))
}

fn setup_shaders
<'a>
(
    gl: Arc<GL>,
)
-> Result<(
    Arc<WebGlBuffer>, // player_vertex_buffer
    Arc<js_sys::Float32Array>, // player_js_vertices,
    Arc<web_sys::WebGlProgram>, // player_shader_program,
    Arc<u32>, // player_vertices_position
    Arc<WebGlUniformLocation>, //player_pos_deltas_loc
    Arc<WebGlUniformLocation>, //player_vifo_theta_loc
    Arc::<WebGlUniformLocation>, // time_location
), &'a str>
{
    let vehicle_100_vertices: Vec<f32> = vec![
        0.021, 0.0, 
         -0.008, -0.008,
        -0.008, 0.008,
    ];

    let vehicle_100_vert_code = include_str!("../shaders/vehicle_100.vert");
    let vehicle_100_vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
    gl.shader_source(&vehicle_100_vert_shader, vehicle_100_vert_code);
    gl.compile_shader(&vehicle_100_vert_shader);
    let vehicle_100_vert_shader_log = gl.get_shader_info_log(&vehicle_100_vert_shader);
    log!("vehicle_100 shader log: ", vehicle_100_vert_shader_log);
    
    let frag_code = include_str!("../shaders/basic.frag");
    let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
    gl.shader_source(&frag_shader, frag_code);
    gl.compile_shader(&frag_shader);
    let basic_frag_shader_log = gl.get_shader_info_log(&frag_shader);

    let player_shader_program = gl.create_program().unwrap();
    gl.attach_shader(&player_shader_program, &vehicle_100_vert_shader);
    gl.attach_shader(&player_shader_program, &frag_shader);
    gl.link_program(&player_shader_program);
    
    let time_location = gl.get_uniform_location(&player_shader_program, "u_time");

    let player_vertex_buffer = gl.create_buffer().unwrap();
    let player_js_vertices = js_sys::Float32Array::from(vehicle_100_vertices.as_slice());
    let player_pos_deltas_loc = gl.get_uniform_location(&player_shader_program, "pos_deltas");
    let player_vifo_theta_loc =  gl.get_uniform_location(&player_shader_program, "vifo_theta");
    let player_vertices_position = gl.get_attrib_location(&player_shader_program, "a_position") as u32;
    
    Ok((
        Arc::new(player_vertex_buffer),
        Arc::new(player_js_vertices),
        Arc::new(player_shader_program),
        Arc::new(player_vertices_position),
        Arc::new(player_pos_deltas_loc.unwrap()),
        Arc::new(player_vifo_theta_loc.unwrap()),
        Arc::new(time_location.unwrap()),
    ))
}

fn set_player_two_events
<'a>
(
    game_state: Arc<Mutex<GameState>>,
)
{
    let document = web_sys::window().unwrap().document().unwrap();
    let et_keys : EventTarget = document.into();
    let keypress_cb = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        match event.key_code() {
    74 => game_state.lock().unwrap().player_two.lock().unwrap().vifo_theta -= Rad(0.1),
    79 => {
        let vniv_scalar = 0.08;
        let vniv_theta = game_state.lock().unwrap().player_two.lock().unwrap().vifo_theta;
        let vniv_dx = Rad::cos(vniv_theta) * vniv_scalar;
        let vniv_dy = Rad::sin(vniv_theta) * vniv_scalar;
        // let vehicle_new_summed_velocity_dx = 
        let vnsv_dx = vniv_dx + game_state.lock().unwrap().player_two.lock().unwrap().velocity_dx;
        let vnsv_dy = vniv_dy + game_state.lock().unwrap().player_two.lock().unwrap().velocity_dy;
        let vnsv_theta = Rad::atan(vnsv_dy / vnsv_dx);
        // let vnsv_scalar = (vnsv_dx as f32) / (Rad::cos(Rad(vnsv_theta)) as f32);
        let vnsv_scalar = vnsv_dx / Rad::cos(vnsv_theta);
        let vnsv_scalar_2 = vnsv_dy / Rad::sin(vnsv_theta);
        // // assert vnvs_scalar == vnsv_scalar_2;
        game_state.lock().unwrap().player_two.lock().unwrap().velocity_dx = vnsv_dx;
        game_state.lock().unwrap().player_two.lock().unwrap().velocity_dy = vnsv_dy;
        game_state.lock().unwrap().player_two.lock().unwrap().velocity_theta = vnsv_theta.into();
        game_state.lock().unwrap().player_two.lock().unwrap().velocity_scalar = vnsv_scalar;
    },
    186 => game_state.lock().unwrap().player_two.lock().unwrap().vifo_theta += Rad(0.1),
    32 => {
        let ticv_scalar = 0.34;
        let ticv_theta = game_state.lock().unwrap().player_two.lock().unwrap().vifo_theta;
        // let torpedo_own_impulse_velocity_dx =
        let ticv_dx = Rad::cos(ticv_theta) * ticv_scalar;
        let ticv_dy = Rad::sin(ticv_theta) * ticv_scalar;
        // let torpedo_summed_velocity_dx =
        let tsv_dx = ticv_dx + game_state.lock().unwrap().player_two.lock().unwrap().velocity_dx;
        let tsv_dy = ticv_dy + game_state.lock().unwrap().player_two.lock().unwrap().velocity_dy;
        // let torpedo_summed_velocity_theta = Rad::atan(tsv_dy / tsv_dx);
        let tsv_theta = Rad::atan(tsv_dy / tsv_dx);
        let tsv_scalar = tsv_dx / Rad::cos(tsv_theta);
        let tsv_scalar_2 = tsv_dy / Rad::sin(tsv_theta);
        // assert tsv_scalar == tsv_scalar_2;
        let t_dx = game_state.lock().unwrap().player_two.lock().unwrap().position_dx;
        let t_dy = game_state.lock().unwrap().player_two.lock().unwrap().position_dy;
        let mut torpedo = Vehicle_100 {
            position_dx:  t_dx,
            position_dy: t_dy,
            vifo_theta: ticv_theta,
            velocity_theta: tsv_theta,
            velocity_scalar: tsv_scalar,
            velocity_dx: tsv_dx,
            velocity_dy: tsv_dy,
        };
        game_state.lock().unwrap().torps_in_flight.lock().unwrap().push(Arc::new(Mutex::new(torpedo)));
    },
            _ => (),
        }

    }) as Box<dyn FnMut(KeyboardEvent)>);
    et_keys
        .add_event_listener_with_callback("keydown", keypress_cb.as_ref().unchecked_ref())
        .unwrap();
    keypress_cb.forget();
}

fn set_player_one_events
<'a>
(
    game_state: Arc<Mutex<GameState>>,
)
{
    let document = web_sys::window().unwrap().document().unwrap();
    let et_keys : EventTarget = document.into();
    let keypress_cb = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        match event.key_code() {
            39 => game_state.lock().unwrap().player_one.lock().unwrap().vifo_theta -= Rad(0.1),
            38 => {
                let vniv_scalar = 0.08;
                let vniv_theta = game_state.lock().unwrap().player_one.lock().unwrap().vifo_theta;
                let vniv_dx = Rad::cos(vniv_theta) * vniv_scalar;
                let vniv_dy = Rad::sin(vniv_theta) * vniv_scalar;
                // let vehicle_new_summed_velocity_dx = 
                let vnsv_dx = vniv_dx + game_state.lock().unwrap().player_one.lock().unwrap().velocity_dx;
                let vnsv_dy = vniv_dy + game_state.lock().unwrap().player_one.lock().unwrap().velocity_dy;
                let vnsv_theta = Rad::atan(vnsv_dy / vnsv_dx);
                // let vnsv_scalar = (vnsv_dx as f32) / (Rad::cos(Rad(vnsv_theta)) as f32);
                let vnsv_scalar = vnsv_dx / Rad::cos(vnsv_theta);
                let vnsv_scalar_2 = vnsv_dy / Rad::sin(vnsv_theta);
                // // assert vnvs_scalar == vnsv_scalar_2;
                game_state.lock().unwrap().player_one.lock().unwrap().velocity_dx = vnsv_dx;
                game_state.lock().unwrap().player_one.lock().unwrap().velocity_dy = vnsv_dy;
                game_state.lock().unwrap().player_one.lock().unwrap().velocity_theta = vnsv_theta.into();
                game_state.lock().unwrap().player_one.lock().unwrap().velocity_scalar = vnsv_scalar;
            },
            37 => game_state.lock().unwrap().player_one.lock().unwrap().vifo_theta += Rad(0.1),
            96 => {
                let ticv_scalar = 0.34;
                let ticv_theta = game_state.lock().unwrap().player_one.lock().unwrap().vifo_theta;
                // let torpedo_own_impulse_velocity_dx =
                let ticv_dx = Rad::cos(ticv_theta) * ticv_scalar;
                let ticv_dy = Rad::sin(ticv_theta) * ticv_scalar;
                // let torpedo_summed_velocity_dx =
                let tsv_dx = ticv_dx + game_state.lock().unwrap().player_one.lock().unwrap().velocity_dx;
                let tsv_dy = ticv_dy + game_state.lock().unwrap().player_one.lock().unwrap().velocity_dy;
                // let torpedo_summed_velocity_theta = Rad::atan(tsv_dy / tsv_dx);
                let tsv_theta = Rad::atan(tsv_dy / tsv_dx);
                let tsv_scalar = tsv_dx / Rad::cos(tsv_theta);
                let tsv_scalar_2 = tsv_dy / Rad::sin(tsv_theta);
                // assert tsv_scalar == tsv_scalar_2;
                let t_dx = game_state.lock().unwrap().player_one.lock().unwrap().position_dx;
                let t_dy = game_state.lock().unwrap().player_one.lock().unwrap().position_dy;
                let mut torpedo = Vehicle_100 {
                    position_dx:  t_dx,
                    position_dy: t_dy,
                    vifo_theta: ticv_theta,
                    velocity_theta: tsv_theta,
                    velocity_scalar: tsv_scalar,
                    velocity_dx: tsv_dx,
                    velocity_dy: tsv_dy,
                };
                game_state.lock().unwrap().torps_in_flight.lock().unwrap().push(Arc::new(Mutex::new(torpedo)));
            },
            _ => (),
        }

    }) as Box<dyn FnMut(KeyboardEvent)>);
    et_keys
        .add_event_listener_with_callback("keydown", keypress_cb.as_ref().unchecked_ref())
        .unwrap();
    keypress_cb.forget();
}

fn draw_torps
(
    gl: Arc<GL>,
    game_state: Arc<Mutex<GameState>>,
    torp_vertex_buffer: Arc<WebGlBuffer>,
    torp_js_vertices: Arc<js_sys::Float32Array>,
    shader_program: Arc<web_sys::WebGlProgram>,
    torp_vertices_position: Arc<u32>,
    time_location: Arc<WebGlUniformLocation>,
    torp_pos_deltas_loc: Arc<WebGlUniformLocation>,
    torp_vifo_theta_loc: Arc<WebGlUniformLocation>,
)
{
    gl.use_program(Some(&shader_program));
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&torp_vertex_buffer));
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &torp_js_vertices, GL::STATIC_DRAW);
    gl.vertex_attrib_pointer_with_i32(*torp_vertices_position, 2, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(*torp_vertices_position);

    gl.use_program(Some(&shader_program));
    gl.uniform1f(Some(&time_location), 0.4 as f32);

    for (idx, torp) in game_state.lock().unwrap()
    .torps_in_flight.lock().unwrap()
    .iter().enumerate() {
        let new_pos_dx = torp.lock().unwrap().position_dx;
        let new_pos_dy = torp.lock().unwrap().position_dy;
        let torp_vifo_theta = torp.lock().unwrap().vifo_theta;
        gl.uniform2f(Some(&torp_pos_deltas_loc), new_pos_dx, new_pos_dy);
        gl.uniform1f(Some(&torp_vifo_theta_loc), torp_vifo_theta.0);
        gl.draw_arrays(GL::TRIANGLES, 0, 6);
    }
}

fn draw_players
// <'a>
(
    gl: Arc<GL>,
    game_state: Arc<Mutex<GameState>>,
    player_vertex_buffer: Arc<WebGlBuffer>,
    player_js_vertices: Arc<js_sys::Float32Array>,
    shader_program: Arc<web_sys::WebGlProgram>,
    player_vertices_position: Arc<u32>,
    time_location: Arc<WebGlUniformLocation>,
    player_pos_deltas_loc: Arc<WebGlUniformLocation>,
    player_vifo_theta_loc: Arc<WebGlUniformLocation>,
)
{
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&player_vertex_buffer));
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &player_js_vertices, GL::STATIC_DRAW);
    gl.vertex_attrib_pointer_with_i32(*player_vertices_position, 2, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(*player_vertices_position);

    gl.use_program(Some(&shader_program));
    gl.uniform1f(Some(&time_location), 0.4 as f32);

    let new_pos_dx = game_state.lock().unwrap().player_one.lock().unwrap().position_dx;
    let new_pos_dy = game_state.lock().unwrap().player_one.lock().unwrap().position_dy;

    gl.uniform2f(Some(&player_pos_deltas_loc), new_pos_dx, new_pos_dy);
    
    let new_vifo_theta = game_state.lock().unwrap().player_one.lock().unwrap().vifo_theta;
    gl.uniform1f(Some(&player_vifo_theta_loc), new_vifo_theta.0);
    gl.draw_arrays(GL::TRIANGLES, 0, 6);

    let new_pos_dx = game_state.lock().unwrap().player_two.lock().unwrap().position_dx;
    let new_pos_dy = game_state.lock().unwrap().player_two.lock().unwrap().position_dy;

    gl.uniform2f(Some(&player_pos_deltas_loc), new_pos_dx, new_pos_dy);
    
    let new_vifo_theta = game_state.lock().unwrap().player_two.lock().unwrap().vifo_theta;
    gl.uniform1f(Some(&player_vifo_theta_loc), new_vifo_theta.0);
    gl.draw_arrays(GL::TRIANGLES, 0, 6);
}

// fn update_positions 
fn update_game_state // A slight misnomer, as game state is also mutated by event-handlers.
<'a>
(
    time_delta: u128,
    game_state: Arc<Mutex<GameState>>,
)
-> Result<Arc<Mutex<GameState>>, &'a str>
{
    let mut collisions_map : HashMap<String, CollisionSpace> = HashMap::new();

    let delta_scalar = (time_delta as f32) * 0.001;
    let old_pos_dx = game_state.lock().unwrap().player_one.lock().unwrap().position_dx;
    let additional_dx = game_state.lock().unwrap().player_one.lock().unwrap().velocity_dx * (delta_scalar as f32);
    let mut new_pos_dx = old_pos_dx + additional_dx;
    if new_pos_dx < -1.0 {
        new_pos_dx = new_pos_dx + 2.0;
    }
    if new_pos_dx > 1.0 {
        new_pos_dx = new_pos_dx - 2.0;
    }
    let old_pos_dy = game_state.lock().unwrap().player_one.lock().unwrap().position_dy;
    let additional_dy = game_state.lock().unwrap().player_one.lock().unwrap().velocity_dy * (delta_scalar as f32);
    let mut new_pos_dy = old_pos_dy + additional_dy;
    if new_pos_dy < -1.0 {
        new_pos_dy += 2.0;
    }
    if new_pos_dy > 1.0 {
        new_pos_dy -= 2.0;
    }

    game_state.lock().unwrap().player_one.lock().unwrap().position_dx = new_pos_dx;
    game_state.lock().unwrap().player_one.lock().unwrap().position_dy = new_pos_dy;


    for i in -10..10 {
        for j in -10..10 {
            let base_dx = ((new_pos_dx * 1000.0) as i32) + i;
            let base_dy = ((new_pos_dx * 1000.0) as i32) + j;

            let s_key : String = [base_dx.to_string(), String::from(":"), base_dy.to_string()].concat();
            
            if collisions_map.contains_key(&s_key) {
                let (k, mut tuple) = collisions_map.remove_entry(&s_key).unwrap();
                tuple.0 = true;
                collisions_map.insert(k, tuple);
            } else {
                let tuple : CollisionSpace = (true, false, vec![]);
                collisions_map.insert(s_key, tuple);   
            }
        }
    }    
    
    let old_pos_dx = game_state.lock().unwrap().player_two.lock().unwrap().position_dx;
    let additional_dx = game_state.lock().unwrap().player_two.lock().unwrap().velocity_dx * (delta_scalar as f32);
    let mut new_pos_dx = old_pos_dx + additional_dx;
    if new_pos_dx < -1.0 {
        new_pos_dx = new_pos_dx + 2.0;
    }
    if new_pos_dx > 1.0 {
        new_pos_dx = new_pos_dx - 2.0;
    }

    let old_pos_dy = game_state.lock().unwrap().player_two.lock().unwrap().position_dy;
    let additional_dy = game_state.lock().unwrap().player_two.lock().unwrap().velocity_dy * (delta_scalar as f32);
    let mut new_pos_dy = old_pos_dy + additional_dy;
    if new_pos_dy < -1.0 {
        new_pos_dy += 2.0;
    }
    if new_pos_dy > 1.0 {
        new_pos_dy -= 2.0;
    }
    game_state.lock().unwrap().player_two.lock().unwrap().position_dx = new_pos_dx;
    game_state.lock().unwrap().player_two.lock().unwrap().position_dy = new_pos_dy;


    for i in -10..10 {
        for j in -10..10 {
            let base_dx = ((new_pos_dx * 1000.0) as i32) + i;
            let base_dy = ((new_pos_dx * 1000.0) as i32) + j;

            let s_key : String = [base_dx.to_string(), String::from(":"), base_dy.to_string()].concat();
            
            if collisions_map.contains_key(&s_key) {
                let (k, mut tuple) = collisions_map.remove_entry(&s_key).unwrap();
                tuple.1 = true;
                collisions_map.insert(k, tuple);
            } else {
                let tuple : CollisionSpace = (true, false, vec![]);
                collisions_map.insert(s_key, tuple);   
            }
        }
    } 

    let mut removals: Vec<usize> = vec![];
    
    for (idx, torp) in game_state.lock().unwrap()
    .torps_in_flight.lock().unwrap()
    .iter().enumerate() {
        let pos_dx = torp.lock().unwrap().position_dx;
        let pos_dy = torp.lock().unwrap().position_dy;
        let v_dx = torp.lock().unwrap().velocity_dx;
        let v_dy = torp.lock().unwrap().velocity_dy;
        let new_pos_dx = pos_dx + (delta_scalar * v_dx);
        let new_pos_dy = pos_dy + (delta_scalar * v_dy);

        if (new_pos_dx < -1.0) || (new_pos_dx > 1.0) || (new_pos_dy < -1.0) || (new_pos_dy > 1.0) {
            removals.push(idx);
        } else {
            torp.lock().unwrap().position_dx = new_pos_dx;
            torp.lock().unwrap().position_dy = new_pos_dy;

            for i in -5..5 {
                for j in -5..5 {

                    let base_dx = ((new_pos_dx * 1000.0) as i32) + i;
                    let base_dy = ((new_pos_dx * 1000.0) as i32) + j;
        
                    let s_key : String = [base_dx.to_string(), String::from(":"), base_dy.to_string()].concat();
                    
                    if collisions_map.contains_key(&s_key) {
                        let (k, mut tuple) = collisions_map.remove_entry(&s_key).unwrap();
                        // tuple.0 = true;
                        tuple.2.push(idx);
                        collisions_map.insert(k, tuple);
                    } else {
                        let tuple : CollisionSpace = (true, false, vec![]);
                        collisions_map.insert(s_key, tuple);   
                    }
                }
            }
        }
    }

    for i in removals.iter() {
        if *i < game_state.lock().unwrap().torps_in_flight.lock().unwrap().len() {
            game_state.lock().unwrap().torps_in_flight.lock().unwrap().clone().remove(*i);
        }
    }


    for (key, space) in collisions_map.iter() {
        if (space.0 == true) && (space.2.len() > 0) {
            log!("Torpedo kills player one.");
        }

        if (space.0 == true) && (space.1 == true) {
            log!("vehicle collision");
        }

        if (space.1 == true) && (space.2.len() > 0) {
            log!("Torpedo kills player 2.");
        }

    }


    collisions_map.clear();

    Ok(game_state)
}



fn create_game_state
<'a>
()
-> Result<Arc<Mutex<GameState>>, &'a str>
{
    let mode = 0; // Notionally code for 2-player local.

    let player_one = Arc::new(Mutex::new(Vehicle_100 {
        position_dx: 0.3,
        position_dy: 0.3,
        vifo_theta: Rad(0.3),
        velocity_theta: Rad(0.3),
        velocity_scalar: 0.0,
        velocity_dx: 0.0,
        velocity_dy: 0.0,
    }));

    let player_two = Arc::new(Mutex::new(Vehicle_100 {
        position_dx: -0.4,
        position_dy: -0.4,
        vifo_theta: Rad(-0.3),
        velocity_theta: Rad(-0.3),
        velocity_scalar: 0.0,
        velocity_dx: 0.0,
        velocity_dy: 0.0,
    }));

    let torps_in_flight = Arc::new(Mutex::new(vec![]));
    let collisions_two : Arc<Mutex<
        HashMap<&'static str, Arc<Mutex<CollisionSpace>>>
    >> = 
        Arc::new(Mutex::new(HashMap::new()));

    // let collisions : Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(vec![]));

    let game_state = GameState {
        player_one: player_one,
        player_two: player_two,
        torps_in_flight: torps_in_flight,
        start_time: Arc::new(Instant::now()),
        elapsed_time: Arc::new(Mutex::new(0)),
        game_over: Arc::new(Mutex::new(false)),
        collisions_map: collisions_two,
        mode: Arc::new(mode),
        result: Arc::new(Mutex::new(0)),
    };

    // let game_state = GameStateOld {
    //     player_one: player_one,
    //     player_two: player_two,
    //     torps_in_flight: torps_in_flight,
    //     start_time: Arc::new(Instant::now()),
    //     elapsed_time: Arc::new(Mutex::new(0)),
    //     game_over: Arc::new(Mutex::new(false)),
    //     collisions: collisions,
    //     mode: Arc::new(mode),
    //     result: Arc::new(Mutex::new(0)),
    // };

    Ok(Arc::new(Mutex::new(game_state)))
}


// type CollisionSpace = (bool, bool, Arc<Mutex<Vec<usize>>>);
type CollisionSpace = (bool, bool, Vec<usize>);

// struct GameStateOld {
//     player_one: Arc<Mutex<Vehicle_100>>,
//     player_two:  Arc<Mutex<Vehicle_100>>,
//     torps_in_flight: Arc<Mutex<Vec<Arc<Mutex<Vehicle_100>>>>>,
//     start_time: Arc<Instant>,
//     elapsed_time: Arc<Mutex<u128>>,
//     game_over: Arc<Mutex<bool>>,
//     collisions: Arc<Mutex<Vec<Vehicle_100>>>, 
//     // model an explosion around a vector sum of the collided vehicles, with extra effects. covering torpedo collisions
//     // This would be a good place to use Rust traits.
//     result: Arc<Mutex<u8>>,
//     mode: Arc<u8>, // 1 player vs computer, 2 player local, 2 player network
// }


// type CollisionsMap<'a> = Arc<Mutex<HashMap<&'a str, Arc<Mutex<CollisionSpace>>>>>;

struct GameState 
{
    player_one: Arc<Mutex<Vehicle_100>>,
    player_two:  Arc<Mutex<Vehicle_100>>,
    torps_in_flight: Arc<Mutex<Vec<Arc<Mutex<Vehicle_100>>>>>,
    start_time: Arc<Instant>,
    elapsed_time: Arc<Mutex<u128>>,
    game_over: Arc<Mutex<bool>>,
    collisions_map: Arc<Mutex<HashMap<&'static str, Arc<Mutex<CollisionSpace>>>>>,
    // model an explosion around a vector sum of the collided vehicles, with extra effects. covering torpedo collisions
    // This would be a good place to use Rust traits.
    result: Arc<Mutex<u8>>,
    mode: Arc<u8>, // 1 player vs computer, 2 player local, 2 player network
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

