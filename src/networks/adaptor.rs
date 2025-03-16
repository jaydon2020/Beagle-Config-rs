use std::sync::Arc;
use anyhow::Context;
use iwdrs::{adapter::Adapter as iwdAdapter, modes::Mode, session::Session};

use crate::app::AppResult;
use super::device::Device;

#[derive(Debug, Clone)]
pub struct Adapter {
    pub adapter: iwdAdapter,
    pub is_powered: bool,
    pub name: String,
    pub model: Option<String>,
    pub vendor: Option<String>,
    pub supported_modes: Vec<String>,
    pub device: Device,
}

impl Adapter {
    pub async fn new(session: Arc<Session>) -> AppResult<Self> {
        let adapter = session.adapter().context("No adapter found")?;

        let is_powered = adapter.is_powered().await?;
        let name = adapter.name().await?;
        let model = adapter.model().await.ok();
        let vendor = adapter.vendor().await.ok();
        let supported_modes = adapter.supported_modes().await?;
        let device = Device::new(session.clone()).await?;

        Ok(Adapter {
            adapter,
            is_powered,
            name,
            model,
            vendor,
            supported_modes,
            device,
        })
    }

    pub async fn refresh(&mut self) -> AppResult<()> {
        self.is_powered = self.adapter.is_powered().await?;
        self.device.refresh().await?;
        Ok(())
    }
}