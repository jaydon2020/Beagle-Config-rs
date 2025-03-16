use std::sync::{atomic::AtomicBool, Arc};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::{Constraint, Direction, Layout}, style::{Color, Style}, text::Line, widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph, Widget}};
use tokio::sync::mpsc::{self, UnboundedSender};
use color_eyre::Result;

use crate::{action::Action, networks::adapter::Adapter, tui::Event, widgets::{ButtonState, ButtonWidget}};

use super::ViewComponent;

pub struct WifiView {
    title: String,
    error: Option<String>,
    adapter: Option<Adapter>,
    scan_button_state: ButtonState,
    list_state: ListState,
    focus: Focus,
    sender: Option<UnboundedSender<Action>>,
    is_scanning: bool,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum Focus {
    None,
    Scan,
}

impl WifiView {
    pub async fn init(sender: UnboundedSender<Action>) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let session = {
            match iwdrs::session::Session::new().await {
                Ok(session) => Arc::new(session),
                Err(e) => return Self::error_state(e),
            }
        };

        let adapter = match Adapter::new(session.clone(), event_tx.clone()).await {
            Ok(v) => v,
            Err(e) => return Self::error_state(e),
        };

        // let current_mode = adapter.device.mode.clone();

        // let (passkey_sender, passkey_receiver) = async_channel::unbounded();
        // let show_password = false;
        // let (cancel_signal_sender, cancel_signal_receiver) = async_channel::unbounded();

        // let authentication_required = Arc::new(AtomicBool::new(false));
        // let authentication_required_caller = authentication_required.clone();

        WifiView {
            title: String::from("WiFi"),
            error: None,
            adapter: Some(adapter),
            scan_button_state: ButtonState::Normal,
            list_state: ListState::default(),
            focus: Focus::None,
            sender: Some(sender.clone()),
            is_scanning: false,
        }
    }

    fn error_state(e: impl std::fmt::Display) -> Self {
        Self {
            title: String::from("WiFi"),
            error: Some(format!("Initialization error: {}", e)),
            adapter: None,
            scan_button_state: ButtonState::Normal,
            list_state: ListState::default(),
            focus: Focus::None,
            sender: None,
            is_scanning: false,
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
    pub fn scan(&mut self) {
        let (sender, reciever) = mpsc::unbounded_channel();
        self.is_scanning = true;
        let adapter = self
            .adapter
            .as_ref()
            .unwrap()
            .device.station
            .as_ref()
            .unwrap()
            .clone();

        let sender_clone = sender.clone();
        let tx_clone = self.sender.clone().unwrap();

        tokio::spawn(async move {
            match adapter.scan(sender_clone.clone()).await {
                Ok(networks) => {
                    let _ = tx_clone.send(Action::ScanComplete);
                },
                Err(e) => {
                    let _ = tx_clone.send(Action::ScanComplete);
                }
            }
        });
    }
}

impl ViewComponent for WifiView {
    fn title(&self) -> &str {
        &self.title
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Render => {

            }
            Action::Tick => {
                self.is_scanning = false;
                if let Some(adapter) = &self.adapter {
                    // Clone required values before moving into async
                    let mut station = adapter.device.station.clone()
                        .expect("Station should exist");
                    let sender = self.sender.clone()
                        .expect("Sender should exist");
            
                    tokio::spawn(async move {
                        let result = station.refresh().await;
                        match result {
                            Ok(()) => sender.send(Action::ScanComplete)
                                .expect("Failed to send scan complete"),
                            Err(e) => sender.send(Action::Error(e.to_string()))
                                .expect("Failed to send error"),
                        }
                    });
                }
            }
            Action::ScanComplete => {
                self.is_scanning = false;
                return Ok(Some(Action::Render));
            },
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
                    // self.scan(self.sender.clone()).awit();
                    self.focus = Focus::None;
                    self.update_states();
                    let (sender, reciever) = mpsc::unbounded_channel();
                    self.is_scanning = true;
                    let adapter = self
                        .adapter
                        .as_ref()
                        .unwrap()
                        .device.station
                        .as_ref()
                        .unwrap()
                        .clone();

                    let sender_clone = sender.clone();
                    let tx_clone = self.sender.clone().unwrap();

                    tokio::spawn(async move {
                        match adapter.scan(sender_clone.clone()).await {
                            Ok(networks) => {
                                // return Ok(Some(Action::ScanComplete)).unwrap();
                            },
                            Err(e) => {
                                // return Ok(Some(Action::ScanComplete)).unwrap();
                            }
                        }
                    });

                    // return Ok(Some(Action::ScanComplete));
                    return Ok(Some(Action::Render));
                },
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

            // Network List
            let items: Vec<ListItem> = self.adapter
                .as_ref()
                .unwrap()
                .device
                .station
                .as_ref()
                .unwrap()
                .new_networks
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
        }
        else {
            Paragraph::new(Line::raw(self.error.clone().unwrap()).centered()).render(area, f.buffer_mut());
        }
        Ok(())
    }
}