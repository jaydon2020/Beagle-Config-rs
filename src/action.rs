use serde::{Deserialize, Serialize};
use strum::Display;

use crate::components::views::wifi::ImplWiFi;

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    BackToMenu,
    ScanComplete,
    #[serde(skip)]
    UpdateWifiState(ImplWiFi),
}
