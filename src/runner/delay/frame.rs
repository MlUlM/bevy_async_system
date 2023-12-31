use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::{IntoSystemConfigs, Local, Query, Schedules};

use crate::async_schedules::TaskSender;
use crate::prelude::{AsyncScheduleCommand, IntoAsyncScheduleCommand};
use crate::runner::{AsyncSchedule, schedule_initialize, task_running};

pub(crate) struct DelayFrame(pub usize);


impl IntoAsyncScheduleCommand for DelayFrame {
    fn into_schedule_command(self, sender: TaskSender<()>, schedule_label: impl ScheduleLabel + Clone) -> AsyncScheduleCommand {
        AsyncScheduleCommand::new(Scheduler {
            sender,
            schedule_label,
            delay_frames: self.0,
        })
    }
}


struct Scheduler<Label> {
    delay_frames: usize,
    schedule_label: Label,
    sender: TaskSender<()>,
}


impl<Label: ScheduleLabel + Clone> AsyncSchedule for Scheduler<Label> {
    fn initialize(self: Box<Self>, entity_commands: &mut EntityCommands, schedules: &mut Schedules) {
        let schedule = schedule_initialize(schedules, &self.schedule_label);
        entity_commands.insert(self.sender);
        let entity = entity_commands.id();
        let delay_frames = self.delay_frames;
        schedule.add_systems((move |mut frame_count: Local<usize>, mut senders: Query<&mut TaskSender<()>>| {
            *frame_count += 1;
            if delay_frames <= *frame_count {
                let Ok(mut sender) = senders.get_mut(entity) else { return; };
                let _ = sender.try_send(());
                sender.close_channel();
            }
        }).run_if(task_running::<()>(entity)));
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::ecs::event::ManualEventReader;
    use bevy::prelude::Commands;

    use crate::ext::spawn_async_system::SpawnAsyncSystem;
    use crate::runner::{delay, once};
    use crate::test_util::{FirstEvent, is_first_event_already_coming, new_app};

    #[test]
    fn delay_3frames() {
        let mut app = new_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_async(|schedules| async move {
                schedules.add_system(Update, delay::frames(3)).await;
                schedules.add_system(Update, once::send(FirstEvent)).await;
            });
        });
        let mut er = ManualEventReader::default();
        app.update();
        assert!(!is_first_event_already_coming(&mut app, &mut er));
        app.update();
        assert!(!is_first_event_already_coming(&mut app, &mut er));
        app.update();
        assert!(!is_first_event_already_coming(&mut app, &mut er));
        app.update();
        assert!(is_first_event_already_coming(&mut app, &mut er));
        for _ in 0..100 {
            app.update();
            assert!(!is_first_event_already_coming(&mut app, &mut er));
        }
    }
}