use bevy::app::AppExit;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Commands, Event, EventWriter, FromWorld, IntoSystem, IntoSystemConfigs, NextState, ResMut, Resource, States, System, World};

use crate::async_schedules::TaskSender;
use crate::ext::spawn_async_system::SpawnAsyncSystemWorld;
use crate::prelude::{AsyncScheduleCommand, IntoSetupAction, SetupAction};
use crate::runner::config::AsyncSystemConfig;
use crate::runner::main_thread::{add_main_thread_async_system_if_need, SendMainThreadRunner, SystemOnMainRunnable};

/// Run the system only once.
///
/// The system can use `Output`.
/// If any output is returned, it becomes the task's return value.
///
/// ## Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|schedules| async move{
///         schedules.add_system(Update, once::run(without_output)).await;
///         let count: u32 = schedules.add_system(Update, once::run(with_output)).await;
///         assert_eq!(count, 10);
///     });
/// }
///
/// #[derive(Resource)]
/// struct Count(u32);
///
/// fn without_output(mut commands: Commands){
///     commands.insert_resource(Count(10));
/// }
///
///
/// fn with_output(count: Res<Count>) -> u32{
///     count.0
/// }
///
/// ```
///
#[inline(always)]
pub fn run<Out, Marker, Sys>(system: impl IntoSystem<(), Out, Marker, System=Sys> + 'static) -> impl IntoSetupAction<Out>
    where
        Out: Send + Sync + 'static,
        Marker: Send + Sync + 'static,
        Sys: System<In=(), Out=Out> + 'static
{
    OnceOnMain(AsyncSystemConfig::<Out, Sys>::new(system))
}


/// Set the next state.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// #[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone, States)]
/// enum ExampleState{
///     #[default]
///     First,
///     Second,
/// }
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|scheduler|async move{
///         scheduler.add_system(Update, once::set_state(ExampleState::Second)).await;
///     });
/// }
/// ```
///
#[inline]
pub fn set_state<S: States + Copy>(to: S) -> impl IntoSetupAction {
    run(move |mut state: ResMut<NextState<S>>| {
        state.set(to);
    })
}


/// Send the event.
///
/// The event to be send must derive [`Clone`] in addition to [`Event`](bevy::prelude::Event).
///
/// ```
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// #[derive(Event, Clone)]
/// struct ExampleEvent;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|schedules|async move{
///         schedules.add_system(Update, once::send(ExampleEvent)).await;
///     });
/// }
/// ```
#[inline]
pub fn send<E: Event + Clone>(event: E) -> impl IntoSetupAction {
    run(move |mut ew: EventWriter<E>| {
        ew.send(event.clone());
    })
}


/// Send [`AppExit`].
#[inline(always)]
pub fn app_exit() -> impl IntoSetupAction {
    send(AppExit)
}


/// Insert a [`Resource`](bevy::prelude::Resource).
///
/// The resource is cloned inside the function.
///
/// If the resource derives [`Default`], we recommend using [`once::init_resource`](once::init_resource) instead.
/// ```
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// #[derive(Resource, Clone)]
/// struct ExampleResource;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|schedules|async move{
///         schedules.add_system(Update, once::insert_resource(ExampleResource)).await;
///     });
/// }
/// ```
#[inline]
pub fn insert_resource<R: Resource + Clone>(resource: R) -> impl IntoSetupAction {
    run(move |mut commands: Commands| {
        commands.insert_resource(resource.clone());
    })
}


/// Initialize a [`Resource`](bevy::prelude::Resource).
///
/// ```
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// #[derive(Resource, Default)]
/// struct ExampleResource;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|schedules|async move{
///         schedules.add_system(Update, once::init_resource::<ExampleResource>()).await;
///     });
/// }
/// ```
#[inline]
pub fn init_resource<R: Resource + Default>() -> impl IntoSetupAction {
    run(|mut commands: Commands| {
        commands.init_resource::<R>();
    })
}


/// Init a non send resource.
///
/// The system runs on the main thread.
/// ```
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// #[derive(Default)]
/// struct ExampleResource;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|schedules|async move{
///         schedules.add_system(Update, once::init_non_send_resource::<ExampleResource>()).await;
///     });
/// }
/// ```
#[inline]
pub fn init_non_send_resource<R: FromWorld + 'static>() -> impl IntoSetupAction {
    run(move |world: &mut World| {
        world.init_non_send_resource::<R>();
    })
}


struct OnceOnMain<Out, Sys>(AsyncSystemConfig<Out, Sys>);


impl<Out, Sys> IntoSetupAction<Out> for OnceOnMain<Out, Sys>
    where
        Out: Send + Sync + 'static,
        Sys: System<In=(), Out=Out> + Send + Sync + 'static
{
    fn into_action(self, sender: TaskSender<Out>, schedule_label: impl ScheduleLabel + Clone) -> AsyncScheduleCommand {
        AsyncScheduleCommand::new(OnceSetup {
            config: self.0,
            sender,
            schedule_label,
        })
    }
}


struct OnceSetup<Out, Sys, Label> {
    config: AsyncSystemConfig<Out, Sys>,
    schedule_label: Label,
    sender: TaskSender<Out>,
}


impl<Out, Sys, Label> SetupAction for OnceSetup<Out, Sys, Label>
    where
        Out: Send + Sync + 'static,
        Sys: System<In=(), Out=Out>,
        Label: ScheduleLabel + Clone
{
    fn setup(self: Box<Self>, world: &mut World) {
        add_main_thread_async_system_if_need(world, &self.schedule_label);
        world.send_runner::<Label>(OnceRunner {
            config: self.config,
            sender: self.sender,
        });
    }
}


struct OnceRunner<Out, Sys> {
    config: AsyncSystemConfig<Out, Sys>,
    sender: TaskSender<Out>,
}

impl<Out, Sys> SystemOnMainRunnable for OnceRunner<Out, Sys>
    where
        Out: Send + Sync + 'static,
        Sys: System<In=(), Out=Out>
{
    fn run(&mut self, world: &mut World) -> bool {
        if self.sender.is_closed(){
           return true;
        }

        let _ = self.sender.try_send(self.config.run(world));
        self.sender.close_channel();
        true
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{PreUpdate, Startup};
    use bevy::prelude::World;

    use crate::ext::spawn_async_system::SpawnAsyncSystemWorld;
    use crate::runner::once;
    use crate::test_util::{new_app, test_state_finished, TestState};

    #[test]
    fn set_state() {
        let mut app = new_app();
        app.add_systems(Startup, |world: &mut World| {
            world.spawn_async(|schedules| async move {
                schedules.add_system(PreUpdate, once::set_state(TestState::Finished)).await;
            });
        });

        app.update();
        app.update();
        app.update();
        app.update();
        assert!(test_state_finished(&mut app));
    }
    //
    //
    // #[test]
    // fn send_event() {
    //     let mut app = new_app();
    //     app.add_systems(Startup, |mut commands: Commands| {
    //         commands.spawn_async(|schedules| async move {
    //             schedules.add_system(Update, once::send(FirstEvent)).await;
    //             schedules.add_system(Update, once::send(SecondEvent)).await;
    //         });
    //     });
    //
    //     let mut er_first = ManualEventReader::default();
    //     let mut er_second = ManualEventReader::default();
    //
    //     app.update();
    //
    //     assert!(is_first_event_already_coming(&mut app, &mut er_first));
    //     assert!(!is_second_event_already_coming(&mut app, &mut er_second));
    //
    //     app.update();
    //     assert!(!is_first_event_already_coming(&mut app, &mut er_first));
    //     assert!(is_second_event_already_coming(&mut app, &mut er_second));
    //
    //     app.update();
    //     assert!(!is_first_event_already_coming(&mut app, &mut er_first));
    //     assert!(!is_second_event_already_coming(&mut app, &mut er_second));
    // }
    //
    //
    // #[test]
    // fn output() {
    //     let mut app = new_app();
    //     app.add_systems(Startup, setup);
    //
    //     app.update();
    //     app.update();
    // }
    //
    //
    // fn setup(mut commands: Commands) {
    //     commands.spawn_async(|schedules| async move {
    //         schedules.add_system(Update, once::run(without_output)).await;
    //         let count: u32 = schedules.add_system(Update, once::run(with_output)).await;
    //         assert_eq!(count, 10);
    //     });
    // }
    //
    // fn without_output(mut commands: Commands) {
    //     commands.insert_resource(Count(10));
    // }
    //
    //
    // fn with_output(count: Res<Count>) -> u32 {
    //     count.0
    // }
    //
    // #[derive(Resource)]
    // struct Count(u32);
    //
    //
    // #[test]
    // fn init_non_send_resource() {
    //     let mut app = new_app();
    //     app.add_systems(Startup, |mut commands: Commands| {
    //         commands.spawn_async(|schedules| async move {
    //             schedules.add_system(Update, once::init_non_send_resource::<NonSendNum>()).await;
    //             schedules.add_system(Update, once::run(|mut r: NonSendMut<NonSendNum>| {
    //                 r.0 = 3;
    //             })).await;
    //         });
    //     });
    //
    //     app.update();
    //     assert_eq!(app.world.non_send_resource::<NonSendNum>().0, 0);
    //     app.update();
    //     assert_eq!(app.world.non_send_resource::<NonSendNum>().0, 3);
    //
    //     #[derive(Default)]
    //     struct NonSendNum(usize);
    // }
}
