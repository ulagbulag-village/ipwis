use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use ipis::{
    core::anyhow::{bail, Result},
    rkyv::AlignedVec,
};
use ipwis_kernel_common::interrupt::{
    InterruptFallbackHandler, InterruptFallbackModule, InterruptHandler, InterruptId,
    InterruptModule,
};

use crate::memory::IpwisMemory;

#[derive(Default)]
pub struct InterruptManager {
    map: HashMap<InterruptId, Box<dyn InterruptModule<IpwisMemory<'static>>>>,
    fallback: Option<Box<dyn InterruptFallbackModule<IpwisMemory<'static>>>>,
}

impl InterruptManager {
    pub fn insert<H>(&mut self, handler: H) -> Result<()>
    where
        H: InterruptModule<IpwisMemory<'static>> + 'static,
    {
        let id = handler.id();

        match self.map.entry(id) {
            Entry::Vacant(e) => {
                e.insert(Box::new(handler));
                Ok(())
            }
            Entry::Occupied(_) => bail!("duplicated interrupt handler: {id}"),
        }
    }

    pub fn set_fallback<H>(&mut self, handler: H) -> Result<()>
    where
        H: InterruptFallbackModule<IpwisMemory<'static>> + 'static,
    {
        self.fallback = Some(Box::new(handler));
        Ok(())
    }

    pub async fn spawn_handler(
        &self,
        id: InterruptId,
    ) -> Result<Option<Box<dyn InterruptHandler<IpwisMemory<'static>>>>> {
        match self.map.get(&id) {
            Some(module) => module.spawn_handler().await.map(Some),
            None => Ok(None),
        }
    }

    pub async fn spawn_fallback(
        &self,
    ) -> Result<Option<Box<dyn InterruptFallbackHandler<IpwisMemory<'static>>>>> {
        match self.fallback.as_ref() {
            Some(module) => module.spawn_fallback().await.map(Some),
            None => Ok(None),
        }
    }
}

pub struct InterruptHandlerStore {
    manager: Arc<InterruptManager>,
    map: HashMap<InterruptId, Box<dyn InterruptHandler<IpwisMemory<'static>>>>,
    fallback: Option<Box<dyn InterruptFallbackHandler<IpwisMemory<'static>>>>,
}

impl InterruptHandlerStore {
    pub fn with_manager(manager: Arc<InterruptManager>) -> Self {
        Self {
            manager,
            map: Default::default(),
            fallback: Default::default(),
        }
    }

    pub async unsafe fn handle_raw(
        &mut self,
        memory: &mut IpwisMemory<'static>,
        id: InterruptId,
        inputs: &[u8],
    ) -> Result<AlignedVec> {
        match self.map.get_mut(&id) {
            Some(handler) => handler.handle_raw(memory, inputs).await,
            None => match self.manager.spawn_handler(id).await? {
                Some(handler) => {
                    self.map.insert(id, handler);
                    self.map
                        .get_mut(&id)
                        .unwrap()
                        .handle_raw(memory, inputs)
                        .await
                }
                None => match self.fallback.as_ref() {
                    Some(handler) => handler.handle_fallback(memory, id, inputs).await,
                    None => match self.manager.spawn_fallback().await? {
                        Some(handle) => {
                            self.fallback.replace(handle);
                            self.fallback
                                .as_ref()
                                .unwrap()
                                .handle_fallback(memory, id, inputs)
                                .await
                        }
                        None => bail!("failed to find the interrupt handler: {id}"),
                    },
                },
            },
        }
    }

    pub async fn release(&mut self) -> Result<()> {
        for handler in self.map.values_mut() {
            handler.release().await?;
        }
        if let Some(handler) = self.fallback.as_mut() {
            handler.release().await?;
        }
        Ok(())
    }
}
