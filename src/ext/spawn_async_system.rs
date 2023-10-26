use std::future::Future;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use async_compat::CompatExt;
use async_trait::async_trait;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::{Commands, Component, Deref, DerefMut, World};
use bevy::tasks::AsyncComputeTaskPool;
use crate::async_schedules::{AsyncSchedules, TaskHandle};
use crate::runner::AsyncScheduleCommand;



pub trait SpawnAsyncSystemWorld{
    fn spawn_async<'a, F>(&'a mut self, f: impl Fn(AsyncSchedules) -> F)
        where F: Future<Output=()> + Send + 'static;

}


impl SpawnAsyncSystemWorld for  World{
    fn spawn_async<'a, F>(&'a mut self, f: impl Fn(AsyncSchedules) -> F) where F: Future<Output=()> + Send + 'static {
        let tx = self.non_send_resource::<mpsc::SyncSender<AsyncScheduleCommand>>().clone();

        let async_commands = AsyncSchedules{tx};
        let handle = AsyncComputeTaskPool::get().spawn(f(async_commands.clone()).compat());

        self.spawn( TaskHandle(handle));

    }


}





