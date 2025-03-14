use crate::{action::Action, widgets::{ButtonState, ButtonWidget}};
use color_eyre::{eyre::Ok, Result};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, text::{Line, Span}, widgets::{Block, Padding, Paragraph}};

use super::ViewComponent;

pub struct SshView {
    title: String,
    status: String,
    focus: Focus,
    en_button_state: ButtonState,
    di_button_state: ButtonState,
    ba_button_state: ButtonState,
}

#[derive(PartialEq, Clone, Debug)]
enum Focus {
    None,
    Enable,
    Disable,
    Back,
}

impl SshView {
    pub fn init() -> Self {
        let status = String::from("Active");
        SshView {
            title: String::from("SSH"),
            status,
            focus: Focus::None,
            en_button_state: ButtonState::Normal,
            di_button_state: ButtonState::Normal,
            ba_button_state: ButtonState::Normal,
        }
    }
    fn move_focus_up(&mut self) {
        self.focus = match self.focus {
            Focus::Back => Focus::Disable,
            Focus::Disable => Focus::Enable,
            Focus::Enable => Focus::Enable,
            Focus::None => Focus::Enable,
        };
        self.update_states();
    }
    fn move_focus_down(&mut self) {
        self.focus = match self.focus {
            Focus::None => Focus::Enable,
            Focus::Enable => Focus::Disable,
            Focus::Disable => Focus::Back,
            Focus::Back => Focus::Back,
        };
        self.update_states();
    }
    fn update_states(&mut self) {
        self.en_button_state = if self.focus == Focus::Enable {
            ButtonState::Selected
        } else {
            ButtonState::Normal
        };
        self.di_button_state = if self.focus == Focus::Disable {
            ButtonState::Selected
        } else {
            ButtonState::Normal
        };
        self.ba_button_state = if self.focus == Focus::Back {
            ButtonState::Selected
        } else {
            ButtonState::Normal
        };
    }
}

impl ViewComponent for SshView {
    fn title(&self) -> &str {
        &self.title
    }
    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Up => self.move_focus_up(),
            KeyCode::Down => self.move_focus_down(),
            KeyCode::Left => self.move_focus_up(),
            KeyCode::Right => self.move_focus_down(),
            KeyCode::Enter => match self.focus {
                Focus::None => {},
                Focus::Enable => {},
                Focus::Disable => {},
                Focus::Back => {
                    self.focus = Focus::None;
                    self.update_states();
                    return Ok(Some(Action::BackToMenu));
                }
            },
            KeyCode::Backspace => {
                self.focus = Focus::None;
                self.update_states();
                return Ok(Some(Action::BackToMenu));
            },
            _ => {}
        }
        Ok(None)
    }

    fn draw(&self, f: &mut ratatui::Frame<'_>, area: Rect) -> Result<()> {
        let area = Block::new().padding(Padding::horizontal(2)).inner(area);
        let [status_area, button_area, back_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(3),
            ]).areas(area);
        let status = Paragraph::new(Line::from_iter([
            format!("Status: ").into(),
            Span::raw(&self.status),
        ]));

        f.render_widget(status, status_area);
        
        let [en_area, di_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]).areas(button_area);

        let enable = ButtonWidget::new("Enable").state(self.en_button_state);
        let disable = ButtonWidget::new("Disable").state(self.di_button_state);
        let back = ButtonWidget::new("Back").state(self.ba_button_state);
        
        f.render_widget(enable, en_area);
        f.render_widget(disable, di_area);
        f.render_widget(back, back_area);

        Ok(())
    }
}