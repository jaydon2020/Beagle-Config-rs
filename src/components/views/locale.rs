use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, widgets::{Block, Borders, Widget}, Frame};
use color_eyre::{eyre::Ok, Result};

use crate::{action::Action, widgets::{Switch, SwitchState}};

use super::ViewComponent;

pub struct LocaleView {
    title: String,
    switch_focus: SwitchState,
}

impl LocaleView {
    pub fn init() -> Self {
        LocaleView {
            title: String::from("locale"),
            switch_focus: SwitchState::Off,
        }
    }
}

impl ViewComponent for LocaleView {
    fn title(&self) -> &str {
        &self.title
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Up => self.switch_focus = SwitchState::Off,
            KeyCode::Down => self.switch_focus = SwitchState::On,
            KeyCode::Backspace => return Ok(Some(Action::BackToMenu)),
            _ => {},
        }

        Ok(None)
    }

    fn draw(&self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ]).split(area);
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(15),
                Constraint::Min(0),
            ]).split(chunks[0]);

        Switch::new(self.switch_focus)
            .labels("ON", "OFF")
            .block(Block::default().borders(Borders::ALL))
            .render(chunks[0], f.buffer_mut());

        Ok(())
    }
}