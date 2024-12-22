use std::sync::mpsc;
use std::thread;
use rust_socketio::Socket;
use tokio::runtime::Runtime;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;
use warhorse_client::error::ClientError;
use warhorse_client::{WarhorseClient, WarhorseClientCommands};
use warhorse_protocol::{Friend, Language, RequestError, UserPartial, UserRegistration};

fn main() -> Result<(), ClientError> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let language = Language::English;

    // Create channels for shutdown signal
    let (tx, rx) = mpsc::channel();
    let tx_clone = tx.clone();

    ctrlc::set_handler(move || {
        info!("Received Ctrl+C!");
        tx_clone.send(()).ok();
    }).ok();

    let _ = WarhorseClient::new(
        language,
        "http://localhost:3000",
        on_receive_hello,
         on_receive_request_error,
        on_receive_friends_list,
        on_receive_blocked_list,
        on_receive_friend_requests,
        on_receive_friend_request_accepted,
    );

    info!("Client is running. Press Ctrl+C to exit.");

    // Wait for shutdown signal
    rx.recv().unwrap_or(());
    info!("Shutting down...");
    Ok(())
}

fn on_receive_hello(client: &mut WarhorseClientCommands, language: Language) {
    info!("Received hello from server");
    client.user_registration = Some(UserRegistration {
        language,
        account_name: "test".to_string(),
        password: "test123456".to_string(),
        email: "test@example.com".to_string(),
        display_name: "SomeDisplayName".to_string(),
    });
}

fn on_receive_request_error(error_message: RequestError) {
    error!("Request error: {:?}", error_message);
}

fn on_receive_friends_list(friends_list: &Vec<Friend>) {
    info!("Friends list: {:?}", friends_list);
}

fn on_receive_blocked_list(blocked_list: &Vec<UserPartial>) {
    info!("Blocked list: {:?}", blocked_list);
}

fn on_receive_friend_requests(friend_requests: &Vec<Friend>) {
    info!("Friend requests: {:?}", friend_requests);
}

fn on_receive_friend_request_accepted(friend: &Friend) {
    info!("Friend request accepted: {:?}", friend);
}