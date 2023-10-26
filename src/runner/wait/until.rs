use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Event, EventReader, IntoSystem, IntoSystemConfigs, System, World};

use crate::async_schedules::TaskSender;
use crate::runner::{AsyncScheduleCommand, IntoSetupAction, SetupAction};
use crate::runner::config::AsyncSystemConfig;
use crate::runner::main_thread::{add_main_thread_async_system_if_need, SendMainThreadRunner, SystemOnMainRunnable};

/// Runs the system every frame until it returns true.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|schedules|async move{
///         schedules.add_system(Update, wait::until(move_up)).await;
///     });
/// }
///
/// fn move_up(mut transform: Query<&mut Transform>) -> bool{
///     let mut transform = transform.single_mut();
///     transform.translation.y += 1.;
///     50. <= transform.translation.y
/// }
/// ```
#[inline(always)]
pub fn until<Marker, Sys>(system: impl IntoSystem<(), bool, Marker, System=Sys> + 'static) -> impl IntoSetupAction<()>
     where
        Marker: Send + Sync + 'static,
        Sys: System<In=(), Out=bool> + 'static
{
    Until(AsyncSystemConfig::new(system))
}


/// Wait until an event is received.
///
/// Unlike [`wait::output_event`](wait::output_event), there is no return value,
/// but `E` does not need to derive [`Clone`].
#[inline(always)]
pub fn until_event<E: Event>() -> impl IntoSetupAction<()> {
    until(|er: EventReader<E>| { !er.is_empty() })
}


struct Until<Sys>(AsyncSystemConfig<bool, Sys>);


impl<Sys> IntoSetupAction<()> for Until<Sys>
    where
        Sys: System<In=(), Out=bool> + 'static
{
    #[inline]
    fn into_action(self, sender: TaskSender<()>, schedule_label: impl ScheduleLabel + Clone) -> AsyncScheduleCommand {
        AsyncScheduleCommand::new(Setup {
            sender,
            config: self.0,
            schedule_label,
        })
    }
}


struct Setup<Sys, Label> {
    sender: TaskSender<()>,
    config: AsyncSystemConfig<bool, Sys>,
    schedule_label: Label,
}


impl<Sys, Label> SetupAction for Setup<Sys, Label>
    where
        Sys: System<In=(), Out=bool> + Send + Sync + 'static,
        Label: ScheduleLabel + Clone
{
    fn setup(self: Box<Self>, world: &mut World) {
        add_main_thread_async_system_if_need(world, &self.schedule_label);

        world.send_runner::<Label>(Runner {
            config: self.config,
            sender: self.sender,
        });
    }
}


struct Runner<Sys> {
    config: AsyncSystemConfig<bool, Sys>,
    sender: TaskSender<()>,
}

impl<Sys> SystemOnMainRunnable for Runner<Sys>
    where
        Sys: System<In=(), Out=bool>
{
    fn run(&mut self, world: &mut World) -> bool {
        if self.sender.is_closed() {
            return true;
        }

        if self.config.run(world) {
            let _ = self.sender.try_send(());
            self.sender.close_channel();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::core::FrameCount;
    use bevy::ecs::event::ManualEventReader;
    use bevy::prelude::{Res, World};

    use crate::ext::spawn_async_system::SpawnAsyncSystemWorld;
    use crate::runner::{once, wait};
    use crate::test_util::{FirstEvent, is_first_event_already_coming, new_app};

    #[test]
    fn until() {
        let mut app = new_app();
        app.add_systems(Startup, |world: &mut World| {
            world.spawn_async(|schedules| async move {
                schedules.add_system(Update, wait::until(|frame: Res<FrameCount>| {
                    frame.0 == 2
                })).await;
                schedules.add_system(Update, once::send(FirstEvent)).await;
            });
        });

        app.update();
        app.update();
        app.update();

        // send event
        app.update();

        assert!(is_first_event_already_coming(&mut app, &mut ManualEventReader::default()));
    }

    #[test]
    fn never_again() {
        let mut app = new_app();
        app.add_systems(Startup, |world: &mut World| {
            world.spawn_async(|schedules| async move {
                schedules.add_system(Update, wait::until(|frame: Res<FrameCount>| {
                    if 2 <= frame.0 {
                        panic!("must not be called");
                    }
                    frame.0 == 1
                })).await;
                schedules.add_system(Update, wait::until(|| false)).await;
            });
        });

        for _ in 0..100 {
            app.update();
        }
    }
}