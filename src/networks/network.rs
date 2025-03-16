use iwdrs::netowrk::Network as iwdNetwork;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    app::AppResult,
    networks::known_network::KnownNetwork, tui::Event
};

use super::notification::{Notification, NotificationLevel};

#[derive(Debug, Clone)]
pub struct Network {
    pub n: iwdNetwork,
    pub name: String,
    pub netowrk_type: String,
    pub is_connected: bool,
    pub known_network: Option<KnownNetwork>,
}

impl Network {
    pub async fn new(n: iwdNetwork) -> AppResult<Self> {
        let name = n.name().await?;
        let netowrk_type = n.network_type().await?;
        let is_connected = n.connected().await?;
        let known_network = {
            match n.known_network().await {
                Ok(v) => match v {
                    Some(net) => Some(KnownNetwork::new(net).await.unwrap()),
                    None => None,
                },
                Err(_) => None,
            }
        };

        Ok(Self {
            n,
            name,
            netowrk_type,
            is_connected,
            known_network,
        })
    }

    pub async fn connect(&self, sender: UnboundedSender<Event>) -> AppResult<()> {
        match self.n.connect().await {
            Ok(_) => Notification::send(
                format!("Connected to {}", self.name),
                NotificationLevel::Info,
                sender,
            )?,
            Err(e) => {
                if e.to_string().contains("net.connman.iwd.Aborted") {
                    Notification::send(
                        "Connection canceled".to_string(),
                        NotificationLevel::Info,
                        sender,
                    )?
                } else {
                    Notification::send(e.to_string(), NotificationLevel::Error, sender)?
                }
            }
        }
        Ok(())
    }
}
