use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Deref, Resource, Schedules, World};

use crate::execute_main_thread_runners;
use crate::runner::schedule_initialize;

#[derive(Resource)]
pub struct MainThreadRunner<L> {
    runners: Box<dyn SystemOnMainRunnable>,
    _marker: PhantomData<L>,
}


impl<L: ScheduleLabel> MainThreadRunner<L> {
    #[inline]
    pub fn new(runner: impl SystemOnMainRunnable + 'static) -> MainThreadRunner<L> {
        Self {
            runners: Box::new(runner),
            _marker: PhantomData,
        }
    }
}


impl<L: ScheduleLabel> SystemOnMainRunnable for MainThreadRunner<L> {
    #[inline(always)]
    fn run(&mut self, world: &mut World) -> bool {
        self.runners.run(world)
    }
}

pub trait SystemOnMainRunnable: Send + Sync {
    fn run(&mut self, world: &mut World) -> bool;
}


#[derive(Deref)]
pub struct MainThreadRunnerSender<Label>(std::sync::mpsc::Sender<MainThreadRunner<Label>>);

#[derive(Deref)]
pub struct MainThreadRunnerReceiver<Label>(std::sync::mpsc::Receiver<MainThreadRunner<Label>>);


pub(crate) fn add_main_thread_async_system_if_need<Label: ScheduleLabel + Clone>(world: &mut World, schedule_label: &Label) {
    if !world.contains_non_send::<MainThreadRunnerSender<Label>>() {
        let (tx, rx) = std::sync::mpsc::channel::<MainThreadRunner<Label>>();
        world.insert_non_send_resource(MainThreadRunnerSender(tx));
        world.insert_non_send_resource(MainThreadRunnerReceiver(rx));
        let mut schedules = world.resource_mut::<Schedules>();
        let schedule = schedule_initialize(&mut schedules, schedule_label);
        schedule.add_systems(execute_main_thread_runners::<Label>);
    }
}


pub(crate) trait SendMainThreadRunner {
    fn send_runner<Label: ScheduleLabel>(&mut self, runner: impl SystemOnMainRunnable + 'static);
}


impl SendMainThreadRunner for World {
    #[inline(always)]
    fn send_runner<Label: ScheduleLabel>(&mut self, runner: impl SystemOnMainRunnable + 'static) {
        self
            .non_send_resource::<MainThreadRunnerSender<Label>>()
            .send(MainThreadRunner::new(runner))
            .unwrap()
    }
}