use crate::app::AppResult;
use iwdrs::netowrk::Network as iwdNetwork;

use super::know_network::KnownNetwork;

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

    pub async fn connect(&self) -> AppResult<()> {
        match self.n.connect().await {
            Ok(_) => {},
            Err(e) => {}
        }
        Ok(())
    }
}
