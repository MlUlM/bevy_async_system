use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Commands, Component, Entity, IntoSystemConfigs, Query, Resource, Schedules, World};

use crate::async_schedules::TaskSender;
use crate::prelude::{AsyncScheduleCommand, IntoSetupAction};
use crate::runner::{schedule_initialize, SetupAction};

pub(crate) struct DelayFrame(pub usize);


impl IntoSetupAction for DelayFrame {
    fn into_action(self, sender: TaskSender<()>, schedule_label: impl ScheduleLabel + Clone) -> AsyncScheduleCommand {
        AsyncScheduleCommand::new(Setup {
            sender,
            schedule_label,
            delay_frames: self.0,
        })
    }
}


#[derive(Component)]
struct DelayFrameCount {
    current: usize,
}

#[derive(Resource)]
struct FrameSystemExistsMarker;

struct Setup<Label> {
    delay_frames: usize,
    schedule_label: Label,
    sender: TaskSender<()>,
}


impl<Label: ScheduleLabel + Clone> SetupAction for Setup<Label> {
    fn setup(self: Box<Self>, world: &mut World) {
        if !world.contains_resource::<FrameSystemExistsMarker>() {
            world.insert_resource(FrameSystemExistsMarker);
            let mut schedules = world.resource_mut::<Schedules>();
            let schedule = schedule_initialize(&mut schedules, &self.schedule_label);
            schedule.add_systems(count_decrement);
        }

        world.spawn((
            self.sender,
            DelayFrameCount { current: self.delay_frames }
        ));
    }
}


fn count_decrement(
    mut commands: Commands,
    mut frames: Query<(Entity, &mut TaskSender<()>, &mut DelayFrameCount)>,
) {
    for (entity, mut sender, mut frame_count) in frames.iter_mut() {
        if sender.is_closed() || frame_count.current == 0 {
            commands.entity(entity).despawn();
        } else {
            frame_count.current -= 1;
            if frame_count.current == 0 {
                let _ = sender.0.try_send(());
                sender.close_channel();
                commands.entity(entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::ecs::event::ManualEventReader;
    use bevy::prelude::{Commands, World};
    use crate::ext::spawn_async_system::SpawnAsyncSystemWorld;


    use crate::runner::{delay, once};
    use crate::test_util::{FirstEvent, is_first_event_already_coming, new_app};

    #[test]
    fn delay_3frames() {
        let mut app = new_app();
        app.add_systems(Startup, |world: &mut World| {
            world.spawn_async(|schedules| async move {
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