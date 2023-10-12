use bevy::app::{App, Startup, Update};
use bevy::core::TaskPoolPlugin;
use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Event, Events, EventWriter, NonSendMut};

use bevtask::AsyncSystemPlugin;
use bevtask::task::BevTask;

#[derive(Event)]
struct FinishEvent;

#[test]
fn delay_frames() {
    let mut app = App::new();
    app.add_event::<FinishEvent>();
    app.add_plugins((
        TaskPoolPlugin::default(),
        AsyncSystemPlugin
    ));

    app.add_systems(Startup, setup);
    app.update();
    app.update();
    app.update();
    app.update();

    let er = ManualEventReader::<FinishEvent>::default();
    let events = app.world.resource::<Events<FinishEvent>>();
    assert!(!er.is_empty(events));
}


fn setup(
    mut task: NonSendMut<BevTask>,
) {
    task.spawn_async(|commands| async move {
        commands.delay_frame(Update, 3).await;
        commands.once(Update, send_finish_event).await;
    });
}


fn send_finish_event(mut ew: EventWriter<FinishEvent>) {
    ew.send(FinishEvent);
}