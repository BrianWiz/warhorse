use std::sync::mpsc;
use clap::Parser;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;
use warhorse_client::error::ClientError;
use warhorse_client::{WarhorseClient, WarhorseClientCommands};
use warhorse_protocol::{Friend, Language, LoginUserIdentity, RequestError, UserLogin, UserPartial, UserRegistration};

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

    // the server has fake data so we can just try logging in as one of the fake users for now
    let args = Args::parse();
    info!("Using account name: {}", args.account_name);
    let password = "password".into();

    client.user_login = Some(UserLogin {
        identity: LoginUserIdentity::AccountName(args.account_name),
        password,
        language,
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
