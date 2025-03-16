use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use anyhow::anyhow;
use anyhow::Context;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use futures::future::join_all;
use futures::FutureExt;
use iwdrs::{adapter::Adapter, agent::{Agent, AgentManager}, session::Session};
use ratatui::{layout::*, style::*, widgets::*, Frame};
use tokio::sync::mpsc::UnboundedSender;
use color_eyre::Result;
use async_channel::{Receiver, Sender};
use crate::action::Action;
use crate::networks::network::Network;
use crate::networks::notification::Notification;
use crate::networks::notification::NotificationLevel;
use crate::{app::AppResult, tui::Event, widgets::{ButtonState, ButtonWidget}};

use super::ViewComponent;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum Focus {
    None,
    Scan,
}

pub struct WifiView {
    title: String,
    sender: UnboundedSender<Event>,
    session: Arc<Session>,
    networks: Vec<(Network, i16)>,
    is_scanning: bool,
    list_state: ListState,
    error: Option<String>,
    adapter: Adapter,
    agent_manager: AgentManager,
    focus: Focus,
    scan_button_state: ButtonState,
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
    pub async fn init(sender: UnboundedSender<Event>) -> AppResult<Self> {
        let title = String::from("Wifi");
        let session = Arc::new(Session::new().await?);
        let adapter = session.adapter().context("No adapter found")?;
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
        
        let agent_manager = session.register_agent(agent).await?;
        
        // let ap_iwrds = session.access_point().unwrap();
        let iwd_access_point_diagnotic = session.access_point_diagnostic();

        let iwd_station = session.station().unwrap();
        let iwd_station_diagnostic = session.station_diagnostic();
        let discovered_networks = iwd_station.discovered_networks().await?;
        let is_scanning = iwd_station.is_scanning().await?;
        // Get initial networks
        let networks = {
            let collected_futures = discovered_networks
                .iter()
                .map(|(n, signal)| async {
                    match Network::new(n.clone()).await {
                        Ok(network) => Ok((network, signal.to_owned())),
                        Err(e) => Err(e),
                    }
                })
                .collect::<Vec<_>>();
            let results = join_all(collected_futures).await;
            results
                .into_iter()
                .filter_map(Result::ok)
                .collect::<Vec<(Network, i16)>>()
        };
        // Get initial networks
        // let networks = match adapter.().await {
        //     Ok(nets) => nets,
        //     Err(e) => {
        //         return Ok(Self {
        //             title,
        //             session,
        //             networks: Vec::new(),
        //             scan_state: ButtonState::Normal,
        //             list_state: ListState::default(),
        //             error: Some(format!("Initial scan failed: {}", e)),
        //             adapter,
        //             agent_manager,
        //         })
        //     }
        // };

        // let connected_devices = {
        //     if let Some(d) = iwd_access_point_diagnotic {
        //         match d.get().await {
        //             Ok(diagnostic) => diagnostic
        //                 .iter()
        //                 .map(|v| v["Address"].clone().trim_matches('"').to_string())
        //                 .collect(),
        //             Err(_) => Vec::new(),
        //         }
        //     } else {
        //         Vec::new()
        //     }
        // };

        Ok(Self {
            title,
            sender,
            session,
            networks: networks,
            is_scanning,
            list_state: ListState::default(),
            error: None,
            adapter,
            agent_manager,
            focus: Focus::None,
            scan_button_state: ButtonState::Normal,
        })
    }
    pub fn handle_event(&mut self, event: &Event) {
        match event {
            // Event::ScanComplete(networks) => {
            //     self.scan_state = ButtonState::Normal;
            //     self.networks = networks.clone();
            //     self.error = None;
            // }
            Event::ScanError(e) => {
                self.scan_button_state = ButtonState::Normal;
                self.error = e.clone();
            }
            _ => {}
        }
    }

    fn move_focus_up(&mut self) {
        self.focus = match self.focus {
            Focus::Scan => Focus::None,
            Focus::None => Focus::None,
        };
        self.update_states();
    }
    fn move_focus_down(&mut self) {
        self.focus = match self.focus {
            Focus::None => Focus::Scan,
            Focus::Scan => Focus::Scan,
        };
        self.update_states();
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
    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Up => self.move_focus_up(),
            KeyCode::Down => self.move_focus_down(),
            KeyCode::Backspace => { return Ok(Some(Action::BackToMenu)); },
            KeyCode::Enter => match self.focus {
                Focus::None => {},
                Focus::Scan => {
                    // self.scan(self.sender.clone()).awit();
                    self.focus = Focus::None;
                    self.update_states();
                    return Ok(Some(Action::Scan));
                },
            }
            _ => {},
        }
        Ok(None)
    }
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let area = Block::default().padding(Padding::horizontal(2)).inner(area);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(3),     // Network list
                Constraint::Length(3), // Status bar
            ])
            .split(area);

        // Scan Button
        let button_label = if self.is_scanning {
            "Scanning ... "
        } else {
            "󰖩  Scan Networks"
        };
        let scan_btn = ButtonWidget::new(button_label)
            .state(self.scan_button_state);

        // Network List
        let items: Vec<ListItem> = self.networks
            .iter()
            .map(|(net, signal)| {
                ListItem::new(net.name.clone())
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Networks"))
            .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_stateful_widget(list, layout[1], &mut self.list_state);
        f.render_widget(scan_btn, layout[0]);

        // Error display
        if let Some(err) = &self.error {
            let popup = Paragraph::new(err.clone())
                .block(Block::default().borders(Borders::ALL).title("Error"))
                .style(Style::default().fg(Color::Red));
            f.render_widget(popup, area);
        }

        Ok(())
    }
}

