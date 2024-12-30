mod friends;
pub use bevy_egui;

use bevy::prelude::*;
use warhorse_protocol::*;

pub struct WarhorseUIPlugin;
impl Plugin for WarhorseUIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Database::new());
        app.add_systems(Update,friends::ui_system);
    }
}

struct FriendsDatabase {
    selected_tab: i32,
    friends: Vec<Friend>,
    friend_requests: Vec<Friend>,
    blocked: Vec<Friend>,
}

#[derive(Resource)]
struct Database {
    friends: FriendsDatabase,
}

impl Database {
    pub fn new() -> Self {
        Database {
            friends: FriendsDatabase {
                selected_tab: 0,
                friends: vec![
                    Friend {
                        id: "1".to_string(),
                        display_name: "Alice".to_string(),
                        status: FriendStatus::Online,
                    },
                    Friend {
                        id: "2".to_string(),
                        display_name: "Bob".to_string(),
                        status: FriendStatus::Offline,
                    },
                    Friend {
                        id: "3".to_string(),
                        display_name: "Charlie".to_string(),
                        status: FriendStatus::Online,
                    },
                ],
                friend_requests: vec![
                    Friend {
                        id: "4".to_string(),
                        display_name: "David".to_string(),
                        status: FriendStatus::Online,
                    },
                    Friend {
                        id: "5".to_string(),
                        display_name: "Eve".to_string(),
                        status: FriendStatus::Online,
                    },
                ],
                blocked: vec![
                    Friend {
                        id: "6".to_string(),
                        display_name: "Frank".to_string(),
                        status: FriendStatus::Blocked,
                    },
                ],
            },
        }
    }
}
