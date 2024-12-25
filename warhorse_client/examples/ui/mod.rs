mod tabs;

use bevy::color::Color;
use bevy::hierarchy::ChildBuilder;
use bevy::prelude::*;
use crate::ui::tabs::{spawn_tabs, TabContent};
use crate::ui::tabs::systems::TabsPlugin;

pub struct WarhorseUIPlugin;
impl Plugin for WarhorseUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TabsPlugin::<FriendsListTab>::default());
        app.add_systems(
            Startup,
        setup_ui,
        );
    }
}


pub enum FriendsListState {
    Friends,
    FriendRequests,
    Blocked,
}


#[derive(Component, Clone)]
pub enum FriendsListTab {
    Friends,
    FriendRequests,
    Blocked,
}

impl TabContent for FriendsListTab {
    type Content = Text;
    fn create_content(world: &mut World) -> Self::Content {
        match world.query::<&FriendsListTab>().iter(world).next() {
            Some(tab) => {
                let text = match tab {
                    FriendsListTab::Friends => "Friends",
                    FriendsListTab::FriendRequests => "Friend Requests",
                    FriendsListTab::Blocked => "Blocked",
                };
                Text::new(text)
            }
            None => Text::new(""),
        }
    }
}

#[derive(Component)]
pub struct FriendsList;

pub fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2d::default());
    spawn_friends_list(commands);
}

fn spawn_friends_list(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Auto,
            min_width: Val::Px(200.0),
            padding: UiRect {
                left: Val::Px(10.0),
                right: Val::Px(10.0),
                top: Val::Px(10.0),
                bottom: Val::Px(10.0),
            },
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
    ))
        .with_children(|parent| {
            friends_list_tabs(parent);
        });
}

fn friends_list_tabs(builder: &mut ChildBuilder) {
    spawn_tabs(
        builder,
        FriendsListTab::Friends,
        vec![
            ("Friends".to_string(), FriendsListTab::Friends),
            ("Friend Requests".to_string(), FriendsListTab::FriendRequests),
            ("Blocked".to_string(), FriendsListTab::Blocked),
        ],
        0,
    )
}

