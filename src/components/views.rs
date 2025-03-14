use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use color_eyre::Result;
use ratatui::{layout::Rect, Frame};

use crate::action::Action;

pub mod test;
pub mod password;
pub mod ssh;
pub mod locale;

pub use password::PasswordView;
pub use ssh::SshView;
pub use locale::LocaleView;

pub trait ViewComponent {
    fn title(&self) -> &str;
    #[allow(unused_variables)]
    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }

        match key.code {
            KeyCode::Backspace => {},
            _ => {},
        }

        Ok(None)
    }
    #[allow(unused_variables)]
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }
    fn draw(&self, f: &mut Frame<'_>, area: Rect) -> Result<()>;
}