use bevy::prelude::*;

#[derive(Component)]
pub struct Tab<T: TabContent>(T);

#[derive(Component)]
pub struct ActiveTab;

pub trait TabContent: Component + Clone {
    type Content: Bundle + Component;
    fn create_content(world: &mut World) -> Self::Content;
}

const NORMAL_TAB: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_TAB: Color = Color::srgb(0.25, 0.25, 0.25);
const ACTIVE_TAB: Color = Color::srgb(0.35, 0.35, 0.75);

pub mod systems {
    use std::marker::PhantomData;
    use bevy::prelude::*;
    use super::*;

    pub struct TabsPlugin<T: TabContent>(pub PhantomData<T>);

    impl<T: TabContent> Default for TabsPlugin<T> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }

    impl<T: TabContent> Plugin for TabsPlugin<T> {
        fn build(&self, app: &mut App) {
            app.add_systems(Update, (
                interaction::<T>
            ));
        }
    }

    pub fn interaction<T: TabContent>(
        mut commands: Commands,
        interaction_query: Query<(Entity, &Interaction, Option<&ActiveTab>), (Changed<Interaction>, With<Tab<T>>)>,
        mut color_query: Query<(Entity, &mut BackgroundColor), With<Tab<T>>>,
    ) {
        for (entity, interaction, is_active) in &interaction_query {
            match interaction {
                Interaction::Pressed => {
                    if is_active.is_none() {
                        for (other_entity, mut bg_color) in &mut color_query {
                            if other_entity != entity {
                                *bg_color = BackgroundColor(NORMAL_TAB);
                                commands.entity(other_entity).remove::<ActiveTab>();
                            }
                        }
                        if let Ok((_, mut bg_color)) = color_query.get_mut(entity) {
                            *bg_color = BackgroundColor(ACTIVE_TAB);
                            commands.entity(entity).insert(ActiveTab);
                        }
                    }
                }
                Interaction::Hovered => {
                    if is_active.is_none() {
                        if let Ok((_, mut bg_color)) = color_query.get_mut(entity) {
                            *bg_color = BackgroundColor(HOVERED_TAB);
                        }
                    }
                }
                Interaction::None => {
                    if is_active.is_none() {
                        if let Ok((_, mut bg_color)) = color_query.get_mut(entity) {
                            *bg_color = BackgroundColor(NORMAL_TAB);
                        }
                    }
                }
            }
        }
    }
}

pub fn spawn_tabs<T: TabContent>(
    builder: &mut ChildBuilder,
    tab_container: T,
    tabs: Vec<(String, T)>,
    active_tab: i32,
) {
    // spawn the container that holds the tabs
    builder.spawn((
        tab_container,
        Node {
            width: Val::Percent(100.0),
            ..default()
        }
    )).with_children(|parent| {
        // spawn each tab
        for (i, (text, tab_type)) in tabs.into_iter().enumerate() {
            spawn_tab(
                parent,
                text,
                tab_type,
                i as i32 == active_tab,
            );
        }
    });
}

fn spawn_tab<T: TabContent>(
    builder: &mut ChildBuilder,
    text: String,
    tab_type: T,
    is_active: bool,
) -> Entity {
    let mut entity = builder.spawn((
        Tab(tab_type),
        Button,
        Node {
            margin: UiRect {
                top: Val::Px(5.0),
                bottom: Val::Px(5.0),
                left: Val::Px(5.0),
                right: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
    ));

    if is_active {
        entity.insert(ActiveTab);
        entity.insert(BackgroundColor(ACTIVE_TAB));
    } else {
        entity.insert(BackgroundColor(NORMAL_TAB));
    }

    entity
        .with_children(|parent| {
            parent.spawn(Text::new(text));
        })
        .id()
}