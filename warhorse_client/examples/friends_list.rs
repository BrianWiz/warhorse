use std::sync::{mpsc, Arc, RwLock};
use clap::Parser;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;
use warhorse_client::error::ClientError;
use warhorse_client::{WarhorseClient, WarhorseEvent};
use warhorse_protocol::{FriendRequest, Language, LoginUserIdentity, RequestError, UserLogin, UserPartial};

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "test")]
    account_name: String,
}

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

    let client = Arc::new(RwLock::new(WarhorseClient::new(language, "http://localhost:3000")));
    info!("Client is running. Press Ctrl+C to exit.");

    // Main event loop
    loop {
        // Check for shutdown signal
        if rx.try_recv().is_ok() {
            info!("Shutting down...");
            break;
        }

        if let Ok(client_write) = client.clone().write() {
           for event in client_write.pump() {
                match event {
                    WarhorseEvent::Hello => {
                        info!("Received hello from server");

                        // the server has fake data so we can just try logging in as one of the fake users for now
                        let args = Args::parse();
                        info!("Using account name: {}", args.account_name);
                        let password = "password".into();

                        if let Err(e) = client_write.send_user_login_request(UserLogin {
                            identity: LoginUserIdentity::AccountName(args.account_name.clone()),
                            password,
                            language,
                        }) {
                            error!("Error sending login request: {:?}", e);
                        }
                    }
                    WarhorseEvent::LoggedIn => {
                        match Args::parse().account_name.as_str() {
                            "test" => {
                                if let Err(e) = client_write.send_friend_request("1") {
                                    error!("Error sending friend request: {:?}", e);
                                }
                            }
                            "test2" => {
                                if let Err(e) = client_write.send_friend_request("0") {
                                    error!("Error sending friend request: {:?}", e);
                                }
                            }
                            _ => {}
                        }
                    }
                    WarhorseEvent::Error(error_msg) => {
                        error!("Received error: {}", error_msg);
                    }
                    WarhorseEvent::FriendRequests(requests) => {
                        info!("Received friend requests: {:?}", requests);
                    }
                    WarhorseEvent::FriendsList(friends) => {
                        info!("Received friends list: {:?}", friends);
                    }
                    WarhorseEvent::BlockedList(blocked) => {
                        info!("Received blocked list: {:?}", blocked);
                    }
                    WarhorseEvent::FriendRequestAccepted(friend) => {
                        info!("Received friend request accepted: {:?}", friend);
                    }
                    WarhorseEvent::ChatMessage(message) => {
                        info!("Received chat message: {:?}", message);
                    }
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(())
}

