use bevy::prelude::App;

mod ui;

use bevy::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ui::WarhorseUIPlugin)
        .add_systems(
            Startup,
            spawn_camera,
        )
        .run();
    Ok(())
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}


// mod another;
//
// include!(concat!(env!("OUT_DIR"), "/generated/warhorse_ui_schema.rs"));
//
// use std::ops::Deref;
// use bevy::prelude::*;
// use warhorse_ui::Align;
// use warhorse_ui::serde_json::json;
//
// #[derive(Component, Clone)]
// pub struct Identifiable(pub String);
//
// impl Identifiable {
//     pub fn get(&self) -> &str {
//         &self.0
//     }
//
//     pub fn is(&self, other: &str) -> bool {
//         self.0 == other
//     }
// }
//
// #[derive(Component, Clone)]
// pub struct BelongsToDataSet(pub Option<String>);
//
// pub enum MainMenuSignal {
//     GoToSettings,
//     GoToFriendsList,
// }
//
// pub enum SettingsSignal {
//     Save,
//     GoBack,
// }
//
// pub enum FriendsListSignal {
//     GoBack,
//     VisitFriend(String),
// }
//
// #[derive(Event)]
// pub enum Signal {
//     MainMenu(MainMenuSignal),
//     Settings(SettingsSignal),
//     FriendsList(FriendsListSignal),
// }
//
// #[derive(Resource, Clone)]
// pub enum CurrentPage {
//     MainMenu,
//     Settings,
//     FriendsList,
// }
//
// #[derive(Component)]
// pub struct RootNode;
//
// #[derive(Resource)]
// pub struct AppWidgets {
//     pub main_menu: generated::Widget,
//     pub settings: generated::Widget,
//     pub friends_list: generated::Widget,
// }
//
// fn main() -> Result<(), Box<dyn std::error::Error>> {
//
//     let main_menu_ron = r#"
//         Row(
//             id: "main-container",
//             align: Center,
//             inner: [
//                 Label(
//                     id: "label",
//                     text: "Main Menu",
//                 ),
//                 Button(
//                     id: "settings-btn",
//                     text: "Settings",
//                 ),
//                 Button(
//                     id: "friends-list",
//                     text: "Friends List",
//                 ),
//             ]
//         )
//     "#;
//
//     let settings_ron = r#"
//         Row(
//             id: "settings-container",
//             align: Center,
//             inner: [
//                 Label(
//                     id: "label",
//                     text: "Settings",
//                 ),
//                 Button(
//                     id: "save-btn",
//                     text: "Save",
//                 ),
//                 Button(
//                     id: "go-back-btn",
//                     text: "Go back",
//                 ),
//             ]
//         )
//     "#;
//
//     let friends_list_ron = r#"
//         Row(
//             id: "friends-list-container",
//             align: Center,
//             inner: [
//                 Label(
//                     id: "label",
//                     text: "Friends List",
//                 ),
//                 ForEach("friends",
//                     PressableContainer(
//                         id: "replace-me",
//                         inner: [
//                             Column (
//                                 id: "friend",
//                                 align: Center,
//                                 inner: [
//                                     Label(
//                                         id: "friend",
//                                         text: "Unknown Friend",
//                                     )
//                                 ]
//                             )
//                         ]
//                     )
//                 ),
//                 Button(
//                     id: "go-back-btn",
//                     text: "Go back",
//                 ),
//             ]
//         )
//     "#;
//
//     let main_menu = ron::de::from_str::<generated::Widget>(main_menu_ron)?;
//     let settings = ron::de::from_str::<generated::Widget>(settings_ron)?;
//     let friends_list = ron::de::from_str::<generated::Widget>(friends_list_ron)?;
//
//     App::new()
//         .add_plugins(DefaultPlugins)
//         .add_systems(
//             Startup,
//             setup_ui,
//         )
//         .add_systems(
//             Update,
//             (
//                 listen_for_user_interaction,
//                 signal_listener,
//                 page_swapper,
//             ),
//         )
//         .add_event::<Signal>()
//         .insert_resource(AppWidgets {
//             main_menu,
//             settings,
//             friends_list,
//         })
//         .insert_resource(CurrentPage::MainMenu)
//         .run();
//     Ok(())
// }
//
// fn setup_ui(
//     mut commands: Commands,
//     app_widgets: Res<AppWidgets>,
//     current_page: Res<CurrentPage>,
//     mut query: Query<(Entity, &RootNode)>,
// ) {
//     commands.spawn(Camera2d::default());
//     hydrate_current_page(
//         &mut commands,
//         &app_widgets,
//         &current_page,
//         &mut query
//     );
// }
//
// fn hydrate(
//     builder: &mut ChildBuilder,
//     widget: &generated::Widget,
//     data: &warhorse_ui::serde_json::Value,
//     belongs_to_dataset: Option<String>,
// ) {
//     match widget {
//         generated::Widget::Row { id, align, inner } => {
//             builder.spawn((
//                 Identifiable(id.clone()),
//                 BelongsToDataSet(belongs_to_dataset),
//                 Node {
//                     width: Val::Percent(100.0),
//                     flex_direction: FlexDirection::Column,
//                     align_items: match align {
//                         Align::Start => AlignItems::FlexStart,
//                         Align::Center => AlignItems::Center,
//                         Align::End => AlignItems::FlexEnd,
//                         _=> AlignItems::Center,
//                     },
//                     ..default()
//                 }
//             )).with_children(|builder| {
//                 for child in inner {
//                     hydrate(builder, child, data, None);
//                 }
//             });
//         }
//         generated::Widget::Column { id, align, inner } => {
//             builder.spawn((
//                 Identifiable(id.clone()),
//                 BelongsToDataSet(belongs_to_dataset),
//                 Node {
//                     width: Val::Percent(100.0),
//                     flex_direction: FlexDirection::Row,
//                     align_items: match align {
//                         Align::Start => AlignItems::FlexStart,
//                         Align::Center => AlignItems::Center,
//                         Align::End => AlignItems::FlexEnd,
//                         _=> AlignItems::Center,
//                     },
//                     ..default()
//                 }
//             )).with_children(|builder| {
//                 for child in inner {
//                     hydrate(builder, child, data, None);
//                 }
//             });
//         }
//         generated::Widget::Label { id, text } => {
//             builder.spawn((
//                 Identifiable(id.clone()),
//                 BelongsToDataSet(belongs_to_dataset),
//                 Text::new(text)
//             ));
//         }
//         generated::Widget::PressableContainer { id, inner } => {
//             builder.spawn((
//                 Identifiable(id.clone()),
//                 BelongsToDataSet(belongs_to_dataset),
//                 Button,
//                 Node {
//                     width: Val::Auto,
//                     ..default()
//                 }
//             )).with_children(|builder| {
//                 for child in inner {
//                     hydrate(builder, child, data, None);
//                 }
//             });
//         }
//         generated::Widget::Button { id, text, .. } => {
//             builder.spawn((
//                 Button,
//                 Identifiable(id.clone()),
//                 BelongsToDataSet(belongs_to_dataset),
//                 Node {
//                     width: Val::Auto,
//                     ..default()
//                 },
//                 BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
//             ))
//                 .with_child(Text::new(text));
//         }
//         generated::Widget::ForEach(key, block) => {
//             data
//                 .get(key)
//                 .and_then(|v| v.as_array())
//                 .map(|items| {
//                     for item in items {
//                         let mut block = block.clone();
//                         block.feed_data(item);
//                         hydrate(
//                             builder,
//                             &block,
//                             item,
//                             Some(key.clone())
//                         );
//                     }
//                 });
//         }
//     }
// }
//
// fn listen_for_user_interaction(
//     query: Query<(&Interaction, &Identifiable), Changed<Interaction>>,
//     mut signal_writer: EventWriter<Signal>,
//     current_page: Res<CurrentPage>,
// ) {
//     for (interaction, identifiable) in query.iter() {
//         if *interaction == Interaction::Pressed {
//             match current_page.deref() {
//                 CurrentPage::MainMenu => {
//                     if identifiable.is("settings-btn") {
//                         signal_writer.send(Signal::MainMenu(MainMenuSignal::GoToSettings));
//                     } else if identifiable.is("friends-list") {
//                         signal_writer.send(Signal::MainMenu(MainMenuSignal::GoToFriendsList));
//                     }
//                 }
//                 CurrentPage::Settings => {
//                     if identifiable.is("save-btn") {
//                         signal_writer.send(Signal::Settings(SettingsSignal::Save));
//                     } else if identifiable.is("go-back-btn") {
//                         signal_writer.send(Signal::Settings(SettingsSignal::GoBack));
//                     }
//                 }
//                 CurrentPage::FriendsList => {
//                     if identifiable.is("go-back-btn") {
//                         signal_writer.send(Signal::FriendsList(FriendsListSignal::GoBack));
//                     } else {
//                         let friend_id = identifiable.get();
//                         signal_writer.send(Signal::FriendsList(FriendsListSignal::VisitFriend(friend_id.to_string())));
//                     }
//                 }
//             }
//         }
//     }
// }
//
// fn signal_listener(
//     mut current_page: ResMut<CurrentPage>,
//     mut signal_reader: EventReader<Signal>,
// ) {
//     for signal in signal_reader.read() {
//         match signal {
//             Signal::MainMenu(signal) => {
//                 match signal {
//                     MainMenuSignal::GoToSettings => {
//                         *current_page = CurrentPage::Settings;
//                     }
//                     MainMenuSignal::GoToFriendsList => {
//                         *current_page = CurrentPage::FriendsList;
//                     }
//                 }
//             }
//             Signal::Settings(signal) => {
//                 match signal {
//                     SettingsSignal::Save => {
//                         info!("Settings saved!");
//                     }
//                     SettingsSignal::GoBack => {
//                         *current_page = CurrentPage::MainMenu;
//                     }
//                 }
//             }
//             Signal::FriendsList(signal) => {
//                 match signal {
//                     FriendsListSignal::GoBack => {
//                         *current_page = CurrentPage::MainMenu;
//                     }
//                     FriendsListSignal::VisitFriend(id) => {
//                         info!("Visiting friend with id: {}", id);
//                     }
//                 }
//             }
//         }
//     }
// }
//
// fn page_swapper(
//     current_page: Res<CurrentPage>,
//     mut commands: Commands,
//     app_widgets: Res<AppWidgets>,
//     mut query: Query<(Entity, &RootNode)>,
// ) {
//     if current_page.is_changed() {
//         hydrate_current_page(
//             &mut commands,
//             &app_widgets,
//             &current_page,
//             &mut query
//         );
//     }
// }
//
// fn hydrate_current_page(
//     commands: &mut Commands,
//     app_widgets: &Res<AppWidgets>,
//     current_page: &Res<CurrentPage>,
//     query: &mut Query<(Entity, &RootNode)>,
// ) {
//     // clear the root node
//     for (entity, _) in query.iter_mut() {
//         commands.entity(entity).despawn_recursive();
//     }
//
//     let (widget, data) = match current_page.deref() {
//         CurrentPage::MainMenu => (&app_widgets.main_menu, warhorse_ui::serde_json::Value::Null),
//         CurrentPage::Settings => (&app_widgets.settings, warhorse_ui::serde_json::Value::Null),
//         CurrentPage::FriendsList => (&app_widgets.friends_list, get_friends_data()),
//     };
//
//     commands.spawn((
//         RootNode,
//         Node {
//             width: Val::Percent(100.0),
//             height: Val::Percent(100.0),
//             ..default()
//         }
//     )).with_children(|builder| {
//         hydrate(builder, widget, &data, None);
//     });
// }
//
// fn get_friends_data() -> warhorse_ui::serde_json::Value {
//     json!({
//         "friends": [
//             {
//                 "id": "1",
//                 "inner": [
//                     {
//                         "inner": [
//                             {
//                                 "text": "Alice"
//                             }
//                         ]
//                     }
//                 ]
//             }
//         ]
//     })
// }

