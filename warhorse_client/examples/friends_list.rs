use std::sync::mpsc;
use std::thread;
use tokio::runtime::Runtime;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;
use warhorse_client::error::ClientError;
use warhorse_client::WarhorseClient;
use warhorse_protocol::{Friend, RequestError, UserPartial, UserRegistration};

fn main() -> Result<(), ClientError> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let language = warhorse_protocol::Language::English;

    // Create channels for shutdown signal
    let (tx, rx) = mpsc::channel();
    let tx_clone = tx.clone();

    ctrlc::set_handler(move || {
        info!("Received Ctrl+C!");
        tx_clone.send(()).ok();
    }).ok();

    // Initialize client in the main thread
    let client = WarhorseClient::new(
        language,
        "http://localhost:3000",
         on_receive_request_error,
        on_receive_friends_list,
        on_receive_blocked_list,
        on_receive_friend_requests,
        on_receive_friend_request_accepted,
    );

    // Spawn a separate thread for async operations if needed
    let _async_handle = thread::spawn(|| {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // Your async operations here
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                // Do something periodically...
            }
        });
    });

    // Debug: Wait for the client to connect
    thread::sleep(std::time::Duration::from_secs(2));

    // Send registration request
    if let Err(e) = client.send_user_registration_request(UserRegistration {
        language: language.clone(),
        account_name: "test".to_string(),
        password: "test123456".to_string(),
        email: "test@example.com".to_string(),
        display_name: "SomeDisplayName".to_string(),
    }) {
        error!("Failed to send user registration request: {:?}", e);
    } else {
        info!("User registration request sent.");
    }

    info!("Client is running. Press Ctrl+C to exit.");

    // Wait for shutdown signal
    rx.recv().unwrap_or(());

    info!("Shutting down...");
    Ok(())
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