use std::collections::{hash_map::Entry, HashMap};

use ipis::{core::anyhow::{bail, Result}, rkyv::AlignedVec};
use ipwis_kernel_common::interrupt::{InterruptFallbackHandler, InterruptHandler, InterruptId};

#[derive(Default)]
pub struct InterruptHandlerStore {
    map: HashMap<InterruptId, Box<dyn InterruptHandler>>,
    fallback: Option<Box<dyn InterruptFallbackHandler>>,
}

impl InterruptHandlerStore {
    pub fn insert<H>(&mut self, handler: H) -> Result<()>
    where
        H: InterruptHandler + 'static,
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
        H: InterruptFallbackHandler + 'static,
    {
        self.fallback = Some(Box::new(handler));
        Ok(())
    }

    pub fn handle_raw(&self, id: InterruptId, inputs: &[u8]) -> Result<AlignedVec> {
        match self.map.get(&id) {
            Some(handler) => handler.handle_raw(inputs),
            None => match self
                .fallback
                .as_ref()
                .and_then(|handler| handler.handle_fallback(id, inputs).transpose())
            {
                Some(result) => result,
                None => bail!("failed to find the interrupt handler: {id}"),
            },
        }
    }
}
