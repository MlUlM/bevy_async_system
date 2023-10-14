use std::time::Duration;

use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::input::Input;
use bevy::log::info;
use bevy::prelude::{Commands, Entity, KeyCode, Query, Res, With};

use bevy_async_system::AsyncSystemPlugin;
use bevy_async_system::ext::SpawnAsyncCommands;
use bevy_async_system::runner::main_thread::delay::Delay;
use bevy_async_system::async_commands::TaskHandle;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            AsyncSystemPlugin
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, cancel)
        .run();
}


fn setup(mut commands: Commands) {
    commands.spawn_async(|task| async move {
        loop {
            task.spawn_on_main(Update, Delay::Time(Duration::from_secs(1))).await;
            println!("******** tick **********");
        }
    });
}


fn cancel(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    task: Query<Entity, With<TaskHandle>>,
) {
    if input.just_pressed(KeyCode::Return) {
        for entity in task.iter() {
            info!("cancel");
            commands.entity(entity).despawn();
        }
    }
}