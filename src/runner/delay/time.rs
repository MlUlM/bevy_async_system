use std::time::Duration;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Commands, Component, Entity,  Query, Res, Resource, Schedules, TimerMode, World};
use bevy::time::{Time, Timer};

use crate::async_schedules::TaskSender;
use crate::prelude::AsyncScheduleCommand;
use crate::runner::{IntoSetupAction, schedule_initialize, SetupAction};

pub(crate) struct DelayTime(pub Duration);


impl IntoSetupAction for DelayTime {
    fn into_action(self, sender: TaskSender<()>, schedule_label: impl ScheduleLabel + Clone) -> AsyncScheduleCommand {
        AsyncScheduleCommand::new(Setup {
            schedule_label,
            sender,
            timer: Timer::new(self.0, TimerMode::Once),
        })
    }
}


#[derive(Resource)]
struct TimeSystemExists;


#[derive(Component)]
struct LocalTimer(Timer);


struct Setup<Label> {
    sender: TaskSender<()>,
    timer: Timer,
    schedule_label: Label,
}


impl<Label: ScheduleLabel + Clone> SetupAction for Setup<Label> {
    fn setup(self: Box<Self>, world: &mut World) {
        if !world.contains_resource::<TimeSystemExists>() {
            world.insert_resource(TimeSystemExists);
            let mut schedules = world.resource_mut::<Schedules>();
            let schedule = schedule_initialize(&mut schedules, &self.schedule_label);
            schedule.add_systems(tick_timer);
        }

        world.spawn((
            self.sender,
            LocalTimer(self.timer)
        ));
    }
}


fn tick_timer(
    mut commands: Commands,
    mut frames: Query<(Entity, &mut TaskSender<()>, &mut LocalTimer)>,
    time: Res<Time>,
) {
    for (entity, mut sender, mut timer) in frames.iter_mut() {
        if sender.is_closed() {
            commands.entity(entity).despawn();
        } else {
            if timer.0.tick(time.delta()).just_finished() {
                let _ = sender.0.try_send(());
                sender.close_channel();
                commands.entity(entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::app::{Startup, Update};
    use bevy::ecs::event::ManualEventReader;
    use bevy::prelude::{World};
    use crate::ext::spawn_async_system::SpawnAsyncSystemWorld;


    use crate::runner::{delay, once};
    use crate::test_util::{FirstEvent, is_first_event_already_coming, new_app};

    #[test]
    fn delay_time() {
        let mut app = new_app();

        app.add_systems(Startup, |world: &mut World| {
            world.spawn_async(|schedules| async move {
                schedules.add_system(Update, delay::timer(Duration::ZERO)).await;
                schedules.add_system(Update, once::send(FirstEvent)).await;
            });
        });


        // tick
        app.update();
        // send event
        app.update();

        assert!(is_first_event_already_coming(&mut app, &mut ManualEventReader::default()));
    }
}