mod friends;

use bevy::prelude::*;

pub struct WarhorseUIPlugin;
impl Plugin for WarhorseUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(friends::FriendsListPlugin);
    }
}
