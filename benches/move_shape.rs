// use bevy::app::{App, Startup, Update};
// use bevy::core::TaskPoolPlugin;
// use bevy::math::Vec3;
// use bevy::prelude::{Commands, Component, in_state, IntoSystemConfigs, NextState, Query, ResMut, State, States, Transform, With};
// use bevy::transform::TransformBundle;
// use criterion::{Criterion, criterion_group, criterion_main};
//
// use bevy_async_system::AsyncSystemPlugin;
// use bevy_async_system::prelude::SpawnAsyncSystem;
// use bevy_async_system::runner::{once, wait};
//
// fn bm1(c: &mut Criterion) {
//     let mut app = App::new();
//     app
//         .add_state::<MoveState>()
//         .add_plugins(TaskPoolPlugin::default())
//         .add_systems(Startup, setup)
//         .add_systems(Update, transform.run_if(in_state(MoveState::Move)));
//
//     c.bench_function("Pure Bevy", move |b| b.iter(|| {
//         loop {
//             app.update();
//             if matches!(app.world.resource::<State<MoveState>>().get(), MoveState::Finished) {
//                 break;
//             }
//         }
//     }));
// }
//
//
// fn with_plugin(c: &mut Criterion) {
//     let mut app = App::new();
//     app
//         .add_plugins((
//             TaskPoolPlugin::default(),
//             AsyncSystemPlugin
//         ))
//         .add_state::<MoveState>()
//         .add_systems(Startup, (
//             setup,
//             setup_systems
//         ));
//
//     c.bench_function("With Plugin", move |b| b.iter(|| {
//         loop {
//             app.update();
//             if matches!(app.world.resource::<State<MoveState>>().get(), MoveState::Finished) {
//                 break;
//             }
//         }
//     }));
// }
//
//
// fn setup(mut commands: Commands) {
//     commands.spawn((
//         Movable,
//         TransformBundle::from_transform(Transform::from_translation(Vec3::ZERO))
//     ));
// }
//
// fn setup_systems(mut commands: Commands) {
//     commands.spawn_async(|s| async move {
//         s.add_system(Update, wait::until(|mut t: Query<&mut Transform, With<Movable>>| {
//             let mut t = t.single_mut();
//             t.translation.y += 1.;
//             50. <= t.translation.y
//         })).await;
//         s.add_system(Update, once::set_state(MoveState::Finished)).await;
//     });
// }
//
// #[derive(Component)]
// struct Movable;
//
// #[derive(Eq, PartialEq, Copy, Clone, Debug, Default, States, Hash)]
// enum MoveState {
//     #[default]
//     Move,
//     Finished,
// }
//
// fn transform(mut t: Query<&mut Transform, With<Movable>>, mut state: ResMut<NextState<MoveState>>) {
//     let mut t = t.single_mut();
//     t.translation.y += 1.;
//     if 50. <= t.translation.y {
//         state.set(MoveState::Finished);
//     }
// }
//
//
//
//
// criterion_group!(benches, bm1, with_plugin);
// criterion_main!(benches);
//
//
//
