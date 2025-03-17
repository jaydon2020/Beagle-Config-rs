use std::sync::{atomic::AtomicBool, Arc};

use anyhow::anyhow;
use async_channel::Receiver;
use crossterm::event::{KeyCode, KeyEvent};
use futures::{future::ok, FutureExt};
use iwdrs::{agent::Agent, modes::Mode, session::Session};
use ratatui::{layout::*, style::*, text::*, widgets::*};
use color_eyre::Result;
use strum::Display;
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::{action::Action, app::AppResult, networks::{adaptor::Adapter, network::Network, rfkill}, widgets::{ButtonState, ButtonWidget}};

use super::ViewComponent;

#[derive(Debug, Clone)]
pub struct WifiView {
    title: String,
    error: Option<String>,
    iwd_wifi: Option<ImplWiFi>,
    sender: Option<UnboundedSender<Action>>,
    is_scanning: bool,
    focus: Focus,
    scan_button_state: ButtonState,
    list_state: ListState,
    sorted_networks: Vec<(Network, i16)>,
    tick_count: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Focus {
    None,
    Scan,
    List,
}

#[derive(Debug, Clone)]
pub struct ImplWiFi {
    pub session: Arc<Session>,
    pub adapter: Adapter,
    pub current_mode: Mode,
    pub agent_manager: iwdrs::agent::AgentManager,
}

impl PartialEq for ImplWiFi {
    fn eq(&self, other: &Self) -> bool {
        // Custom equality logic here (e.g., compare specific fields)
        true
    }
}

pub async fn request_confirmation(
    authentication_required: Arc<AtomicBool>,
    rx_key: Receiver<String>,
    rx_cancel: Receiver<()>,
) -> Result<String, Box<dyn std::error::Error>> {
    authentication_required.store(true, std::sync::atomic::Ordering::Relaxed);

    tokio::select! {
    r = rx_key.recv() =>  {
            match r {
                Ok(key) => Ok(key),
                Err(_) => Err(anyhow!("Failed to receive the key").into()),
            }
        }

    r = rx_cancel.recv() => {
            match r {
                Ok(_) => {
                        Err(anyhow!("Operation Canceled").into())},
                Err(_) => Err(anyhow!("Failed to receive cancel signal").into()),
            }

        }

    }
}

impl WifiView {
    pub async fn init(sender: mpsc::UnboundedSender<Action>) -> Self {
        match rfkill::check() {
            Ok(_) => {},
            Err(e) => return Self::error_state(e),
        };
        
        let session = {
            match iwdrs::session::Session::new().await {
                Ok(session) => Arc::new(session),
                Err(e) => return Self::error_state(e),
            }
        };

        let adapter = match Adapter::new(session.clone()).await {
            Ok(v) => v,
            Err(e) => return Self::error_state(e),
        };
        let current_mode = adapter.device.mode.clone();

        let (passkey_sender, passkey_receiver) = async_channel::unbounded();
        let show_password = false;
        let (cancel_signal_sender, cancel_signal_receiver) = async_channel::unbounded();

        let authentication_required = Arc::new(AtomicBool::new(false));
        let authentication_required_caller = authentication_required.clone();

        let agent = Agent {
            request_passphrase_fn: Box::new(move || {
                {
                    let auth_clone = authentication_required_caller.clone();
                    request_confirmation(
                        auth_clone,
                        passkey_receiver.clone(),
                        cancel_signal_receiver.clone(),
                    )
                }
                .boxed()
            }),
        };

        let agent_manager = match session.register_agent(agent).await {
            Ok(v) => v,
            Err(e) => return Self::error_state(e),
        };
        
        let impl_wi_fi = ImplWiFi {
            session,
            adapter,
            current_mode,
            agent_manager,
        };

        // Generate sorted network list
        let sorted_networks = impl_wi_fi.adapter.device.station.as_ref()
            .map(|station| {
                let mut networks = station.new_networks.clone();
                networks.sort_by(|a, b| b.1.cmp(&a.1)); // Descending sort by signal
                networks
            })
            .unwrap_or_default();

        WifiView {
            title: String::from("WiFi"),
            error: None,
            iwd_wifi: Some(impl_wi_fi),
            is_scanning: false,
            focus: Focus::None,
            scan_button_state: ButtonState::Normal,
            list_state: ListState::default(),
            sender: Some(sender),
            sorted_networks,
            tick_count: 0_u8,
        }
    }

    fn error_state(e: impl std::fmt::Display) -> Self {
        WifiView {
            title: String::from("WiFi"),
            error: Some(format!("Initialization error: {}", e)),
            iwd_wifi: None,
            is_scanning: false,
            focus: Focus::None,
            scan_button_state: ButtonState::Normal,
            list_state: ListState::default(),
            sender: None,
            sorted_networks: Vec::new(),
            tick_count: 0_u8,
        }
    }

    fn move_focus_up(&mut self) {
        self.focus = match self.focus {
            Focus::List => {
                if let Some(0) = self.list_state.selected() {
                    // Top of list, move to Scan button
                    Focus::Scan
                } else {
                    // Move up in list
                    self.move_list_selection(-1);
                    Focus::List
                }
            }
            Focus::Scan => Focus::None,
            Focus::None => Focus::None,
        };
        self.update_states();
    }
    
    fn move_focus_down(&mut self) {
        self.focus = match self.focus {
            Focus::None => Focus::Scan,
            Focus::Scan => {
                // Initialize list selection if empty
                if self.list_state.selected().is_none() && !self.sorted_networks.is_empty() {
                    self.list_state.select(Some(0));
                }
                Focus::List
            }
            Focus::List => {
                if let Some(selected) = self.list_state.selected() {
                    if selected < self.sorted_networks.len() - 1 {
                        self.move_list_selection(1);
                    }
                }
                Focus::List
            }
        };
        self.update_states();
    }
    
    fn move_list_selection(&mut self, offset: i32) {
        let current = self.list_state.selected().unwrap_or(0) as i32;
        let new_index = current + offset;
        
        if new_index >= 0 && new_index < self.sorted_networks.len() as i32 {
            self.list_state.select(Some(new_index as usize));
        }
    }

    fn update_states(&mut self) {
        self.scan_button_state = if self.focus == Focus::Scan {
            ButtonState::Selected
        } else {
            ButtonState::Normal
        };
    }
}

impl ViewComponent for WifiView {
    fn title(&self) -> &str {
        &self.title
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::UpdateWifiState(impl_wi_fi) => {
                // Generate sorted network list
                let sorted_networks = impl_wi_fi.adapter.device.station.as_ref()
                    .map(|station| {
                        let mut networks = station.new_networks.clone();
                        networks.sort_by(|a, b| b.1.cmp(&a.1)); // Descending sort by signal
                        networks
                    })
                    .unwrap_or_default();
                
                self.sorted_networks = sorted_networks;
                self.iwd_wifi = Some(impl_wi_fi);
            }
            Action::ScanComplete => {
                self.is_scanning = false;
                let iwd_wifi_clone = self.iwd_wifi.clone().unwrap();
                let mut adapter_clone = iwd_wifi_clone.adapter.clone();
                let sender = self.sender.clone().unwrap();
                let current_mode = adapter_clone.device.mode.clone();

                // In your async block
                tokio::spawn(async move {
                    match adapter_clone.refresh().await {
                        Ok(_) => {
                            let new_impl = ImplWiFi {
                                session: iwd_wifi_clone.session,
                                adapter: adapter_clone,
                                current_mode: current_mode,
                                agent_manager: iwd_wifi_clone.agent_manager,
                            };

                            // Send the update through the channel
                            let _ = sender.send(Action::UpdateWifiState(new_impl));
                        }
                        Err(e) => {
                            let _ = sender.send(Action::Error(format!("Refresh failed: {}", e)));
                        }
                    }
                });
            }
            Action::Tick => {
                if self.tick_count <= 180 {
                    self.tick_count += 1;
                    return Ok(None);
                }
                self.tick_count = 0_u8;
                let iwd_wifi_clone = self.iwd_wifi.clone().unwrap();
                let mut adapter_clone = iwd_wifi_clone.adapter.clone();
                let sender = self.sender.clone().unwrap();
                let current_mode = adapter_clone.device.mode.clone();

                // In your async block
                tokio::spawn(async move {
                    match adapter_clone.refresh().await {
                        Ok(_) => {
                            let new_impl = ImplWiFi {
                                session: iwd_wifi_clone.session,
                                adapter: adapter_clone,
                                current_mode: current_mode,
                                agent_manager: iwd_wifi_clone.agent_manager,
                            };

                            // Send the update through the channel
                            let _ = sender.send(Action::UpdateWifiState(new_impl));
                        }
                        Err(e) => {
                            let _ = sender.send(Action::Error(format!("Refresh failed: {}", e)));
                        }
                    }
                });
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Up => self.move_focus_up(),
            KeyCode::Down => self.move_focus_down(),
            KeyCode::Backspace => { return Ok(Some(Action::BackToMenu)); },
            KeyCode::Enter => match self.focus {
                Focus::None => {},
                Focus::Scan => { 
                    let iwd_wifi_clone = self.iwd_wifi.clone().unwrap();
                    let station_clone = iwd_wifi_clone.adapter.device.station.clone().unwrap();
                    let sender = self.sender.clone().unwrap();

                    self.is_scanning = true;

                    // In your async block
                    tokio::spawn(async move {
                        match station_clone.scan().await {
                            Ok(_) => {
                                // Send the update through the channel
                                let _ = sender.send(Action::ScanComplete);
                            }
                            Err(e) => {
                                let _ = sender.send(Action::Error(format!("Refresh failed: {}", e)));
                            }
                        }
                    });
                },
                Focus::List => {},
            }
            _ => {},
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut ratatui::Frame<'_>, area: ratatui::prelude::Rect) -> color_eyre::eyre::Result<()> {
        let area = Block::default().padding(Padding::horizontal(2)).inner(area);

        if self.error.is_none() {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Header
                    Constraint::Length(5),  // Know Network List
                    Constraint::Min(3),     // Network list
                    Constraint::Length(3), // Status bar
                ])
                .split(area);
            
            let scan_btn = ButtonWidget::new(
                if self.is_scanning {
                    "Scanning..."
                } else {
                    "Scan"
                }
            )
                .state(self.scan_button_state);

            // Know Network List
            let know_network = self.iwd_wifi.as_ref().unwrap().adapter.device.station.as_ref()
                    .map(|station| {
                        let mut networks = station.known_networks.clone();
                        networks.sort_by(|a, b| b.1.cmp(&a.1)); // Descending sort by signal
                        networks
                    })
                    .unwrap_or_default();

            let items: Vec<ListItem> = know_network
                .iter()
                .enumerate()
                .map(|(i, (net, signal))| {
                    let line = Line::from(vec![
                        format!("{}    ", net.name).into(),
                        Span::raw({
                            let signal = {
                                if *signal / 100 >= -50 {
                                    100
                                } else {
                                    2 * (100 + signal / 100)
                                }
                            };
                            match signal {
                                n if n >= 75 => format!("{:3}% 󰤨", signal),
                                n if (50..75).contains(&n) => format!("{:3}% 󰤥", signal),
                                n if (25..50).contains(&n) => format!("{:3}% 󰤢", signal),
                                _ => format!("{:3}% 󰤟", signal),
                            }
                }       ),
                    ]);

                    ListItem::new(line).style(Style::default())
                })
                .collect();

            let know_list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Known Networks"))
                .highlight_style(Style::default().bg(Color::DarkGray));

            // Network List
            let items: Vec<ListItem> = self.sorted_networks
                .iter()
                .enumerate()
                .map(|(i, (net, signal))| {
                    let style = if self.list_state.selected() == Some(i) {
                        Style::default()
                            .bg(Color::DarkGray)
                            .fg(Color::White)
                    } else {
                        Style::default()
                    };
                    
                    let line = Line::from(vec![
                        format!("{}    ", net.name).into(),
                        Span::raw({
                            let signal = {
                                if *signal / 100 >= -50 {
                                    100
                                } else {
                                    2 * (100 + signal / 100)
                                }
                            };
                            match signal {
                                n if n >= 75 => format!("{:3}% 󰤨", signal),
                                n if (50..75).contains(&n) => format!("{:3}% 󰤥", signal),
                                n if (25..50).contains(&n) => format!("{:3}% 󰤢", signal),
                                _ => format!("{:3}% 󰤟", signal),
                            }
                        }),
                    ]);

                    ListItem::new(line).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Networks"))
                .highlight_style(Style::default().bg(Color::DarkGray));
            
            f.render_widget(know_list, layout[1]);
            f.render_stateful_widget(list, layout[2], &mut self.list_state);
            f.render_widget(scan_btn, layout[0]);
        }
        else {
            Paragraph::new(Line::raw(self.error.clone().unwrap()).centered()).render(area, f.buffer_mut());
        }
        Ok(())
    }
}