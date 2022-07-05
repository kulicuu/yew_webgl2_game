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

}


fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}