use crate::prelude::*;

use crate::replay::ReplayModule;

// region: Event ModuleLoaded

#[derive(Debug)]
pub struct ModuleLoaded {
    pub position: crate::bindings::root::TTD::Replay::Position,
    pub module: ReplayModule,
}

impl TryFrom<&crate::bindings::root::TTD::Replay::ModuleLoadedEvent> for ModuleLoaded {
    fn try_from(value: &crate::bindings::root::TTD::Replay::ModuleLoadedEvent) -> Result<Self> {
        let module = unsafe { *value.pModule };
        Ok(Self {
            position: value.Position,
            module: ReplayModule::try_from(&module)?,
        })
    }
    type Error = crate::error::Error;
}

// endregion: ModuleLoaded type

// region: ModuleUnloaded type

#[derive(Debug)]
pub struct ModuleUnloaded {
    pub position: crate::bindings::root::TTD::Replay::Position,
    pub module: ReplayModule,
}
impl TryFrom<&crate::bindings::root::TTD::Replay::ModuleUnloadedEvent> for ModuleUnloaded {
    fn try_from(value: &crate::bindings::root::TTD::Replay::ModuleUnloadedEvent) -> Result<Self> {
        let module = unsafe { *value.pModule };
        Ok(Self {
            position: value.Position,
            module: ReplayModule::try_from(&module)?,
        })
    }
    type Error = crate::error::Error;
}

// endregion: ModuleUnloaded type
