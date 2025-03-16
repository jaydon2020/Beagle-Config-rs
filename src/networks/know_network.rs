use chrono::{DateTime, FixedOffset};
use iwdrs::known_netowk::KnownNetwork as iwdKnownNetwork;

use crate::app::AppResult;

#[derive(Debug, Clone)]
pub struct KnownNetwork {
    pub n: iwdKnownNetwork,
    pub name: String,
    pub netowrk_type: String,
    pub is_autoconnect: bool,
    pub is_hidden: bool,
    pub last_connected: Option<DateTime<FixedOffset>>,
}

impl KnownNetwork {
    pub async fn new(n: iwdKnownNetwork) -> AppResult<Self> {
        let name = n.name().await?;
        let netowrk_type = n.network_type().await?;
        let is_autoconnect = n.get_autoconnect().await?;
        let is_hidden = n.hidden().await?;
        let last_connected = match n.last_connected_time().await {
            Ok(v) => DateTime::parse_from_rfc3339(&v).ok(),
            Err(_) => None,
        };

        Ok(Self {
            n,
            name,
            netowrk_type,
            is_autoconnect,
            is_hidden,
            last_connected,
        })
    }

    pub async fn forget(&self) -> AppResult<()> {
        if let Err(e) = self.n.forget().await {
            return Ok(());
        }
        Ok(())
    }

    pub async fn toggle_autoconnect(&self) -> AppResult<()> {
        if self.is_autoconnect {
            match self.n.set_autoconnect(false).await {
                Ok(_) => {

                }
                Err(e) => {

                }
            }
        } else {
            match self.n.set_autoconnect(true).await {
                Ok(_) => {

                }
                Err(e) => {

                }
            }
        }
        Ok(())
    }
}
