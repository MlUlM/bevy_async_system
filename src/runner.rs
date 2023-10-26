use std::sync::{Arc, Mutex};

use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::EntityCommands;
use bevy::hierarchy::BuildChildren;
use bevy::prelude::{Component, Condition, Deref, DerefMut, Entity, IntoSystem, Query, Schedule, Schedules, World};

use crate::async_schedules::TaskSender;

pub(crate) mod config;
pub mod once;

pub mod main_thread;
pub mod delay;
pub mod wait;


pub mod preludes {
    pub use crate::runner::{
        SetupAction,
        AsyncScheduleCommand,
        IntoSetupAction,
        once,
        delay,
        wait
    };
}


pub trait IntoSetupAction<Out = ()>: Sized {
    fn into_action(self, sender: TaskSender<Out>, schedule_label: impl ScheduleLabel + Clone) -> AsyncScheduleCommand;
}


pub trait SetupAction: Send + Sync {
    fn setup(self: Box<Self>, world: &mut World);
}


#[derive(Component, Deref, DerefMut)]
pub struct AsyncScheduleCommand(pub Box<dyn SetupAction>);

impl AsyncScheduleCommand {
    #[inline]
    pub fn new(s: impl SetupAction + 'static) -> Self {
        Self(Box::new(s))
    }
}


pub fn schedule_initialize<'a, Label: ScheduleLabel + Clone>(schedules: &'a mut Schedules, schedule_label: &Label) -> &'a mut Schedule {
    if !schedules.contains(schedule_label) {
        schedules.insert(schedule_label.clone(), Schedule::default());
    }

    schedules.get_mut(schedule_label).unwrap()
}

