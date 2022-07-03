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

use gloo_console::log;

use std::f32::consts::PI;

const AMORTIZATION: f32 = 0.95;

pub enum Msg {}

pub struct Game {
    gl: Option<Rc<GL>>,
    node_ref: NodeRef,
    canvas_width: i32,
    canvas_height: i32,
}

struct GameState {
    pos_x: f64,
    pos_y: f64,
}

impl Component for Game {
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
            // <canvas width="{self.canvas_width}" height="{self.canvas_height}" ref={self.node_ref.clone()} />
            <canvas width=2000 height=2000 ref={self.node_ref.clone()} />
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        // let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
        // let gl: GL = canvas
        //     .get_context("webgl2")
        //     .unwrap()
        //     .unwrap()
        //     .dyn_into()
        //     .unwrap();
        // self.gl = Some(Rc::new(gl));
        self.render_gl(ctx.link());
    }
}

impl Game {

    fn request_animation_frame(f: &Closure<dyn FnMut()>) {
        window().unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())
            .expect("should register `requestAnimationFrame` OK");
    }

    fn render_gl(&mut self, _link: &Scope<Self>) {
        let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        // let canvas = Rc::new(canvas);
        let gl: GL = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<GL>()
            .unwrap();
        
        let gl = Some(Rc::new(gl));
        let gl = gl.as_ref().expect("GL Context not initialized!");
        // let canvas = Rc::new(canvas);
        // let document = web_sys::window().unwrap().document().unwrap();
        // let canvas = document.get_element_by_id("canvas").unwrap();
        // let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        // let canvas = canvas.clone();
        // let canvas = canvas.clone();
        let event_target: EventTarget = canvas.into();


        let drag = Rc::new(RefCell::new(false));
        let theta = Rc::new(RefCell::new(0.0));
        let phi = Rc::new(RefCell::new(0.0));
        let dX = Rc::new(RefCell::new(0.0));
        let dY = Rc::new(RefCell::new(0.0));

        let r1 = Rc::new(RefCell::new(0.0));
        let r2 = Rc::new(RefCell::new(0.0));

        let dX_2 = dX.clone();
        let dY_2 = dY.clone();

        // let i1 = Rc::new(RefCell::new(0.0));
        // let i1 = i1.clone();

        let r1 = r1.clone();
        let r2 = r2.clone();


        let canvas_width = Rc::new(RefCell::new(self.canvas_width as f32));
        let canvas_height = Rc::new(RefCell::new(self.canvas_height as f32));
        let document = web_sys::window().unwrap().document().unwrap();
        let event_target_2 : EventTarget = document.into();


        let r3 = r1.clone();

        {
            let keypress_cb = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                // log!("keypress {#:?}", event.key_code());
                match event.key_code() {
                    39 => *r1.borrow_mut() -= 0.1,
                    // 38 => *i1.borrow_mut() += 0.5,
                    38 => {
                        *dX_2.borrow_mut() += - 0.005 * (Rad::sin(Rad(*r1.borrow())));
                        *dY_2.borrow_mut() += - 0.005 * (Rad::cos(Rad(*r1.borrow())));
                    },
                    37 => *r1.borrow_mut() += 0.1,
                    _ => (),
                }

            }) as Box<dyn FnMut(KeyboardEvent)>);
            event_target_2
                .add_event_listener_with_callback("keydown", keypress_cb.as_ref().unchecked_ref())
                .unwrap();
            keypress_cb.forget();
                
        }

        {
            let drag = drag.clone();
            let mousedown_cb = Closure::wrap(Box::new(move |_event: MouseEvent| {
                *drag.borrow_mut() = true;
            }) as Box<dyn FnMut(MouseEvent)>);
            event_target
                .add_event_listener_with_callback("mousedown", mousedown_cb.as_ref().unchecked_ref())
                .unwrap();
            mousedown_cb.forget();
        }

        {
            let drag = drag.clone();
            let mouseup_cb = Closure::wrap(Box::new(move |_event: MouseEvent| {
                *drag.borrow_mut() = false;
            }) as Box<dyn FnMut(MouseEvent)>);
            event_target
                .add_event_listener_with_callback("mouseup", mouseup_cb.as_ref().unchecked_ref())
                .unwrap();
            event_target
                .add_event_listener_with_callback("mouseout", mouseup_cb.as_ref()
                .unchecked_ref())
                .unwrap();
            mouseup_cb.forget();
        }

        {
            let theta = theta.clone();
            let phi = phi.clone();
            let canvas_width = canvas_width.clone();
            let canvas_height = canvas_height.clone();
            let dX = dX.clone();
            let dY = dY.clone();
            let drag = drag.clone();
            let mousemove_cb = Closure::wrap(Box::new(move |event: MouseEvent| {
                if *drag.borrow() {
                    let cw = *canvas_width.borrow();
                    let ch = *canvas_height.borrow();
                    *dX.borrow_mut() = (event.movement_x() as f32) * 2.0 * PI / cw;
                    *dY.borrow_mut() = (event.movement_y() as f32) * 2.0 * PI / ch;
                    *theta.borrow_mut() += *dX.borrow();
                    *phi.borrow_mut() += *dY.borrow();
                }
            }) as Box<dyn FnMut(web_sys::MouseEvent)>);
            event_target
                .add_event_listener_with_callback("mousemove", mousemove_cb.as_ref().unchecked_ref())
                .unwrap();
            mousemove_cb.forget();
        }







        let vert_code = include_str!("./basic.vert");
        let frag_code = include_str!("./basic.frag");

        let vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
        gl.shader_source(&vert_shader, vert_code);
        gl.compile_shader(&vert_shader);

        let ans = gl.get_shader_info_log(&vert_shader);
        log!(ans);

        let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
        gl.shader_source(&frag_shader, frag_code);
        gl.compile_shader(&frag_shader);

        let shader_program = gl.create_program().unwrap();
        gl.attach_shader(&shader_program, &vert_shader);
        gl.attach_shader(&shader_program, &frag_shader);
        gl.link_program(&shader_program);

        let mut timestamp = 0.0;

        let vertices_vv: Vec<f32> = vec![
            0.0, 0.034,
             -0.011, -0.011,
            0.011, -0.011,
        ];

        let vertex_buffer = gl.create_buffer().unwrap();
        let verts = js_sys::Float32Array::from(vertices_vv.as_slice());

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &verts, GL::STATIC_DRAW);

        gl.use_program(Some(&shader_program));

        let verts_position = gl.get_attrib_location(&shader_program, "a_position") as u32;
        gl.vertex_attrib_pointer_with_i32(verts_position, 2, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(verts_position);


        let gl = gl.clone();

        // let game_state = Rc::new(RefCell::new(GameState {
        //     pos_x: 0.3,
        //     pos_y: 0.5,
        // }));

        // let game_state = game_state.clone();

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        let width = Rc::new(self.canvas_width);
        let height = Rc::new(self.canvas_height);

        let width = width.clone();
        let height = height.clone();

        let f1_d_location = gl.get_uniform_location(&shader_program, "f1_displacement");
        let time_location = gl.get_uniform_location(&shader_program, "u_time");

        let f3_d_loc = gl.get_uniform_location(&shader_program, "f3_d");



        // let rotation_location = gl.get_uniform_location(&shader_program, "rotation");
        let r1_location = gl.get_uniform_location(&shader_program, "r1");


        let mut x_d = 0.0;
        let mut y_d = 0.0;
        let mut x2_d = 0.40;
        let mut y2_d = 0.93;





        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            timestamp+= 23.0;
            if (x_d < -1.0) {
                x_d = 1.0 + (x_d + 1.0);
            }
            if (x_d > 1.0) {
                x_d = (x_d - 1.0) - 1.0;
            }
            if (y_d < -1.0) {
                y_d = 1.0 + (y_d + 1.0);
            }
            if (y_d > 1.0) {
                y_d = (y_d - 1.0) - 1.0;
            }

            x_d += *dX.borrow();
            y_d -= *dY.borrow();
            gl.uniform1f(time_location.as_ref(), timestamp as f32);
            // gl.uniform4f(f1_d_location.as_ref(), x_d, y_d, x2_d, y2_d);
            gl.uniform2f(f3_d_loc.as_ref(), x_d, y_d);
            gl.uniform1f(r1_location.as_ref(), *r3.borrow());
            // x_d += 0.0003;


            // y_d += 0.0004;
            x2_d -= 0.0003;
            y2_d -= 0.00023;


            gl.clear_color(0.5, 0.3, 0.4, 1.0);
            gl.clear(GL::COLOR_BUFFER_BIT);

            // gl.draw_arrays_instanced(GL::TRIANGLES, 0, 6, 2);


            gl.draw_arrays(GL::TRIANGLES, 0, 6);

            gl.uniform2f(f3_d_loc.as_ref(), x2_d, y2_d);
            gl.draw_arrays(GL::TRIANGLES, 0, 6);


            Game::request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));

        Game::request_animation_frame(g.borrow().as_ref().unwrap());
    }
}
