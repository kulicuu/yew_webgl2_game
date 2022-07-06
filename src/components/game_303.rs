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

    setup_shaders(gl);



    // create state independently of setup event listener effects api

    let game_state = create_game_state().unwrap();


    set_player_one_events(
        game_state.clone(),
    );


}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn setup_shaders
(
    gl: Arc<GL>,
)
{
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
}

fn set_player_one_events
<'a>
(
    game_state: Arc<Mutex<GameState>>,
)
{

    // why not just use the document exposed by web-sys?
    let document = web_sys::window().unwrap().document().unwrap();

    let et_keys : EventTarget = document.into();

    

    let keypress_cb = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        log!("keypress {#:?}", event.key_code());
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
            32 => {
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
                game_state.lock().unwrap().torps_in_flight.lock().unwrap().push(torpedo);

            },
            _ => (),
        }

    }) as Box<dyn FnMut(KeyboardEvent)>);
    et_keys
        .add_event_listener_with_callback("keydown", keypress_cb.as_ref().unchecked_ref())
        .unwrap();
    keypress_cb.forget();


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
        position_dx: -0.3,
        position_dy: -0.3,
        vifo_theta: Rad(-0.3),
        velocity_theta: Rad(-0.3),
        velocity_scalar: 0.0,
        velocity_dx: 0.0,
        velocity_dy: 0.0,
    }));

    let torps_in_flight = Arc::new(Mutex::new(vec![]));
    let collisions = Arc::new(Mutex::new(vec![]));

    let game_state = GameState {
        player_one: player_one,
        player_two: player_two,
        torps_in_flight: torps_in_flight,
        start_time: Arc::new(Instant::now()),
        elapsed_time: Arc::new(Mutex::new(0)),
        game_over: Arc::new(Mutex::new(false)),
        collisions: collisions,
        mode: Arc::new(mode),
        result: Arc::new(Mutex::new(0)),
    };

    Ok( Arc::new(Mutex::new(game_state)))
    // Err("Not set up yet.")
}


struct GameState {
    player_one: Arc<Mutex<Vehicle_100>>,
    player_two:  Arc<Mutex<Vehicle_100>>,
    torps_in_flight: Arc<Mutex<Vec<Vehicle_100>>>,
    start_time: Arc<Instant>,
    elapsed_time: Arc<Mutex<u128>>,
    game_over: Arc<Mutex<bool>>,
    collisions: Arc<Mutex<Vec<Vehicle_100>>>, 
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