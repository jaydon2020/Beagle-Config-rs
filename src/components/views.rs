use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use color_eyre::Result;
use ratatui::{layout::Rect, Frame};

use crate::action::Action;

pub mod test;
pub mod password;
pub mod ssh;
pub mod locale;
pub mod wifi;

pub use password::PasswordView;
pub use ssh::SshView;
pub use locale::LocaleView;
pub use wifi::WifiView;
pub use test::TestViewComponent;

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
    #[allow(dead_code)]
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()>;
}