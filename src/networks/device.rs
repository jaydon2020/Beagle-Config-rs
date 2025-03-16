use std::sync::Arc;

use anyhow::Context;
use iwdrs::{device::Device as iwdDevice, modes::Mode, session::Session};

use crate::app::AppResult;

use super::station::Station;

#[derive(Debug, Clone)]
pub struct Device {
    pub session: Arc<Session>,
    pub device: iwdDevice,
    pub name: String,
    pub address: String,
    pub mode: Mode,
    pub is_powered: bool,
    pub station: Option<Station>,
}

impl Device {
    pub async fn new(session: Arc<Session>) -> AppResult<Self> {
        let device = session.device().context("No device found")?;

        let name = device.name().await?;
        let address = device.address().await?;
        let mode = device.get_mode().await?;
        let is_powered = device.is_powered().await?;

        let station = match session.station() {
            Some(_) => match Station::new(session.clone()).await {
                Ok(v) => Some(v),
                Err(e) => {
                    None
                }
            },
            None => None,
        };
        
        Ok(Device {
            session,
            device,
            name,
            address,
            mode,
            is_powered,
            station,
        })
    }

    pub async fn refresh(&mut self) -> AppResult<()> {
        self.is_powered = self.device.is_powered().await?;
        let current_mode = self.device.get_mode().await?;

        match current_mode {
            Mode::Station => {
                match self.mode {
                    Mode::Station => {
                        // refresh exisiting station
                        if let Some(station) = &mut self.station {
                            station.refresh().await?;
                        }
                    }
                    Mode::Ap => {
                        // Switch mode from ap to station
                        // self.access_point = None;
                        self.station = match self.session.station() {
                            Some(_) => match Station::new(self.session.clone()).await {
                                Ok(v) => Some(v),
                                Err(e) => {
                                    // Notification::send(
                                    //     e.to_string(),
                                    //     crate::notification::NotificationLevel::Error,
                                    //     sender,
                                    // )?;
                                    None
                                }
                            },
                            None => None,
                        };
                    }
                    _ => {}
                }
            }
            Mode::Ap => {
                match self.mode {
                    Mode::Station => {
                        self.station = None;
                        // self.access_point = match self.session.access_point() {
                        //     Some(_) => match AccessPoint::new(self.session.clone()).await {
                        //         Ok(v) => Some(v),
                        //         Err(e) => {
                        //             Notification::send(
                        //                 e.to_string(),
                        //                 crate::notification::NotificationLevel::Error,
                        //                 sender,
                        //             )?;
                        //             None
                        //         }
                        //     },
                        //     None => None,
                        // };
                    }
                    Mode::Ap => {
                        // Switch mode
                        // if self.access_point.is_some() {
                        //     self.access_point.as_mut().unwrap().refresh().await?;
                        // }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        self.mode = current_mode;
        Ok(())
    }
}