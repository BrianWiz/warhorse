mod warhorse;

use std::{collections::HashMap, sync::{Arc,Mutex}, time::Duration};

use dioxus::{desktop::{tao::window::Theme, wry::dpi::{LogicalSize, Size}, WindowBuilder}, logger::tracing::info, prelude::*};
use warhorse_client::{warhorse_protocol::*, WarhorseClient, WarhorseEvent};

use warhorse::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

pub fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");
    
    // Initialize client before Dioxus starts
    let client = WarhorseClient::new("http://localhost:3000").ok();
    let context = Arc::new(Mutex::new(Warhorse {
        client,
    }));

    dioxus::LaunchBuilder::new()
        .with_context(context)
        .with_cfg(dioxus::desktop::Config::new()
            .with_background_color((0, 0, 0, 255))
            .with_window(WindowBuilder::new()
                .with_title("Warhorse")
                .with_inner_size(Size::Logical(LogicalSize::new(800.0, 600.0)))
                .with_resizable(true)
                .with_theme(Some(Theme::Dark))
            )
        )
        .launch(App);
}

#[component]
pub fn App() -> Element {
    let wh = consume_context::<Arc<Mutex<Warhorse>>>();
    
    let mut received_hello = use_signal(|| ReceivedHello(false));
    let mut received_logged_in = use_signal(|| ReceivedLoggedIn(false));
    let mut friends_list = use_signal(|| FriendsList(HashMap::new()));
    let mut chat_messages = use_signal(|| ChatMessages(vec![]));
    let interactive_state = use_signal(|| InteractiveState::Nothing);

    provide_context(wh.clone());
    provide_context(received_hello);
    provide_context(received_logged_in);
    provide_context(friends_list);
    provide_context(chat_messages);
    provide_context(interactive_state);

    // Periodically run the pump function
    use_future(move ||  {
        let wh_cloned = wh.clone();
        async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            loop {
                interval.tick().await;

                let events = wh_cloned.lock().unwrap().pump();
                for event in events {
                    match event {
                        WarhorseEvent::Hello => {
                            info!("Received Hello event");
                            received_hello.write().0 = true;
                        }
                        WarhorseEvent::LoggedIn => {
                            info!("Received LoggedIn event");
                            received_logged_in.write().0 = true;
                        }
                        WarhorseEvent::Error(error) => {
                            info!("Received Error event: {:?}", error);
                        }
                        WarhorseEvent::FriendsList(friends) => {
                            info!("Received FriendsList event");
                            friends_list.write().0 = categorize_friends(friends);
                        }
                        WarhorseEvent::FriendRequestReceived(friend) => {
                            info!("Received FriendRequestReceived event");
                        }
                        WarhorseEvent::FriendRequestAccepted(friend) => {
                            info!("Received FriendRequestAccepted event");
                        }
                        WarhorseEvent::ChatMessage(message) => {
                            info!("Received ChatMessage event");
                            chat_messages.write().0.push(message);
                        }
                    }
                }
            }
        }
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        if !received_logged_in.read().0 {
            wh_login {}
        } else {
            wh_main {}
        }
    }
}

#[component]
fn wh_login() -> Element {
    let received_hello = use_context::<Signal<ReceivedHello>>();
    let wh_cloned = use_context::<Arc<Mutex<Warhorse>>>();
    let wh_cloned2 = wh_cloned.clone();

    rsx! {
        if received_hello.read().0 {
            section { class: "login",
                h2 { "Login" }
                form { 
                    class: "login-form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        wh_cloned.lock().unwrap().send_user_login_request(
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
                        wh_cloned2.lock().unwrap().send_user_registration_request(
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
        } else {
            section { class: "login",
                h2 { "Connecting to Warhorse..." }
            }
        }
    }
}

#[component]
fn wh_main() -> Element {
    let wh = use_context::<Arc<Mutex<Warhorse>>>();
    let interactive_state = use_context::<Signal<InteractiveState>>();
    let chat_messages = use_context::<Signal<ChatMessages>>();

    let mut message_input = use_signal(|| String::new());

    rsx! {
        header {
            h1 { "Warhorse" }
            p { "A social backend for video games" }
        }
        wh_sidebar {}
        section { class: "main", 
            h2 { "Main" }
            div { class: "chat",
                for message in chat_messages.read().0.iter() {
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
                    let message = message_input.to_string();
                    wh.lock().unwrap().send_chat_message(message);

                    // Clears the input field
                    message_input.set(String::new());
                },
                input {
                    r#type: "text",
                    name: "message", 
                    placeholder: "Type a message...",
                    value: message_input.read().to_string(),
                    oninput: move |e| {
                        message_input.set(e.values().get("message").unwrap_or(&FormValue(vec![])).as_value());
                    }
                }
                button {
                    r#type: "submit",
                    "Send"
                }
            }
        }

        if *interactive_state.read() == InteractiveState::AddFriendModal {
            wh_add_friend_modal {}
        }
    
        if let InteractiveState::WhisperFriendModal(friend) = &*interactive_state.read() {
            wh_whisper_friend_modal { friend: friend.clone() }
        }
    
        if let InteractiveState::RemoveFriendModal(friend) = &*interactive_state.read() {
            wh_remove_friend_modal { friend: friend.clone() }
        }
    
        if let InteractiveState::BlockFriendModal(friend) = &*interactive_state.read() {
            wh_block_friend_modal { friend: friend.clone() }
        }

        if let InteractiveState::UnblockFriendModal(friend) = &*interactive_state.read() {
            wh_unblock_friend_modal { friend: friend.clone() }
        }

        if let InteractiveState::AcceptFriendRequestModal(friend) = &*interactive_state.read() {
            wh_accept_friend_request_modal { friend: friend.clone() }
        }

        if let InteractiveState::RejectFriendRequestModal(friend) = &*interactive_state.read() {
            wh_reject_friend_request_modal { friend: friend.clone() }
        }
    }
}

#[component]
fn wh_sidebar() -> Element {
    let friends_list = use_context::<Signal<FriendsList>>();
    let mut interactive_state = use_context::<Signal<InteractiveState>>();
    rsx! {
        section { class: "sidebar",
            div { class: "friends-container",
                h2 { "Friends" }
                div { class: "friends",
                    if let Some(friends) = friends_list.read().0.get(&FriendStatus::FriendRequestReceived) {
                        wh_friend_category { status: FriendStatus::FriendRequestReceived, friends: friends.clone() }
                    }

                    if let Some(friends) = friends_list.read().0.get(&FriendStatus::Online) {
                        wh_friend_category { status: FriendStatus::Online, friends: friends.clone() }
                    }

                    if let Some(friends) = friends_list.read().0.get(&FriendStatus::Offline) {
                        wh_friend_category { status: FriendStatus::Offline, friends: friends.clone() }
                    }

                    if let Some(friends) = friends_list.read().0.get(&FriendStatus::FriendRequestSent) {
                        wh_friend_category { status: FriendStatus::FriendRequestSent, friends: friends.clone() }
                    }

                    if let Some(friends) = friends_list.read().0.get(&FriendStatus::Blocked) {
                        wh_friend_category { status: FriendStatus::Blocked, friends: friends.clone() }
                    }
                }
                div { class: "add-friend-container",
                    button { 
                        class: "secondary add-friend",
                        onclick: move |_| *interactive_state.write() = InteractiveState::AddFriendModal,
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
        FriendStatus::FriendRequestSent => "Friend Requests Sent",
        FriendStatus::FriendRequestReceived => "Friend Requests Received",
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
    let mut interactive_state = use_context::<Signal<InteractiveState>>();
    let friend_id = friend.id.clone();
    rsx! {
        div { 
            class: "friend",
            onclick: move |_| *interactive_state.write() = InteractiveState::FriendContextMenu(friend_id.clone()),
            span { class: "friend-name", "{friend.display_name}" }
            span { class: "friend-menu", "⋮" }
        }
        if match &*interactive_state.read() {
            InteractiveState::FriendContextMenu(id) if *id == friend_id => true,
            _ => false
        } {
            wh_friend_context_menu { friend: friend.clone() }
        }
    }
}

#[component]
fn wh_friend_context_menu(friend: Friend) -> Element {
    let mut interactive_state = use_context::<Signal<InteractiveState>>();
    let friend_clone = friend.clone();
    let friend_clone2 = friend.clone();
    let friend_clone3 = friend.clone();
    let friend_clone4 = friend.clone();
    let friend_clone5 = friend.clone();
    rsx! {
        div {
            class: "friend-context-menu",
            if friend.status != FriendStatus::Blocked {
                button {
                    onclick: move |e| {
                        e.stop_propagation();
                        *interactive_state.write() = InteractiveState::WhisperFriendModal(friend_clone.clone());
                    },
                    "Whisper"
                }
                button {
                    class: "secondary",
                    onclick: move |e| {
                        e.stop_propagation();
                        *interactive_state.write() = InteractiveState::BlockFriendModal(friend_clone2.clone());
                    },
                    "Block"
                }
            }
            if friend.status == FriendStatus::Blocked {
                button {
                    class: "secondary",
                    onclick: move |e| {
                        e.stop_propagation();
                        *interactive_state.write() = InteractiveState::UnblockFriendModal(friend_clone3.clone());
                    },
                    "Unblock"
                }
            }
            if friend.status == FriendStatus::FriendRequestReceived {
                button {
                    class: "secondary",
                    onclick: move |e| {
                        e.stop_propagation();
                        *interactive_state.write() = InteractiveState::AcceptFriendRequestModal(friend_clone4.clone());
                    },
                    "Accept"
                }
                button {
                    class: "secondary",
                    onclick: move |e| {
                        e.stop_propagation();
                        *interactive_state.write() = InteractiveState::RejectFriendRequestModal(friend_clone5.clone());
                    },
                    "Reject"
                }
            }
            button {
                class: "secondary",
                onclick: move |e| {
                    e.stop_propagation();
                    *interactive_state.write() = InteractiveState::RemoveFriendModal(friend.clone());
                },
                "Remove"
            }
        }
        div {
            class: "friend-context-menu-backdrop",
            onclick: move |e| {
                e.stop_propagation();
                *interactive_state.write() = InteractiveState::Nothing;
            }
        }
    }
}

#[component]
fn wh_add_friend_modal() -> Element {
   let wh = use_context::<Arc<Mutex<Warhorse>>>();
   let mut interactive_state = use_context::<Signal<InteractiveState>>();
   rsx! {
       div { class: "modal",
            div { class: "modal-content",
                h2 { "Add Friend" }
                form { 
                    class: "add-friend-form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        *interactive_state.write() = InteractiveState::Nothing;
                        wh.lock().unwrap().send_friend_request(
                            e.values().get("friend_id").unwrap_or(&FormValue(vec![])).as_value()
                        );
                    },
                    input {
                        r#type: "text",
                        name: "friend_id",
                        placeholder: "Friend ID"
                    }
                    
                    button {
                        r#type: "submit",
                        "Request"
                    }
                }
            }
            div { class: "modal-buttons",
                button {
                    class: "secondary",
                    onclick: move |_| *interactive_state.write() = InteractiveState::Nothing,
                    "Close" 
                }
            }
       }
   }
}

#[component]
fn wh_block_friend_modal(friend: Friend) -> Element {
    let wh = use_context::<Arc<Mutex<Warhorse>>>();
    let mut interactive_state = use_context::<Signal<InteractiveState>>();
    rsx! {
        div { class: "modal",
            div { class: "modal-content",
                h2 { "Block Friend" }
                p { "Are you sure you want to block {friend.display_name}?" }
            }
            div { class: "modal-buttons",
                button {
                    class: "secondary",
                    onclick: move |_| *interactive_state.write() = InteractiveState::Nothing,
                    "Cancel"
                }
                button {
                    class: "danger",
                    onclick: move |_| {
                        wh.lock().unwrap().send_block_friend(friend.id.clone());
                        *interactive_state.write() = InteractiveState::Nothing;
                    },
                    "Block"
                }
            }
        }
   }
}

#[component]
fn wh_accept_friend_request_modal(friend: Friend) -> Element {
    let wh = use_context::<Arc<Mutex<Warhorse>>>();
    let mut interactive_state = use_context::<Signal<InteractiveState>>();
    rsx! {
        div { class: "modal",
            div { class: "modal-content",
                h2 { "Accept Friend Request" }
                p { "Are you sure you want to accept {friend.display_name}'s friend request?" }
            }
            div { class: "modal-buttons",
                button {
                    class: "secondary",
                    onclick: move |_| *interactive_state.write() = InteractiveState::Nothing,
                    "Cancel"
                }
                button {
                    class: "danger",
                    onclick: move |_| {
                        wh.lock().unwrap().send_accept_friend_request(friend.id.clone());
                        *interactive_state.write() = InteractiveState::Nothing;
                    },
                    "Accept"
                }
            }
        }
    }
}

#[component]
fn wh_reject_friend_request_modal(friend: Friend) -> Element {
    let wh = use_context::<Arc<Mutex<Warhorse>>>();
    let mut interactive_state = use_context::<Signal<InteractiveState>>();
    rsx! {
        div { class: "modal",
            div { class: "modal-content",
                h2 { "Reject Friend Request" }
                p { "Are you sure you want to reject {friend.display_name}'s friend request?" }
            }
            div { class: "modal-buttons",
                button {
                    class: "secondary",
                    onclick: move |_| *interactive_state.write() = InteractiveState::Nothing,
                    "Cancel"
                }
                button {
                    class: "danger",
                    onclick: move |_| {
                        wh.lock().unwrap().send_reject_friend_request(friend.id.clone());
                        *interactive_state.write() = InteractiveState::Nothing;
                    },
                    "Reject"
                }
            }
        }
    }
}

#[component]
fn wh_unblock_friend_modal(friend: Friend) -> Element {
    let wh = use_context::<Arc<Mutex<Warhorse>>>();
    let mut interactive_state = use_context::<Signal<InteractiveState>>();
    rsx! {
        div { class: "modal",
            div { class: "modal-content",
                h2 { "Unblock Friend" }
                p { "Are you sure you want to unblock {friend.display_name}?" }
            }
            div { class: "modal-buttons",
                button {
                    class: "secondary",
                    onclick: move |_| *interactive_state.write() = InteractiveState::Nothing,
                    "Cancel"
                }
                button {
                    class: "danger",
                    onclick: move |_| {
                        wh.lock().unwrap().send_unblock_friend(friend.id.clone());
                        *interactive_state.write() = InteractiveState::Nothing;
                    },
                    "Unblock"
                }
            }
        }
    }
}

#[component]
fn wh_remove_friend_modal(friend: Friend) -> Element {
    let wh = use_context::<Arc<Mutex<Warhorse>>>();
    let mut interactive_state = use_context::<Signal<InteractiveState>>();
    rsx! {
        div { class: "modal",
            div { class: "modal-content",
                h2 { "Remove Friend" }
                p { "Are you sure you want to remove {friend.display_name}?" }
            }
            div { class: "modal-buttons",
                button {
                    class: "secondary",
                    onclick: move |_| *interactive_state.write() = InteractiveState::Nothing,
                    "Cancel"
                }
                button {
                    class: "danger",
                    onclick: move |_| {
                        wh.lock().unwrap().send_remove_friend(friend.id.clone());
                        *interactive_state.write() = InteractiveState::Nothing;
                    },
                    "Remove"
                }
            }
        }
    }
}

#[component]
fn wh_whisper_friend_modal(friend: Friend) -> Element {
    let wh = use_context::<Arc<Mutex<Warhorse>>>();
    let mut interactive_state = use_context::<Signal<InteractiveState>>();
    rsx! {
        div { class: "modal",
            div { class: "modal-content",
                h2 { "Whisper to {friend.display_name}" }
                form { 
                    class: "whisper-form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        *interactive_state.write() = InteractiveState::Nothing;
                        wh.lock().unwrap().send_whisper_message(
                            friend.id.clone(),
                            e.values().get("message").unwrap_or(&FormValue(vec![])).as_value()
                        );
                    },
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
                    onclick: move |_| *interactive_state.write() = InteractiveState::Nothing,
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
