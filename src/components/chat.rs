
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};
use crate::components::game::Game;


pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    messages_2: Vec<String>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::info!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            messages_2: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                self.messages_2.push(s);
                true
                // let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();

                // let msg: Option<WebSocketMessage> = match s {
                //     Some(serde_json::from_str(&s).unwrap()) => 
                // }

                // match msg.message_type {
                //     MsgTypes::Users => {
                //         let users_from_message = msg.data_array.unwrap_or_default();
                //         self.users = users_from_message
                //             .iter()
                //             .map(|u| UserProfile {
                //                 name: u.into(),
                //             })
                //             .collect();
                //         return true;
                //     }
                //     MsgTypes::Message => {
                //         let message_data: MessageData =
                //             serde_json::from_str(&msg.data.unwrap()).unwrap();
                //         self.messages.push(message_data);
                //         return true;
                //     }
                //     _ => {
                //         return false;
                //     }
                // }

            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);

        html! {
            <div>
                <div>
                    <div>{"Users"}</div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div>
                                    <div class="flex text-xs justify-between">
                                        <div>{u.name.clone()}</div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div>
                    <div>
                        <div>{"💬 Chat!"}</div>
                    </div>
                    <div>
                        {
                            self.messages_2.iter().map(|m| {

                                html!{
                                    <div> { m } </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    <div>
                        <input ref={self.chat_input.clone()} type="text" placeholder="Message" name="message" required=true />
                        <button onclick={submit} >
                        </button>
                    </div>
                </div>
                <Game />
            </div>
        }
    }
}
