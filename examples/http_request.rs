use bevy::app::{App, Startup, Update};
use bevy::asset::AssetServer;
use bevy::DefaultPlugins;
use bevy::prelude::{Camera2dBundle, Color, Commands, Query, Res, TextBundle};
use bevy::text::{Text, TextStyle};

use bevtask::BevTaskPlugin;
use bevtask::ext::AsyncPool;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            BevTaskPlugin
        ))
        .add_systems(Startup, (
            setup_ui,
            setup_tasks
        ))
        .run();
}


fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(TextBundle::from_section("Loading", TextStyle {
        font_size: 80.,
        color: Color::BLACK,
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
    }));
}


fn setup_tasks(mut commands: Commands) {
    commands.spawn_async(|task| async move {
        let client = reqwest::get("https://github.com/elm-register").await;
        task.once(Update, move |mut text: Query<&mut Text>| {
            text.single_mut().sections[0].value = if let Ok(response) = client.as_ref() {
                format!("status code: {:?}", response.status())
            } else {
                "Failed".to_string()
            };
        }).await;
    });
}