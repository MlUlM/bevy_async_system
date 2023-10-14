use std::marker::PhantomData;

use bevy::prelude::IntoSystem;
use crate::prelude::IntoAsyncScheduleCommand;
use crate::runner::repeat::times::Times;

mod times;
// mod forever;

/// Repeats the system call  a specified number of times or indefinitely.
///
///
/// ```no_run
/// use std::time::Duration;
/// use bevy::prelude::*;
/// use bevy_async_system::ext::SpawnAsyncCommands;
/// use bevy_async_system::prelude::*;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|task| async move{
///         // Call `my_system` for 3 frames.
///         task.spawn_on_main(Update, Repeat::times(3, my_system)).await;
///
///         // It's called every frame while this task is running.
///         let handle = task.spawn_on_main(Update, Repeat::forever(my_system));
///
///         // When the handle is dropped, calling `my_system` also stops.
///         drop(handle)
///     });
/// }
///
/// fn my_system(){
///     // ... your code
/// }
/// ```
pub struct Repeat(PhantomData<()>);


#[inline(always)]
pub fn times<Marker, Sys>(num: usize, system: Sys) -> impl IntoAsyncScheduleCommand
 where
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), (), Marker> + Send + Sync + 'static
{
    Times::create(num, system)
}


// #[inline(always)]
// pub fn forever<Marker>(system: impl IntoSystem<(), (), Marker> + 'static + Send) -> impl IntoMainThreadExecutor {
//     Forever(AsyncSystemConfig::new(system))
// }


