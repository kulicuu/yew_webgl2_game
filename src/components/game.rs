use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as GL, window};
use yew::html::Scope;
use yew::{html, Component, Context, Html, NodeRef};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use std::cell::RefCell;
use std::rc::Rc;

use gloo_console::log;

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
            canvas_width: 3000,
            canvas_height: 2000,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            // <canvas width="{self.canvas_width}" height="{self.canvas_height}" ref={self.node_ref.clone()} />
            <canvas width=3000 height=2000 ref={self.node_ref.clone()} />
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
        let gl: GL = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        self.gl = Some(Rc::new(gl));
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
        let gl = self.gl.as_ref().expect("GL Context not initialized!");

        let vert_code = include_str!("./basic.vert");
        let frag_code = include_str!("./basic.frag");

        let vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
        gl.shader_source(&vert_shader, vert_code);
        gl.compile_shader(&vert_shader);

        let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
        gl.shader_source(&frag_shader, frag_code);
        gl.compile_shader(&frag_shader);

        let shader_program = gl.create_program().unwrap();
        gl.attach_shader(&shader_program, &vert_shader);
        gl.attach_shader(&shader_program, &frag_shader);
        gl.link_program(&shader_program);



        let deltas_uniforms_location = gl.get_uniform_block_index(&shader_program, "Deltas");
        // gl.get_uniform_block_index(program,)
        // gl.uniformBlockBinding(program, massUniformsLocation, 0);

        let mut timestamp = 0.0;



        let vertices: Vec<f32> = vec![
            -0.1828, -0.8411, 0.3393993, -0.777, -0.3333442, 0.29555444, -0.0333888, 0.3, 0.5, -0.3, 0.2, 0.2,
        ];


        let vertices_vv: Vec<f32> = vec![
            0.0, 0.034,
             -0.011, -0.011,
            0.011, -0.011,
        ];

        let vertex_buffer = gl.create_buffer().unwrap();
        let verts = js_sys::Float32Array::from(vertices_vv.as_slice());
        // let verts = js_sys::Float32Array::from(vertices.as_slice());

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &verts, GL::STATIC_DRAW);

        gl.use_program(Some(&shader_program));

        let verts_position = gl.get_attrib_location(&shader_program, "a_position") as u32;
        gl.vertex_attrib_pointer_with_i32(verts_position, 2, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(verts_position);

        let gl = gl.clone();

        let game_state = Rc::new(RefCell::new(GameState {
            pos_x: 0.3,
            pos_y: 0.5,
        }));

        let game_state = game_state.clone();

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        let width = Rc::new(self.canvas_width);
        let height = Rc::new(self.canvas_height);

        let width = width.clone();
        let height = height.clone();

        gl.clear_color(0.0, 0.0, 0.0, 1.0);

        gl.enable(GL::BLEND);
        gl.blend_func(GL::ONE, GL::ONE_MINUS_SRC_ALPHA);

        let time_location = gl.get_uniform_location(&shader_program, "u_time");








        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {



            timestamp+= 23.0;
            
            gl.uniform1f(time_location.as_ref(), timestamp as f32);

            gl.draw_arrays_instanced(GL::TRIANGLES, 0, 6, 1);
            Game::request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));

        Game::request_animation_frame(g.borrow().as_ref().unwrap());
    }
}
