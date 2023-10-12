use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{IntoSystem, World};
use futures::channel::mpsc::Sender;
use crate::task::commands::runner::{AsyncSystemRunnable, BaseRunner, BoxedAsyncSystemRunner, SystemRunningStatus};

pub struct AsyncSystemUntilRunner {
    base: BaseRunner<bool>,
}

impl AsyncSystemUntilRunner {
    pub fn boxed<Marker>(
        tx: Sender<bool>,
        schedule_label: impl ScheduleLabel,
        system: impl IntoSystem<(), bool, Marker> + Send + 'static,
    ) -> BoxedAsyncSystemRunner {
        Box::new(Self {
            base: BaseRunner::new(tx, schedule_label, system)
        })
    }
}


impl AsyncSystemRunnable for AsyncSystemUntilRunner
{
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        let no_finished = self.base.run_with_output(world);
        if !no_finished {
            self.base.tx.try_send(true).unwrap();
            SystemRunningStatus::Finished
        } else {
            SystemRunningStatus::Running
        }
    }


    #[inline]
    fn should_run(&self, schedule_label: &dyn ScheduleLabel) -> bool {
        self.base.should_run(schedule_label)
    }
}