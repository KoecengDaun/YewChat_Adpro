use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
    ToggleDarkMode,
    ReactToMessage(usize, String),
}

#[derive(Deserialize, Clone)]
struct MessageData {
    from: String,
    message: String,
    reactions: Option<Vec<String>>, 
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
    Reaction,
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
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    is_dark_mode: bool,
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
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
            is_dark_mode: false, 
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
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
            Msg::ToggleDarkMode => {
                self.is_dark_mode = !self.is_dark_mode;
                true
            }
            Msg::ReactToMessage(message_index, emoji) => {
               
                if let Some(message) = self.messages.get_mut(message_index) {
                    if message.reactions.is_none() {
                        message.reactions = Some(vec![]);
                    }
                    if let Some(reactions) = &mut message.reactions {
                        reactions.push(emoji);
                    }
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let toggle_dark = ctx.link().callback(|_| Msg::ToggleDarkMode);

        
        let bg_main = if self.is_dark_mode { "bg-gray-900" } else { "bg-gray-50" };
        let bg_sidebar = if self.is_dark_mode { "bg-gray-800" } else { "bg-gray-100" };
        let text_main = if self.is_dark_mode { "text-white" } else { "text-gray-900" };
        let text_secondary = if self.is_dark_mode { "text-gray-300" } else { "text-gray-600" };
        let bg_message = if self.is_dark_mode { "bg-gray-700" } else { "bg-white" };
        let border_color = if self.is_dark_mode { "border-gray-600" } else { "border-gray-300" };

        html! {
            <div class={format!("flex w-screen h-screen {}", bg_main)}>
                
                <div class={format!("flex-none w-56 h-screen {}", bg_sidebar)}>
                   
                    <div class="flex justify-between items-center p-3 border-b border-gray-300">
                        <div class={format!("text-xl font-bold {}", text_main)}>{"👥 Users"}</div>
                        <button 
                            onclick={toggle_dark}
                            class={format!("text-2xl p-2 rounded hover:bg-gray-200 dark:hover:bg-gray-600 {}", text_main)}
                            title="Toggle Dark Mode"
                        >
                            {if self.is_dark_mode { "🌞" } else { "🌙" }}
                        </button>
                    </div>
                    
                    
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class={format!("flex m-3 {} rounded-lg p-2 shadow-sm", bg_message)}>
                                    <div>
                                        <img class="w-12 h-12 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class="flex text-sm justify-between">
                                            <div class={format!("font-medium {}", text_main)}>{u.name.clone()}</div>
                                        </div>
                                        <div class={format!("text-xs {}", text_secondary)}>
                                            {"Online 🟢"}
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>

                
                <div class="grow h-screen flex flex-col">
                    
                    <div class={format!("w-full h-14 border-b-2 {} flex items-center px-4", border_color)}>
                        <div class={format!("text-xl font-bold {}", text_main)}>{"💬 YewChat"}</div>
                        <div class={format!("ml-auto text-sm {}", text_secondary)}>
                            {format!("{} users online", self.users.len())}
                        </div>
                    </div>
                    
                   
                    <div class="w-full grow overflow-auto p-4">
                        {
                            self.messages.iter().enumerate().map(|(index, m)| {
                                let user = self.users.iter().find(|u| u.name == m.from);
                                html!{
                                    <div class="mb-4">
                                        <div class="flex items-start space-x-3">
                                            {if let Some(user) = user {
                                                html! { <img class="w-10 h-10 rounded-full" src={user.avatar.clone()} alt="avatar"/> }
                                            } else {
                                                html! {
                                                    <div class="w-10 h-10 rounded-full bg-gray-400 flex items-center justify-center text-white">
                                                        {"👤"}
                                                    </div>
                                                }
                                            }}
                                            <div class="flex-grow">
                                                <div class={format!("text-sm font-medium mb-1 {}", text_main)}>
                                                    {m.from.clone()}
                                                </div>
                                                <div class={format!("p-3 rounded-lg shadow-sm {}", bg_message)}>
                                                    {if m.message.ends_with(".gif") {
                                                        html! { <img class="max-w-xs rounded" src={m.message.clone()}/> }
                                                    } else {
                                                        html! { <span class={text_main}>{m.message.clone()}</span> }
                                                    }}
                                                </div>
                                                
                                                <div class="flex items-center space-x-2 mt-2">
                                                    {["❤️", "👍", "😂", "😮"].iter().enumerate().map(|(_, emoji)| {
                                                        let emoji_str = emoji.to_string();
                                                        let react_callback = ctx.link().callback(move |_| Msg::ReactToMessage(index, emoji_str.clone()));
                                                        html! {
                                                            <button 
                                                                onclick={react_callback}
                                                                class="text-lg hover:scale-125 transition-transform duration-150 p-1 rounded"
                                                                title={format!("React with {}", emoji)}
                                                            >
                                                                {emoji}
                                                            </button>
                                                        }
                                                    }).collect::<Html>()}
                                                    
                                                    {if let Some(reactions) = &m.reactions {
                                                        html! {
                                                            <div class="flex space-x-1 ml-4">
                                                                {reactions.iter().map(|reaction| {
                                                                    html! {
                                                                        <span class={format!("px-2 py-1 rounded-full text-xs {} {}", bg_sidebar, text_main)}>
                                                                            {reaction}
                                                                        </span>
                                                                    }
                                                                }).collect::<Html>()}
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    
                    <div class={format!("w-full h-14 flex px-3 items-center border-t-2 {}", border_color)}>
                        <input 
                            ref={self.chat_input.clone()} 
                            type="text" 
                            placeholder="Type a message..." 
                            class={format!("block w-full py-2 pl-4 mx-3 {} rounded-full outline-none border-2 border-transparent focus:border-blue-500 {}", bg_message, text_main)}
                            name="message" 
                            required=true 
                        />
                        <button 
                            onclick={submit} 
                            class="p-3 bg-blue-600 hover:bg-blue-700 w-12 h-12 rounded-full flex justify-center items-center text-white transition-colors duration-200"
                            title="Send message"
                        >
                            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white w-6 h-6">
                                <path d="M0 0h24v24H0z" fill="none"></path>
                                <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}