#![allow(clippy::type_complexity)]

use std::sync::mpsc::{Receiver, Sender, SyncSender};

use bevy::app::{App, First, Main, Plugin, Update};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::prelude::{Commands, Entity, NonSend, Query, ResMut, Schedules, World};
use futures_lite::future::block_on;

use crate::async_schedules::TaskHandle;
use crate::runner::AsyncScheduleCommand;
use crate::runner::main_thread::{MainThreadRunner, MainThreadRunnerReceiver, MainThreadRunnerSender, SystemOnMainRunnable};

pub mod async_schedules;
pub mod ext;

pub mod runner;


pub mod prelude {
    pub use crate::{
        async_schedules::*,
        AsyncSystemPlugin,
        runner::preludes::*,
    };
}

/// Provides the async systems.
pub struct AsyncSystemPlugin;


impl Plugin for AsyncSystemPlugin {
    fn build(&self, app: &mut App) {
        {
            use bevy::prelude::IntoSystemConfigs;
            let (tx, rx) = std::sync::mpsc::sync_channel::<AsyncScheduleCommand>(100);
            app
                .add_systems(Main, remove_finished_tasks)
                .add_systems(First, scedule_command)
                .insert_non_send_resource(rx)
                .insert_non_send_resource(tx);
        }
    }
}



fn scedule_command(
    world: &mut World
){
    let rx= world.remove_non_send_resource::<Receiver<AsyncScheduleCommand>>().unwrap();
    for s in rx.try_iter(){
        s.0.setup(world);
    }
    world.insert_non_send_resource(rx);
}


fn execute_main_thread_runners<L: ScheduleLabel>(
    world: &mut World
) {
    let rx = world.remove_non_send_resource::<MainThreadRunnerReceiver<L>>().unwrap();
    let mut non_completes = Vec::new();
    for mut runner in rx.try_iter() {
        if !runner.run(world) {
           non_completes.push(runner);
        }
    }

    for runner in non_completes.into_iter(){
         world.non_send_resource::<MainThreadRunnerSender<L>>().send(runner).unwrap();
    }

    world.insert_non_send_resource(rx);
}



fn remove_finished_tasks(
    mut commands: Commands,
    mut task_handles: Query<(Entity, &mut TaskHandle)>,
) {
    for (entity, mut task) in task_handles.iter_mut() {
        if block_on(futures_lite::future::poll_once(&mut task.0)).is_some() {
            commands.entity(entity).despawn_recursive();
        }
    }
}


#[cfg(test)]
pub(crate) mod test_util {
    use bevy::app::App;
    use bevy::core::{FrameCountPlugin, TaskPoolPlugin};
    use bevy::ecs::event::ManualEventReader;
    use bevy::prelude::{Event, Events, State, States};
    use bevy::time::TimePlugin;

    use crate::AsyncSystemPlugin;

    #[derive(Event, Copy, Clone, Debug, Eq, PartialEq)]
    pub struct FirstEvent;


    #[derive(Event, Copy, Clone, Debug, Eq, PartialEq)]
    pub struct SecondEvent;


    #[derive(Default, Debug, Copy, Clone, Eq, PartialEq, States, Hash)]
    pub enum TestState {
        #[default]
        Empty,
        Finished,
    }

    pub fn new_app() -> App {
        let mut app = App::new();
        app.add_state::<TestState>();
        app.add_plugins((
            TaskPoolPlugin::default(),
            FrameCountPlugin,
            TimePlugin,
            AsyncSystemPlugin
        ));
        app.add_event::<FirstEvent>();
        app.add_event::<SecondEvent>();
        app
    }


    pub fn is_first_event_already_coming(app: &mut App, er: &mut ManualEventReader<FirstEvent>) -> bool {
        is_event_already_coming::<FirstEvent>(app, er)
    }


    pub fn is_second_event_already_coming(app: &mut App, er: &mut ManualEventReader<SecondEvent>) -> bool {
        is_event_already_coming::<SecondEvent>(app, er)
    }


    pub fn is_event_already_coming<E: Event>(app: &mut App, er: &mut ManualEventReader<E>) -> bool {
        let events = app.world.resource::<Events<E>>();
        let come = !er.is_empty(events);
        er.clear(events);

        come
    }

    pub fn test_state_finished(app: &mut App) -> bool {
        matches!(app.world.resource::<State<TestState>>().get(), TestState::Finished)
    }
}
