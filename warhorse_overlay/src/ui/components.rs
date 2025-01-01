use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use dioxus::prelude::*;
use tracing::{error, info};

use super::signals::*;
use warhorse_client::{warhorse_protocol::*, WarhorseClient, WarhorseEvent};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
pub fn app() -> Element {
    let wh = consume_context::<Arc<Mutex<WarhorseClient>>>();
    let wh_cloned = wh.clone();

    let mut notifications = use_signal(|| Notifications(Vec::new()));

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
    provide_context(notifications);

    // Periodically pump events from the Warhorse client
    use_future(move || {
        let wh_cloned = wh.clone();
        async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100)); // be nice to the cpu
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
                            notifications.write().0.push(Notification {
                                message: "You have successfully logged in".to_string(),
                                timestamp: Instant::now(),
                                notification_type: NotificationType::Generic,
                            });
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
                            notifications.write().0.push(Notification {
                                message: format!(
                                    "You have received a friend request from {}",
                                    friend.display_name
                                ),
                                timestamp: Instant::now(),
                                notification_type: NotificationType::FriendRequestReceived,
                            });
                        }
                        WarhorseEvent::FriendRequestAccepted(friend) => {
                            info!("Received FriendRequestAccepted event");
                            notifications.write().0.push(Notification {
                                message: format!(
                                    "{} has accepted your friend request",
                                    friend.display_name
                                ),
                                timestamp: Instant::now(),
                                notification_type: NotificationType::FriendAccepted,
                            });
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

    // TEMP:
    // Automatically login
    use_effect(move || {
        wh_cloned
            .lock()
            .unwrap()
            .send_user_login_request("test".into(), "password".into());
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        if !received_logged_in.read().0 {
            wh_login {}
        } else {
            wh_logged_in {}
        }
    }
}

#[component]
fn wh_notifications() -> Element {
    let notifications = use_context::<Signal<Notifications>>();
    let mut active_notifs = use_signal(Vec::new);

    use_effect(move || {
        active_notifs.set(notifications.read().0.clone());
    });

    // delete notifications older than 7 seconds
    use_future(move || async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let now = Instant::now();
            let current = active_notifs.read().clone();
            let filtered = current
                .iter()
                .filter(|n| now.duration_since(n.timestamp).as_secs() < 7)
                .cloned()
                .collect::<Vec<_>>();
            active_notifs.set(filtered);
        }
    });

    rsx! {
        div { class: "notifications",
            for notification in active_notifs.read().iter() {
                wh_notification { notification: notification.clone() }
            }
        }
    }
}

#[component]
fn wh_notification(notification: Notification) -> Element {
    let mut notifications = use_context::<Signal<Notifications>>();

    rsx! {
        div { class: "notification",
            div { class: "notification-message animate-fade-in animate-slide-in",
                "{notification.message}"
            }
            button {
                class: "notification-close",
                onclick: move |_| {
                    notifications.write().0.retain(|n| n != &notification);
                },
                "×"
            }
        }
    }
}

#[component]
fn wh_login() -> Element {
    let received_hello = use_context::<Signal<ReceivedHello>>();
    let wh_cloned = use_context::<Arc<Mutex<WarhorseClient>>>();
    let wh_cloned2 = wh_cloned.clone();

    rsx! {
        if received_hello.read().0 {
            header { class: "container mx-auto px-4",
                h1 { "Warhorse" }
            }
            section { class: "login-section",
                h2 { class: "login-title", "LOGIN" }
                form {
                    class: "login-form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        if let Err(e) = wh_cloned
                            .lock()
                            .unwrap()
                            .send_user_login_request(
                                e.values().get("username").unwrap_or(&FormValue(vec![])).as_value(),
                                e.values().get("password").unwrap_or(&FormValue(vec![])).as_value(),
                            )
                        {
                            error!("Failed to send login request: {:?}", e);
                        }
                    },
                    div {
                        input {
                            class: "login-input",
                            r#type: "text",
                            name: "username",
                            placeholder: "Account or Email",
                        }
                    }
                    div {
                        input {
                            class: "login-input",
                            r#type: "password",
                            name: "password",
                            placeholder: "Password",
                        }
                    }
                    button { class: "login-button", r#type: "submit", "EXECUTE LOGIN" }
                }

                div { class: "login-divider" }

                h2 { class: "login-title", "REGISTER" }
                form {
                    class: "login-form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        if let Err(e) = wh_cloned2
                            .lock()
                            .unwrap()
                            .send_user_registration_request(
                                e.values().get("account_name").unwrap_or(&FormValue(vec![])).as_value(),
                                e.values().get("password").unwrap_or(&FormValue(vec![])).as_value(),
                                e.values().get("display_name").unwrap_or(&FormValue(vec![])).as_value(),
                                e.values().get("email").unwrap_or(&FormValue(vec![])).as_value(),
                            )
                        {
                            error!("Failed to send registration request: {:?}", e);
                        }
                    },
                    div {
                        input {
                            class: "login-input",
                            r#type: "text",
                            name: "account_name",
                            placeholder: "Account Name",
                        }
                    }
                    div {
                        input {
                            class: "login-input",
                            r#type: "text",
                            name: "display_name",
                            placeholder: "Display Name",
                        }
                    }
                    div {
                        input {
                            class: "login-input",
                            r#type: "text",
                            name: "email",
                            placeholder: "Email",
                        }
                    }
                    div {
                        input {
                            class: "login-input",
                            r#type: "password",
                            name: "password",
                            placeholder: "Password",
                        }
                    }
                    button { class: "login-button", r#type: "submit", "INITIALIZE ACCOUNT" }
                }
            }
        } else {
            section { class: "loading-container",
                div { class: "loading-box",
                    h2 { class: "loading-text",
                        span { class: "loading-cursor", ">" }
                        "ESTABLISHING CONNECTION..."
                    }
                }
            }
        }
    }
}

#[component]
fn wh_logged_in() -> Element {
    let interactive_state = use_context::<Signal<InteractiveState>>();

    rsx! {
        div { class: "main-container",
            wh_sidebar {}
            wh_chat {}
        }
        wh_notifications {}

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
fn wh_title_header() -> Element {
    rsx! {
        header { class: "title-header",
            h1 { "Warhorse" }
        }
    }
}

#[component]
fn wh_sidebar() -> Element {
    let friends_list = use_context::<Signal<FriendsList>>();
    let mut interactive_state = use_context::<Signal<InteractiveState>>();
    rsx! {
        section { class: "sidebar",
            h2 { "Friends" }
            div { class: "friends-container",

                // add fake friend category
                wh_friend_category {
                    status: FriendStatus::Online,
                    friends: {
                        let mut friends = Vec::new();
                        for i in 0..10 {
                            let friend = Friend {
                                id: i.to_string(),
                                display_name: format!("Friend {}", i),
                                status: FriendStatus::Online,
                            };
                            friends.push(friend);
                        }
                        friends
                    },
                }

                if let Some(friends) = friends_list.read().0.get(&FriendStatus::FriendRequestReceived) {
                    wh_friend_category {
                        status: FriendStatus::FriendRequestReceived,
                        friends: friends.clone(),
                    }
                }

                if let Some(friends) = friends_list.read().0.get(&FriendStatus::Online) {
                    wh_friend_category {
                        status: FriendStatus::Online,
                        friends: friends.clone(),
                    }
                }

                if let Some(friends) = friends_list.read().0.get(&FriendStatus::Offline) {
                    wh_friend_category {
                        status: FriendStatus::Offline,
                        friends: friends.clone(),
                    }
                }

                if let Some(friends) = friends_list.read().0.get(&FriendStatus::FriendRequestSent) {
                    wh_friend_category {
                        status: FriendStatus::FriendRequestSent,
                        friends: friends.clone(),
                    }
                }

                if let Some(friends) = friends_list.read().0.get(&FriendStatus::Blocked) {
                    wh_friend_category {
                        status: FriendStatus::Blocked,
                        friends: friends.clone(),
                    }
                }
            }
            button {
                class: "add-friend",
                onclick: move |_| *interactive_state.write() = InteractiveState::AddFriendModal,
                "Add Friend"
            }
        }
    }
}

#[component]
fn wh_chat() -> Element {
    let wh = use_context::<Arc<Mutex<WarhorseClient>>>();
    let chat_messages = use_context::<Signal<ChatMessages>>();

    let mut message_input = use_signal(|| String::new());

    rsx! {
        section { class: "content",
            h2 { class: "chat-header", "Chat: #general" }
            div { class: "chat",
                div { class: "chat-messages",
                    // dummy message
                    wh_chat_message {
                        display_name: "Warhorse".to_string(),
                        time: "12:00".to_string(),
                        message: "Welcome to Warhorse!".to_string(),
                    }
                    for message in chat_messages.read().0.iter() {
                        wh_chat_message {
                            display_name: message.display_name.clone(),
                            time: message.time.to_string(),
                            message: message.message.clone(),
                        }
                    }
                }
                form {
                    class: "chat-form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        let message = message_input.to_string();
                        if let Err(e) = wh.lock().unwrap().send_room_message("general".into(), message) {
                            error!("Failed to send room message: {:?}", e);
                        }
                        message_input.set(String::new());
                    },
                    input {
                        r#type: "text",
                        name: "message",
                        placeholder: "Type a message...",
                        value: message_input.read().to_string(),
                        oninput: move |e| {
                            message_input
                                .set(e.values().get("message").unwrap_or(&FormValue(vec![])).as_value());
                        },
                    }
                    button { r#type: "submit", "Send" }
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
            onclick: move |_| {
                *interactive_state.write() = InteractiveState::FriendContextMenu(
                    friend_id.clone(),
                );
            },
            span { class: "friend-name", "{friend.display_name}" }
            span { class: "friend-menu", "⋮" }
        }
        if match &*interactive_state.read() {
            InteractiveState::FriendContextMenu(id) if *id == friend_id => true,
            _ => false,
        }
        {
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
        div { class: "friend-context-menu",

            if friend.status == FriendStatus::Online {
                button {
                    onclick: move |e| {
                        e.stop_propagation();
                        *interactive_state.write() = InteractiveState::WhisperFriendModal(
                            friend_clone.clone(),
                        );
                    },
                    "Whisper"
                }
            }

            if friend.status != FriendStatus::Blocked {
                button {
                    class: "secondary",
                    onclick: move |e| {
                        e.stop_propagation();
                        *interactive_state.write() = InteractiveState::BlockFriendModal(
                            friend_clone2.clone(),
                        );
                    },
                    "Block"
                }
            }

            if friend.status == FriendStatus::Blocked {
                button {
                    class: "secondary",
                    onclick: move |e| {
                        e.stop_propagation();
                        *interactive_state.write() = InteractiveState::UnblockFriendModal(
                            friend_clone3.clone(),
                        );
                    },
                    "Unblock"
                }
            }

            if friend.status == FriendStatus::FriendRequestReceived {
                button {
                    class: "secondary",
                    onclick: move |e| {
                        e.stop_propagation();
                        *interactive_state.write() = InteractiveState::AcceptFriendRequestModal(
                            friend_clone4.clone(),
                        );
                    },
                    "Accept"
                }
                button {
                    class: "secondary",
                    onclick: move |e| {
                        e.stop_propagation();
                        *interactive_state.write() = InteractiveState::RejectFriendRequestModal(
                            friend_clone5.clone(),
                        );
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
            },
        }
    }
}

#[component]
fn wh_add_friend_modal() -> Element {
    let wh = use_context::<Arc<Mutex<WarhorseClient>>>();
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
                        if let Err(e) = wh
                            .lock()
                            .unwrap()
                            .send_friend_request(
                                e.values().get("friend_id").unwrap_or(&FormValue(vec![])).as_value(),
                            )
                        {
                            error!("Failed to send friend request: {:?}", e);
                        }
                    },
                    input {
                        r#type: "text",
                        name: "friend_id",
                        placeholder: "Friend ID",
                    }
                    button { r#type: "submit", "Request" }
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
    let wh = use_context::<Arc<Mutex<WarhorseClient>>>();
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
                        if let Err(e) = wh.lock().unwrap().send_block_friend(friend.id.clone()) {
                            error!("Failed to block friend: {:?}", e);
                        }
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
    let wh = use_context::<Arc<Mutex<WarhorseClient>>>();
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
                        if let Err(e) = wh.lock().unwrap().send_accept_friend_request(friend.id.clone())
                        {
                            error!("Failed to accept friend request: {:?}", e);
                        }
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
    let wh = use_context::<Arc<Mutex<WarhorseClient>>>();
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
                        if let Err(e) = wh.lock().unwrap().send_reject_friend_request(friend.id.clone())
                        {
                            error!("Failed to reject friend request: {:?}", e);
                        }
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
    let wh = use_context::<Arc<Mutex<WarhorseClient>>>();
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
                        if let Err(e) = wh.lock().unwrap().send_unblock_friend(friend.id.clone()) {
                            error!("Failed to unblock friend: {:?}", e);
                        }
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
    let wh = use_context::<Arc<Mutex<WarhorseClient>>>();
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
                        if let Err(e) = wh.lock().unwrap().send_remove_friend(friend.id.clone()) {
                            error!("Failed to remove friend: {:?}", e);
                        }
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
    let wh = use_context::<Arc<Mutex<WarhorseClient>>>();
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
                        if let Err(e) = wh
                            .lock()
                            .unwrap()
                            .send_whisper_message(
                                friend.id.clone(),
                                e.values().get("message").unwrap_or(&FormValue(vec![])).as_value(),
                            )
                        {
                            error!("Failed to send whisper message: {:?}", e);
                        }
                    },
                    input {
                        r#type: "text",
                        name: "message",
                        placeholder: "Type a message...",
                    }
                    button { r#type: "submit", "Send" }
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
