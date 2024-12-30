mod ui;

use std::time::{Duration};
use bevy::prelude::*;
use warhorse_client::{WarhorseClient, WarhorseEvent};
use warhorse_client::error::ClientError;
use warhorse_protocol::{ChatMessage, Friend, Language};
use ui::WarhorseUIPlugin;

#[derive(Event, Default)]
struct FriendsDataChanged;

#[derive(Component)]
pub struct WarhorseFriend(pub Friend);

#[derive(Component)]
pub struct WarhorseBlockedUser(pub Friend);

#[derive(Component)]
pub struct WarhorseFriendRequest(pub Friend);

#[derive(Component)]
pub struct WarhorseChatMessage(pub ChatMessage);

pub enum WarhorseNotificationKind {
    Error,
    Info,
}

#[derive(Component)]
pub struct WarhorseNotification {
    pub message: String,
    pub kind: WarhorseNotificationKind,
    pub lifetime: Timer,
}

#[derive(Resource)]
pub struct WarhorseLoggedIn;

#[derive(Resource)]
pub struct BevyWarhorseClient {
    warhorse_client: WarhorseClient,
}

impl BevyWarhorseClient {
    pub fn new(language: Language, connection_string: &str) -> Result<Self, ClientError> {
        let warhorse_client = WarhorseClient::new(
            language,
            connection_string,
        )?;

        Ok(Self {
            warhorse_client,
        })
    }
}

pub struct BevyWarhorsePlugin;
impl Plugin for BevyWarhorsePlugin {
    fn build(&self, app: &mut App) {

        app.add_plugins(WarhorseUIPlugin);
        app.add_systems(
            PreUpdate,
            (
                poll_events,
                update_notifications,
            ),
        );
        app.add_event::<FriendsDataChanged>();

        match BevyWarhorseClient::new(
            Language::English,
            "http://localhost:3000",
        ) {
            Ok(client) => {
                app.insert_resource(client);
            }
            Err(e) => {
                error!("Error creating Warhorse client: {:?}", e);
            }
        }
    }
}

fn update_notifications(
    time: Res<Time>,
    mut commands: Commands,
    mut q_notifications: Query<(Entity, &mut WarhorseNotification)>,
) {
    for (entity, mut notification) in q_notifications.iter_mut() {
        notification.lifetime.tick(time.delta());
        if notification.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn poll_events(
    client: ResMut<BevyWarhorseClient>,
    mut friends_data_changed_event_writer: EventWriter<FriendsDataChanged>,
    mut commands: Commands,
    mut q_blocked: Query<(Entity, &WarhorseBlockedUser)>,
    mut q_friends: Query<(Entity, &WarhorseFriend)>,
    mut q_friend_requests: Query<(Entity, &WarhorseFriendRequest)>,
) {
    for event in client.warhorse_client.pump() {
        match event {
            WarhorseEvent::Hello => {
                // the server has fake data so we can just try logging in as one of the fake users for now
                let account_name = "test";
                let password = "password".into();

                if let Err(e) = client.warhorse_client.send_user_login_request(
                    warhorse_protocol::UserLogin {
                        language: Language::English,
                        identity: warhorse_protocol::LoginUserIdentity::AccountName(account_name.into()),
                        password,
                    }
                ) {
                    error!("Error sending login request: {:?}", e);
                }
            }
            WarhorseEvent::LoggedIn => {
                commands.insert_resource(WarhorseLoggedIn);
                if let Err(e) = client.warhorse_client.send_friend_request("1") {
                    error!("Error sending friend request: {:?}", e);
                }
            }
            WarhorseEvent::Error(error_msg) => {
                commands.spawn(WarhorseNotification {
                    message: error_msg,
                    kind: WarhorseNotificationKind::Error,
                    lifetime: Timer::new(Duration::from_secs(5), TimerMode::Once),
                });
            }
            WarhorseEvent::BlockedList(blocked) => {
                // delete all existing blocked users
                for entity in q_blocked.iter_mut() {
                    commands.entity(entity.0).despawn();
                }

                // spawn new blocked users
                for blocked_user in blocked {
                    commands.spawn(WarhorseBlockedUser(blocked_user));
                }

                friends_data_changed_event_writer.send(FriendsDataChanged);
            }
            WarhorseEvent::FriendsList(friends) => {
                // delete all existing friends
                for entity in q_friends.iter_mut() {
                    commands.entity(entity.0).despawn();
                }

                // spawn new friends
                for friend in friends {
                    commands.spawn(WarhorseFriend(friend));
                }

                friends_data_changed_event_writer.send(FriendsDataChanged);
            }
            WarhorseEvent::FriendRequests(requests) => {
                // delete all existing friend requests
                for (entity, _) in q_friend_requests.iter_mut() {
                    commands.entity(entity).despawn();
                }

                // spawn new friend requests
                for request in requests {
                    commands.spawn(WarhorseFriendRequest(request));
                }

                friends_data_changed_event_writer.send(FriendsDataChanged);
            }
            WarhorseEvent::FriendRequestAccepted(friend) => {
                commands.spawn(WarhorseNotification {
                    message: format!("Friend request accepted: {}", friend.display_name),
                    kind: WarhorseNotificationKind::Info,
                    lifetime: Timer::new(Duration::from_secs(5), TimerMode::Once),
                });
            }
            WarhorseEvent::ChatMessage(message) => {
                commands.spawn(WarhorseChatMessage(message));
            }
        }
    }
}
