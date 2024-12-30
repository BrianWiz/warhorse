mod utils;

use std::sync::atomic::Ordering;
use std::time::Duration;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::AtomicBool;
use tauri::Emitter;
use tracing::{error, info};
use warhorse_client::*;
use warhorse_client::warhorse_protocol::*;

type AppState<'a> = tauri::State<'a, Arc<Mutex<WarhorseApp>>>;

struct WarhorseApp {
    client: RwLock<WarhorseClient>,
    running: Arc<AtomicBool>,
    friends: Mutex<Vec<Friend>>,
    received_hello: bool,
    received_logged_in: bool,
}

impl WarhorseApp {
    fn new(connection_string: &str) -> Self {
        match WarhorseClient::new(Language::English, connection_string) {
            Ok(client) => {
                Self {
                    client: RwLock::new(client),
                    friends: Mutex::new(Vec::new()),
                    running: Arc::new(AtomicBool::new(true)),
                    received_hello: false,
                    received_logged_in: false,
                }
            },
            Err(e) => {
                panic!("Error creating WarhorseClient: {}", e);
            }
        }
    }

    fn set_friends(&self, app_handle: &tauri::AppHandle, friends: Vec<Friend>) {
        *self.friends.lock().unwrap() = friends;
        match app_handle.emit("friends_updated", (*self.friends.lock().unwrap()).clone()) {
            Ok(_) => info!("Successfully emitted friends_updated event"),
            Err(e) => error!("Error emitting friends_updated event: {} {:?}", e, e),
        }
    }

    fn emit_received_hello(&self, app_handle: &tauri::AppHandle) {
        match app_handle.emit("received_hello", true) {
            Ok(_) => info!("Successfully emitted received_hello event"),
            Err(e) => error!("Error emitting received_hello event: {} {:?}", e, e),
        }
    }

    fn emit_received_logged_in(&self, app_handle: &tauri::AppHandle) {
        match app_handle.emit("received_logged_in", true) {
            Ok(_) => info!("Successfully emitted received_logged_in event"),
            Err(e) => error!("Error emitting received_logged_in event: {} {:?}", e, e),
        }
    }
    
    fn emit_chat_message(&self, app_handle: &tauri::AppHandle, message: ChatMessage) {
        match app_handle.emit("chat_message", message) {
            Ok(_) => info!("Successfully emitted chat_message event"),
            Err(e) => error!("Error emitting chat_message event: {} {:?}", e, e),
        }
    }

    fn get_friends(&self) -> Vec<Friend> {
        self.friends.lock().unwrap().clone()
    }

    fn received_hello(&self) -> bool {
        self.received_hello
    }

    fn received_logged_in(&self) -> bool {
        self.received_logged_in
    }

    fn tick(&mut self, app_handle: &tauri::AppHandle) {
        let mut events_to_process = Vec::new();
        
        {
            if let Ok(client) = self.client.read() {
                events_to_process.extend(client.pump());
            }
        }
    
        for event in events_to_process {
            match event {
                WarhorseEvent::Hello => {
                    info!("Received hello event");
                    self.received_hello = true;
                    self.emit_received_hello(app_handle);
                },
                WarhorseEvent::Error(e) => {
                    error!("Warhorse error: {}", e);
                },
                WarhorseEvent::LoggedIn => {
                    info!("Received logged-in event");
                    self.received_logged_in = true;
                    self.emit_received_logged_in(app_handle);
                },
                WarhorseEvent::FriendsList(friends) => {
                    info!("Received friends-list event {:?}", friends);
                    self.set_friends(app_handle, friends);
                },
                WarhorseEvent::FriendRequests(friends) => {
                    info!("Received friend-requests event {:?}", friends);
                },
                WarhorseEvent::FriendRequestAccepted(friend) => {
                    info!("Received friend-request-accepted event {:?}", friend);
                },
                WarhorseEvent::ChatMessage(message) => {
                    info!("Received chat-message event {:?}", message);
                    self.emit_chat_message(app_handle, message);
                },
            }
        }
    }
}

#[tauri::command]
fn get_friends(app: AppState) -> Result<Vec<Friend>, String> {
    if let Ok(app) = app.lock() {
        Ok(app.get_friends())
    } else {
        Err("Failed to lock app state".to_string())
    }
}

#[tauri::command]
fn received_hello(app: AppState) -> Result<bool, String> {
    if let Ok(app) = app.lock() {
        Ok(app.received_hello())
    } else {
        Err("Failed to lock app state".to_string())
    }
}

#[tauri::command]
fn received_logged_in(app: AppState) -> Result<bool, String> {
    if let Ok(app) = app.lock() {
        Ok(app.received_logged_in())
    } else {
        Err("Failed to lock app state".to_string())
    }
}

#[tauri::command]
fn send_chat_message(
    app: AppState<'_>,
    message: String,
) -> Result<(), String> {
    if let Ok(app) = app.lock() {
        let client = app.client.write().unwrap();
        match client.send_chat_message(SendChatMessage {
            language: Language::English,
            channel: ChatChannel::Room("general".to_string()),
            message,
        }) {
            Ok(_) => {
                info!("Sent chat-message");
                Ok(())
            }
            Err(e) => {
                error!("Error sending chat-message: {}", e);
                Err(e.to_string())
            }
        }
    } else {
        Err("Failed to lock app state".to_string())
    }
}

#[tauri::command]
async fn login(
    app: AppState<'_>,
    username: String,
    password: String,
) -> Result<(), String> {
    if let Ok(app) = app.lock() {
        let client = app.client.write().unwrap();
        match client.send_user_login_request(UserLogin {
            language: Language::English,
            identity: match utils::is_username_or_email(&username) {
                utils::UserLoginType::AccountName => LoginUserIdentity::AccountName(username),
                utils::UserLoginType::Email => LoginUserIdentity::Email(username),
            },
            password,
        }) {
            Ok(_) => {
                info!("Sent user-login-request");
                Ok(())
            }
            Err(e) => {
                error!("Error sending user-login-request: {}", e);
                Err(e.to_string())
            }
        }
    } else {
        Err("Failed to lock app state".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    let app = Arc::new(Mutex::new(WarhorseApp::new("http://localhost:3000")));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app.clone())
        .setup(move |tauri_app| {
            let app_state = app.clone();
            let handle = tauri_app.handle().clone();

            std::thread::spawn(move || {
                while let Ok(mut app_state) = app_state.try_lock() {
                    if !app_state.running.load(Ordering::Relaxed) {
                        break;
                    }
                    
                    app_state.tick(&handle);
                    drop(app_state); // Explicitly release the lock
                    std::thread::sleep(Duration::from_millis(100));
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_friends,
            login,
            send_chat_message,
            received_hello,
            received_logged_in
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
