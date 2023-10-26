use std::marker::PhantomData;

use bevy::prelude::{IntoSystem, System, World};

pub(crate) struct AsyncSystemConfig<Out, Sys> {
    pub system: Sys,
    m2: PhantomData<Out>,
    initialized: bool,
}


impl<Out, Sys> AsyncSystemConfig<Out, Sys>
    where Sys: System<In=(), Out=Out> + 'static
{
    #[inline(always)]
    pub fn new<Marker>(system: impl IntoSystem<(), Out, Marker, System=Sys> + 'static) -> AsyncSystemConfig<Out, Sys> {
        Self {
            system: IntoSystem::into_system(system),
            m2: PhantomData,
            initialized: false,
        }
    }


    pub fn run(&mut self, world: &mut World) -> Out {

        if !self.initialized {
            self.system.initialize(world);
            self.system.apply_deferred(world);
            self.initialized = true;
        }
        self.system.run((), world)
    }
}