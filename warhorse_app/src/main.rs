use std::{collections::HashMap, sync::{Arc,Mutex}, time::Duration};

use dioxus::{logger::tracing::{info, warn}, prelude::*};
use warhorse_client::{warhorse_protocol::*, WarhorseClient, WarhorseEvent};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[derive(PartialEq, Eq)]
pub enum InteractiveState {
    Nothing,
    AddFriendModal,
    WhisperFriendModal(Friend),
    RemoveFriendModal(Friend),
    BlockFriendModal(Friend),
    FriendContextMenu(String)
}

pub struct AppState {
    pub client: Option<WarhorseClient>,
    pub received_hello: bool,
    pub received_logged_in: bool,
    pub friends: HashMap<FriendStatus, Vec<Friend>>,
    pub chat_messages: Vec<ChatMessage>, // @todo: make HashMap with room id as key
    pub interactive_state: InteractiveState,
}

impl AppState {
    pub fn send_friend_request(&mut self, id: String) {
        if let Some(client) = &self.client {
            if let Ok(()) = client.send_friend_request(&id) {
                info!("Sent friend request to {}", id);
            }
        }
    }

    pub fn send_user_login_request(&mut self, username: String, password: String) {
        if let Some(client) = &self.client {
            let username_clone = username.clone();
            let user_login_request = UserLogin {
                language: Language::English,
                identity: if Self::is_email_as_username(&username) {
                    LoginUserIdentity::Email(username)
                } else {
                    LoginUserIdentity::AccountName(username)
                },
                password,
            };
            if let Ok(()) = client.send_user_login_request(user_login_request) {
                info!("Sent login request for user {}", username_clone);
            }
        }
    }
    
    pub fn send_user_registration_request(&mut self, account_name: String, password: String, display_name: String, email: String) {
        if let Some(client) = &self.client {
            let account_name_clone = account_name.clone();
            let user_registration_request = UserRegistration {
                account_name,
                password,
                email,
                display_name,
                language: Language::English,
            };
            if let Ok(()) = client.send_user_registration_request(user_registration_request) {
                info!("Sent registration request for user {}", account_name_clone);
            }
        }
    }

    pub fn send_whisper_message(&mut self, friend_id: String, message: String) {
        if let Some(client) = &self.client {
            let message = SendChatMessage {
                language: Language::English,
                message,
                channel: ChatChannel::PrivateMessage(friend_id.clone()),
            };
            if let Ok(()) = client.send_chat_message(message) {
                info!("Sent whisper message to {}", friend_id);
            }
        }
    }

    pub fn send_chat_message(&mut self, message: String) {
        if let Some(client) = &self.client {
            let message = SendChatMessage {
                language: Language::English,
                message,
                channel: ChatChannel::Room("general".to_string()),
            };
            if let Ok(()) = client.send_chat_message(message) {
                info!("Sent chat message to #general");
            }
        }
    }

    pub fn pump(&mut self) -> Vec<WarhorseEvent> {
        if let Some(client) = &self.client {
            client.pump()
        } else {
            vec![]
        }
    }

    pub fn should_show_login(&self) -> bool {
        !self.is_logged_in()
    }

    pub fn should_allow_login_submit(&self) -> bool {
        self.received_hello
    }

    pub fn is_logged_in(&self) -> bool {
        self.received_logged_in
    }

    fn is_email_as_username(input: &str) -> bool {
        input.contains('@')
    }
}

pub fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");
    
    // Initialize client before Dioxus starts
    let client = WarhorseClient::new("http://localhost:3000").ok();
    let context = Arc::new(Mutex::new(AppState {
        client,
        received_hello: false,
        received_logged_in: false,
        friends: HashMap::new(),
        chat_messages: Vec::new(),
        interactive_state: InteractiveState::Nothing,
    }));

    dioxus::LaunchBuilder::new()
        .with_context(context)
        .launch(App);
}

#[component]
pub fn App() -> Element {
    let state = consume_context::<Arc<Mutex<AppState>>>();
    provide_context(state.clone());

    // Periodically run the pump function
    let state_cloned = state.clone();
    use_future(move ||  {
        let state_cloned = state_cloned.clone();
        async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            loop {
                interval.tick().await;

                let events = state_cloned.lock().unwrap().pump();
                for event in events {
                    match event {
                        WarhorseEvent::Hello => {
                            info!("Received Hello event");
                            state_cloned.lock().unwrap().received_hello = true;
                        }
                        WarhorseEvent::LoggedIn => {
                            info!("Received LoggedIn event");
                            state_cloned.lock().unwrap().received_logged_in = true;
                        }
                        WarhorseEvent::Error(error) => {
                            info!("Received Error event: {:?}", error);
                        }
                        WarhorseEvent::FriendsList(friends) => {
                            info!("Received FriendsList event");
                            state_cloned.lock().unwrap().friends = categorize_friends(friends);
                        }
                        WarhorseEvent::FriendRequestAccepted(friend) => {
                            info!("Received FriendRequestAccepted event");
                        }
                        WarhorseEvent::ChatMessage(message) => {
                            info!("Received ChatMessage event");
                            state_cloned.lock().unwrap().chat_messages.push(message);
                        }
                    }
                }
            }
        }
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        if state.lock().unwrap().should_show_login() {
            wh_login {}
        } else {
            wh_main {}
        }
    }
}

#[component]
fn wh_login() -> Element {
    let state = use_context::<Arc<Mutex<AppState>>>();
    let state_cloned = state.clone();
    let state_cloned2 = state.clone();
    
    rsx! {
        section { class: "login",
            h2 { "Login" }
            form { 
                class: "login-form",
                onsubmit: move |e| {
                    e.prevent_default();
                    state_cloned.lock().unwrap().send_user_login_request(
                        e.values().get("username").unwrap_or(&FormValue(vec![])).as_value(),
                        e.values().get("password").unwrap_or(&FormValue(vec![])).as_value()
                    );
                },
                input {
                    r#type: "text",
                    name: "username",
                    placeholder: "Username",
                }
                input {
                    r#type: "password",
                    name: "password",
                    placeholder: "Password",
                }
                button {
                    r#type: "submit",
                    "Login"
                }
            }
            h2 { "Register" }
            form { 
                class: "register-form",
                onsubmit: move |e| {
                    e.prevent_default();
                    state_cloned2.lock().unwrap().send_user_registration_request(
                        e.values().get("account_name").unwrap_or(&FormValue(vec![])).as_value(),
                        e.values().get("password").unwrap_or(&FormValue(vec![])).as_value(),
                        e.values().get("display_name").unwrap_or(&FormValue(vec![])).as_value(),
                        e.values().get("email").unwrap_or(&FormValue(vec![])).as_value()
                    );
                },
                input {
                    r#type: "text",
                    name: "account_name",
                    placeholder: "Account Name",
                }
                input {
                    r#type: "text",
                    name: "display_name",
                    placeholder: "Display Name",
                }
                input {
                    r#type: "text",
                    name: "email",
                    placeholder: "Email",
                }
                input {
                    r#type: "password",
                    name: "password",
                    placeholder: "Password",
                }
                button {
                    r#type: "submit",
                    "Register"
                }
            }
        }
    }
}

#[component]
fn wh_main() -> Element {
    let state = use_context::<Arc<Mutex<AppState>>>();
    rsx! {
        header {
            h1 { "Warhorse" }
            p { "A social backend for video games" }
        }
        wh_sidebar {}
        section { class: "main", 
            h2 { "Main" }
            div { class: "chat",
                for message in state.lock().unwrap().chat_messages.iter() {
                    wh_chat_message {
                        display_name: message.display_name.clone(),
                        time: message.time.to_string(),
                        message: message.message.clone()
                    }
                }
            }
            form { 
                class: "chat-form",
                onsubmit: move |e| {
                    e.prevent_default();
                    state.lock().unwrap().send_chat_message(e.values().get("message").unwrap_or(&FormValue(vec![])).as_value());
                },
                input {
                    r#type: "text",
                    name: "message", 
                    placeholder: "Type a message...",
                }
                button {
                    r#type: "submit",
                    "Send"
                }
            }
        }
    
    
        if state.lock().unwrap().interactive_state == InteractiveState::AddFriendModal {
            wh_add_friend_modal {}
        }
    
        if let InteractiveState::WhisperFriendModal(friend) = &state.lock().unwrap().interactive_state {
            wh_whisper_friend_modal { friend: friend.clone() }
        }
    
        if let InteractiveState::RemoveFriendModal(friend) = &state.lock().unwrap().interactive_state {
            wh_remove_friend_modal { friend: friend.clone() }
        }
    
        if let InteractiveState::BlockFriendModal(friend) = &state.lock().unwrap().interactive_state {
            wh_block_friend_modal { friend: friend.clone() }
        }
    }
}

#[component]
fn wh_sidebar() -> Element {
  let state = use_context::<Arc<Mutex<AppState>>>();
  
   rsx! {
       section { class: "sidebar",
           div { class: "friends-container",
               h2 { "Friends" }
               div { class: "friends",
                    if let Some(friends) = state.lock().unwrap().friends.get(&FriendStatus::PendingRequest) {
                        wh_friend_category { status: FriendStatus::PendingRequest, friends: friends.clone() }
                    }

                    if let Some(friends) = state.lock().unwrap().friends.get(&FriendStatus::Online) {
                        wh_friend_category { status: FriendStatus::Online, friends: friends.clone() }
                    }

                    if let Some(friends) = state.lock().unwrap().friends.get(&FriendStatus::Offline) {
                        wh_friend_category { status: FriendStatus::Offline, friends: friends.clone() }
                    }

                    if let Some(friends) = state.lock().unwrap().friends.get(&FriendStatus::InviteSent) {
                        wh_friend_category { status: FriendStatus::InviteSent, friends: friends.clone() }
                    }

                    if let Some(friends) = state.lock().unwrap().friends.get(&FriendStatus::Blocked) {
                        wh_friend_category { status: FriendStatus::Blocked, friends: friends.clone() }
                    }
                }
                div { class: "add-friend-container",
                    button { 
                        class: "secondary add-friend",
                        onclick: move |_| state.lock().unwrap().interactive_state = InteractiveState::AddFriendModal,
                        "Add Friend"
                    }
                }
           }
       }
   }
}

#[component]
fn wh_friend_category(status: FriendStatus, friends: Vec<Friend>) -> Element {

    let status = match status {
        FriendStatus::Online => "Online",
        FriendStatus::Offline => "Offline",
        FriendStatus::InviteSent => "Invites Sent",
        FriendStatus::PendingRequest => "Pending Requests",
        FriendStatus::Blocked => "Blocked",
    };

    rsx! {
        div { class: "friends-category",
            h3 { "{status}" }
            for friend in friends {
                wh_friend { friend: friend.clone() }
            }
        }
    }
}

#[component]
fn wh_friend(friend: Friend) -> Element {
    let state = use_context::<Arc<Mutex<AppState>>>();
    let friend_id = friend.id.clone();
    rsx! {
        div { 
            class: "friend",
            onclick: move |_| state.lock().unwrap().interactive_state = InteractiveState::FriendContextMenu(friend_id.clone()),
            span { class: "friend-name", "{friend.display_name}" }
            span { class: "friend-menu", "⋮" }
        }
        if match &state.lock().unwrap().interactive_state {
            InteractiveState::FriendContextMenu(id) if *id == friend_id => true,
            _ => false
        } {
            wh_friend_context_menu { friend: friend.clone() }
        }
    }
}

#[component]
fn wh_friend_context_menu(friend: Friend) -> Element {
    let state = use_context::<Arc<Mutex<AppState>>>();
    let state_cloned = state.clone();
    let state_cloned2 = state.clone();
    let state_cloned3 = state.clone();
    let friend_clone = friend.clone();
    let friend_clone2 = friend.clone();
    rsx! {
        div {
            class: "friend-context-menu",
            if friend.status != FriendStatus::Blocked {
                button {
                    onclick: move |e| {
                        e.stop_propagation();
                        state_cloned.lock().unwrap().interactive_state = InteractiveState::WhisperFriendModal(friend_clone.clone());
                    },
                    "Whisper"
                }
                button {
                    class: "secondary",
                    onclick: move |e| {
                        e.stop_propagation();
                        state_cloned2.lock().unwrap().interactive_state = InteractiveState::BlockFriendModal(friend_clone2.clone());
                    },
                    "Block"
                }
            }
            button {
                class: "secondary",
                onclick: move |e| {
                    e.stop_propagation();
                    state_cloned3.lock().unwrap().interactive_state = InteractiveState::RemoveFriendModal(friend.clone());
                },
                "Remove"
            }
        }
        div {
            class: "friend-context-menu-backdrop",
            onclick: move |e| {
                e.stop_propagation();
                state.lock().unwrap().interactive_state = InteractiveState::Nothing;
            }
        }
    }
}

#[component]
fn wh_add_friend_modal() -> Element {
   let state = use_context::<Arc<Mutex<AppState>>>();
   let state_cloned = state.clone();
   rsx! {
       div { class: "modal",
            div { class: "modal-content",
                h2 { "Add Friend" }
                form { 
                    class: "add-friend-form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        state_cloned.lock().unwrap().interactive_state = InteractiveState::Nothing;
                    },
                    input {
                        r#type: "text",
                        name: "friend_id",
                        placeholder: "Friend ID"
                    }
                    
                    button {
                        r#type: "submit",
                        "Add"
                    }
                }
            }
            div { class: "modal-buttons",
                button {
                    class: "secondary",
                    onclick: move |_| state.lock().unwrap().interactive_state = InteractiveState::Nothing,
                    "Close" 
                }
            }
       }
   }
}

#[component]
fn wh_block_friend_modal(friend: Friend) -> Element {
    let state = use_context::<Arc<Mutex<AppState>>>();
    let state_cloned = state.clone();

    rsx! {
        div { class: "modal",
            div { class: "modal-content",
                h2 { "Block Friend" }
                p { "Are you sure you want to block {friend.display_name}?" }
            }
            div { class: "modal-buttons",
                button {
                    class: "secondary",
                    onclick: move |_| state_cloned.lock().unwrap().interactive_state = InteractiveState::Nothing,
                    "Cancel"
                }
                button {
                    class: "danger",
                    onclick: move |_| state.lock().unwrap().interactive_state = InteractiveState::Nothing,
                    "Block"
                }
            }
        }
   }
}

#[component]
fn wh_remove_friend_modal(friend: Friend) -> Element {
   let state = use_context::<Arc<Mutex<AppState>>>();
   let state_cloned = state.clone();

   rsx! {
       div { class: "modal",
           div { class: "modal-content",
               h2 { "Remove Friend" }
               p { "Are you sure you want to remove {friend.display_name}?" }
           }
           div { class: "modal-buttons",
               button {
                   class: "secondary",
                   onclick: move |_| state_cloned.lock().unwrap().interactive_state = InteractiveState::Nothing,
                   "Cancel"
               }
               button {
                   class: "danger",
                   onclick: move |_| state.lock().unwrap().interactive_state = InteractiveState::Nothing,
                   "Remove"
               }
           }
       }
   }
}

#[component]
fn wh_whisper_friend_modal(friend: Friend) -> Element {
   let state = use_context::<Arc<Mutex<AppState>>>();

   rsx! {
       div { class: "modal",
           div { class: "modal-content",
               h2 { "Whisper to {friend.display_name}" }
               form { class: "whisper-form",
                   input {
                       r#type: "text",
                       name: "message",
                       placeholder: "Type a message..."
                   }
                   button {
                       r#type: "submit",
                       "Send"
                   }
               }
           }
           div { class: "modal-buttons",
               button {
                   class: "secondary",
                   onclick: move |_| state.lock().unwrap().interactive_state = InteractiveState::Nothing,
                   "Close"
               }
           }
       }
   }
}

#[component]
fn wh_chat_message(display_name: String, time: String, message: String) -> Element {
   rsx! {
       div { class: "chat-message",
           div { class: "chat-message-author", "{display_name}" }
           div { class: "chat-message-time", "{time}" }
           div { class: "chat-message-content", "{message}" }
       }
   }
}
