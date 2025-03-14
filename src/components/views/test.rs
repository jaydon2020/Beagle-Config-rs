use color_eyre::eyre::Ok;
use ratatui::{text::Line, widgets::{Block, Borders, Paragraph, Widget}};
use color_eyre::Result;
use super::ViewComponent;

// #[derive(Default)]
pub(crate) struct TestViewComponent {
    title: String
}

impl Default for TestViewComponent {
    fn default() -> Self {
        Self { title: Default::default() }
    }
}

impl TestViewComponent {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }
}

impl ViewComponent for TestViewComponent {
    fn title(&self) -> &str {
        &self.title
    }
    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<crate::action::Action>> {
        Ok(None)
    }
    fn draw(&self, f: &mut ratatui::Frame<'_>, area: ratatui::prelude::Rect) -> Result<()> {
        Paragraph::new(Line::from_iter([" ", self.title.as_str(), ": ", "Component"]))
            .block(Block::default().borders(Borders::BOTTOM))
            .render(area, f.buffer_mut());
        Ok(())
    }
}