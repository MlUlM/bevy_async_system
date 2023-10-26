use bevy::app::{App, Startup, Update};
use bevy::core::TaskPoolPlugin;
use bevy::prelude::{ResMut, Resource, World};
use criterion::{Criterion, criterion_group, criterion_main};

use bevy_async_system::AsyncSystemPlugin;
use bevy_async_system::ext::spawn_async_system::SpawnAsyncSystemWorld;
use bevy_async_system::prelude::once;
use bevy_async_system::runner::wait;

const COUNT: isize = 1000;


#[derive(Eq, PartialEq, Copy, Clone, Debug, Default, Resource, Hash)]
struct Count(isize);


fn benchmark_delay(c: &mut Criterion) {
    c.bench_function("delay", move |b| b.iter(|| {
        let mut app = App::new();
        app
            .add_plugins((
                TaskPoolPlugin::default(),
                AsyncSystemPlugin
            ))
            .insert_resource(Count(COUNT))
            .add_systems(Startup, |world: &mut World| {
                world.spawn_async(|schedules| async move {
                    schedules.add_system(Update, wait::until(|mut count: ResMut<Count>| {
                        count.0 -= 1;
                        count.0 == 0
                    })).await;
                });
            });

        loop {
            app.update();
            if app.world.resource::<Count>().0 == 0 {
                break;
            }
        }
    }));
}


fn benchmark(c: &mut Criterion) {
    c.bench_function("amount_of_add_systems", move |b| b.iter(|| {
        let mut app = App::new();
        app
            .add_plugins((
                TaskPoolPlugin::default(),
                AsyncSystemPlugin
            ))
            .insert_resource(Count(COUNT))
            .add_systems(Startup, |world: &mut World| {
                world.spawn_async(|schedules| async move {
                    loop {
                        schedules.add_system(Update, once::run(|mut count: ResMut<Count>| {
                            count.0 -= 1;
                        })).await;
                    }
                });
            });

        loop {
            app.update();
            if app.world.resource::<Count>().0 == 0 {
                break;
            }
        }
    }));
}


criterion_group!(benches, benchmark_delay, benchmark);
criterion_main!(benches);



