mod game;

use crate::game::Game;
use bevy::prelude::*;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Squares".to_string(),
            width: 710.0,
            height: 710.0,
            resizable: false,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
        .add_plugins(DefaultPlugins)
        .add_plugin(Game)
        .run();
}
