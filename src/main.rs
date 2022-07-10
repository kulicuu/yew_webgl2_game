#![recursion_limit = "1024"]

#[macro_use]
extern crate lazy_static;

mod components;
mod services;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

// use components::chat::Chat;
// use components::login::Login;
// use components::game_202::{GameTwo};
use components::game_303::{GameThree};
use components::particles::Particles;
use components::game_404::{GameFour};
use components::game_505::{GameFive};


use wasm_logger;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
pub enum Route {
    #[at("/particles")]
    Particles,
    // #[at("/")]
    // GameTwo,
    #[at("/")]
    GameFour,
    #[at("/game_5")]
    GameFive,
    #[at("/game_3_old")]
    GameThree,
    // #[at("/login")]
    // Login,
    // #[at("/chat")]
    // Chat,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub type User = Rc<UserInner>;

#[derive(Debug, PartialEq)]
pub struct UserInner {
    pub username: RefCell<String>,
}

#[function_component(App)]
fn main() -> Html {
    let ctx = use_state(|| {
        Rc::new(UserInner {
            username: RefCell::new("initial".into()),
        })
    });
    html! {
        <ContextProvider<User> context={(*ctx).clone()}>
            <BrowserRouter>
                <div>
                    <Switch<Route> render={Switch::render(switch)}/>
                </div>
            </BrowserRouter>
        </ContextProvider<User>>
    }
}

fn switch(selected_route: &Route) -> Html {
    match selected_route {
        Route::Particles => html! {<Particles />},
        // Route::GameTwo => html! {<GameTwo />},
        Route::GameThree => html! {<GameThree />},
        Route::GameFour => html! {<GameFour />},
        Route::GameFive => html! {<GameFive />},
        // Route::Login => html! {<Login />},
        // Route::Chat => html! {<Chat/>},
        Route::NotFound => html! {<h1>{"404"}</h1>},
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
