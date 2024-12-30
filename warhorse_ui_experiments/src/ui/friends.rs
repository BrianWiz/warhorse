use bevy::prelude::{Query, Res, ResMut};
use bevy_egui::egui::*;
use bevy_egui::EguiContext;
use warhorse_protocol::{Friend, FriendStatus};
use crate::ui::Database;

pub fn ui_system(
    mut contexts: Query<&mut EguiContext>,
    mut database: ResMut<Database>
) {
    let mut selected_tab = database.friends.selected_tab;
    for mut context in contexts.iter_mut() {
        let ctx = context.get_mut();

        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {

                ui.horizontal(|ui| {
                    let style = Style {
                        spacing: Spacing {
                            item_spacing: vec2(5.0, 0.0),
                            ..Spacing::default()
                        },
                        ..Style::default()
                    };
                    ui.set_style(style);

                    // Friends Tab
                    if ui.add(
                        Button::new("Friends")
                            .selected(selected_tab == 0)
                            .min_size(vec2(100.0, 32.0))
                    ).clicked() {
                        selected_tab = 0;
                    }

                    // Friend Requests Tab
                    if ui.add(
                        Button::new("Friend Requests")
                            .selected(selected_tab == 1)
                            .min_size(vec2(100.0, 32.0))
                    ).clicked() {
                        selected_tab = 1;
                    }

                    // Blocked Tab
                    if ui.add(
                        Button::new("Blocked")
                            .selected(selected_tab == 2)
                            .min_size(vec2(100.0, 32.0))
                    ).clicked() {
                        selected_tab = 2;
                    }
                });

                // Content
                match selected_tab {
                    0 => {
                        ui.vertical(|ui| {
                            for friend in &database.friends.friends {
                                friend_card(ui, friend);
                            }
                        });
                    },
                    1 => {
                        // add search form for friend requests
                        ui.vertical(|ui| {
                            
                        });
                        // list friend requests
                        ui.vertical(|ui| {
                            for friend in &database.friends.friend_requests {
                                friend_card(ui, friend);
                            }
                        });
                    },
                    2 => {
                        ui.vertical(|ui| {
                            for friend in &database.friends.blocked {
                                friend_card(ui, friend);
                            }
                        });
                    },
                    _ => unreachable!()
                }
            });
    }

    database.friends.selected_tab = selected_tab;
}

fn friend_card(ui: &mut Ui, friend: &Friend) {
    ui.vertical(|ui| {
        ui.label(friend.display_name.clone());
        ui.label(status_to_string(&friend.status));
    });
}

fn status_to_string(status: &FriendStatus) -> String {
    match status {
        FriendStatus::Online => "Online".to_string(),
        FriendStatus::Offline => "Offline".to_string(),
        FriendStatus::PendingRequest => "Pending Request".to_string(),
        FriendStatus::Blocked => "Blocked".to_string(),
    }
}

// use bevy::prelude::*;
//
// #[derive(Debug, PartialEq, Clone, Copy)]
// enum FriendsListTab {
//     Friends,
//     FriendRequests,
//     Blocked,
// }
//
// #[derive(Component)]
// struct FriendId(String);
//
// #[derive(Component)]
// struct TabId(FriendsListTab);
//
// #[derive(Component)]
// struct FriendsListWidget;
//
// #[derive(Component)]
// struct FriendsListTabsContainer;
//
// #[derive(Component)]
// struct FriendsListContentContainer;
//
// #[derive(Component)]
// struct ActiveTab;
//
// // New resource to track the current tab
// #[derive(Resource)]
// struct CurrentTab(FriendsListTab);
//
// struct Friend {
//     id: String,
//     name: String,
//     status: String,
// }
//
// pub struct FriendsListPlugin;
// impl Plugin for FriendsListPlugin {
//     fn build(&self, app: &mut App) {
//         app.init_resource::<CurrentTab>()
//             .add_systems(Startup, spawn_system)
//             .add_systems(
//                 Update,
//                 (
//                     tab_interaction_system,
//                     friend_interaction_system,
//                     update_tab_content,
//                 ),
//             );
//     }
// }
//
// impl Default for CurrentTab {
//     fn default() -> Self {
//         CurrentTab(FriendsListTab::Friends)
//     }
// }
//
// fn spawn_system(mut commands: Commands) {
//     commands.spawn((
//         FriendsListWidget,
//         Node {
//             width: Val::Auto,
//             min_width: Val::Px(200.0),
//             flex_direction: FlexDirection::Column,
//             ..default()
//         },
//         BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
//     ))
//         .with_children(|parent| {
//             // Spawn tabs container
//             parent.spawn((
//                 FriendsListTabsContainer,
//                 Node {
//                     width: Val::Auto,
//                     flex_direction: FlexDirection::Row,
//                     ..default()
//                 },
//                 BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
//             ))
//                 .with_children(|parent| {
//                     parent.spawn((
//                         TabId(FriendsListTab::Friends),
//                         tab_button("Friends"),
//                         active_tab(),
//                     ));
//                     parent.spawn((
//                         TabId(FriendsListTab::FriendRequests),
//                         tab_button("Friend Requests"),
//                     ));
//                     parent.spawn((TabId(FriendsListTab::Blocked), tab_button("Blocked")));
//                 });
//
//             // Spawn content container
//             parent.spawn((
//                 FriendsListContentContainer,
//                 Node {
//                     width: Val::Auto,
//                     flex_direction: FlexDirection::Column,
//                     ..default()
//                 },
//             ))
//                 .with_children(|parent| {
//                     spawn_tab_content(FriendsListTab::Friends, parent);
//                 });
//         });
// }
//
// fn tab_interaction_system(
//     mut current_tab: ResMut<CurrentTab>,
//     interaction_query: Query<
//         (&Interaction, &TabId),
//         (Changed<Interaction>, With<Button>),
//     >,
// ) {
//     for (interaction, tab_id) in interaction_query.iter() {
//         if matches!(interaction, Interaction::Pressed) {
//             current_tab.0 = tab_id.0;
//         }
//     }
// }
//
// fn update_tab_content(
//     current_tab: Res<CurrentTab>,
//     mut commands: Commands,
//     tabs_query: Query<(Entity, &TabId)>,
//     content_container_query: Query<Entity, With<FriendsListContentContainer>>,
// ) {
//     if !current_tab.is_changed() {
//         return;
//     }
//
//     // Update tab visuals
//     for (entity, tab_id) in tabs_query.iter() {
//         if tab_id.0 == current_tab.0 {
//             commands
//                 .entity(entity)
//                 .insert(active_tab());
//         } else {
//             commands
//                 .entity(entity)
//                 .remove::<ActiveTab>()
//                 .remove::<BackgroundColor>();
//         }
//     }
//
//     // Update content
//     if let Ok(container_entity) = content_container_query.get_single() {
//         if let Some(mut container) = commands.get_entity(container_entity) {
//             container.despawn_descendants();
//             container.with_children(|parent| {
//                 spawn_tab_content(current_tab.0, parent);
//             });
//         }
//     }
// }
//
// fn friend_interaction_system(
//     mut commands: Commands,
//     interaction_query: Query<
//         (Entity, &Interaction, &FriendId),
//         (Changed<Interaction>, With<Button>),
//     >,
// ) {
//     for (entity, interaction, friend_id) in interaction_query.iter() {
//         match interaction {
//             Interaction::Pressed => {
//                 println!("Friend pressed: {}", friend_id.0);
//             }
//             Interaction::Hovered => {
//                 if let Some(mut cmd) = commands.get_entity(entity) {
//                     cmd.insert(friend_button_bg_hovered());
//                 }
//             }
//             Interaction::None => {
//                 if let Some(mut cmd) = commands.get_entity(entity) {
//                     cmd.remove::<BackgroundColor>();
//                 }
//             }
//         }
//     }
// }
//
// fn spawn_tab_content(tab: FriendsListTab, parent: &mut ChildBuilder) {
//     match tab {
//         FriendsListTab::Friends => {
//             for friend in friends_data() {
//                 friend_button(&friend, parent);
//             }
//         }
//         FriendsListTab::FriendRequests => {
//             parent.spawn(Text::new("No pending friend requests"));
//         }
//         FriendsListTab::Blocked => {
//             parent.spawn(Text::new("No blocked users"));
//         }
//     }
// }
//
// fn friends_data() -> Vec<Friend> {
//     vec![
//         Friend {
//             id: "1".to_string(),
//             name: "Alice".to_string(),
//             status: "Online".to_string(),
//         },
//         Friend {
//             id: "2".to_string(),
//             name: "Bob".to_string(),
//             status: "Offline".to_string(),
//         },
//     ]
// }
//
// fn active_tab() -> impl Bundle {
//     (ActiveTab, BackgroundColor(Color::srgb(0.5, 0.1, 0.2)))
// }
//
// fn tab_button(text: &str) -> impl Bundle {
//     (
//         Button,
//         Node {
//             width: Val::Auto,
//             padding: UiRect {
//                 left: Val::Px(5.0),
//                 right: Val::Px(5.0),
//                 top: Val::Px(2.0),
//                 bottom: Val::Px(2.0),
//             },
//             justify_content: JustifyContent::Center,
//             align_items: AlignItems::Center,
//             ..default()
//         },
//         Text::new(text),
//         TextLayout {
//             justify: JustifyText::Center,
//             ..default()
//         },
//     )
// }
//
// fn friend_button_bg_hovered() -> impl Bundle {
//     BackgroundColor(Color::srgb(0.2, 0.2, 0.2))
// }
//
// fn friend_button(friend: &Friend, builder: &mut ChildBuilder) {
//     builder
//         .spawn((
//             Button,
//             FriendId(friend.id.clone()),
//             Node {
//                 width: Val::Percent(100.0),
//                 padding: UiRect {
//                     left: Val::Px(5.0),
//                     right: Val::Px(5.0),
//                     top: Val::Px(5.0),
//                     bottom: Val::Px(5.0),
//                     ..default()
//                 },
//                 flex_direction: FlexDirection::Column,
//                 ..default()
//             },
//         ))
//         .with_children(|parent| {
//             parent.spawn(Text::new(friend.name.clone()));
//             parent.spawn(Text::new(friend.status.clone()));
//         });
// }
