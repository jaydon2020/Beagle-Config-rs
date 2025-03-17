use ratatui::{buffer::Buffer, layout::Rect, style::{palette::tailwind::SLATE, Color, Style}, text::{Line, Text}, widgets::{Block, Paragraph, Widget}};

#[derive(Debug, Clone)]
pub struct TextButtonWidget<'a> {
    label: Line<'a>,
    state: ButtonState,
}

#[derive(Debug, Clone)]
pub struct ButtonWidget<'a> {
    label: Line<'a>,
    state: ButtonState,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Selected,
    Active,
}

/// A button with a label that can be themed.
impl<'a> TextButtonWidget<'a> {
    pub fn new<T: Into<Line<'a>>>(label: T) -> Self {
        TextButtonWidget {
            label: label.into(),
            state: ButtonState::Normal,
        }
    }

    pub const fn state(mut self, state: ButtonState) -> Self {
        self.state = state;
        self
    }
}

/// A button with a label that can be themed.
impl<'a> ButtonWidget<'a> {
    pub fn new<T: Into<Line<'a>>>(label: T) -> Self {
        ButtonWidget {
            label: label.into(),
            state: ButtonState::Normal,
        }
    }

    pub const fn state(mut self, state: ButtonState) -> Self {
        self.state = state;
        self
    }
}

impl<'a> Widget for TextButtonWidget<'a> {
    #[allow(clippy::cast_possible_truncation)]
    fn render(self, area: Rect, buf: &mut Buffer) {
        
        let style = if self.state == ButtonState::Selected {
            Style::new().bg(SLATE.c500)
        } else {
            Style::default()
        };

        buf.set_line(
            area.x + (area.width.saturating_sub(self.label.width() as u16)) / 2,
            area.y + (area.height.saturating_sub(1)) / 2,
            &Line::from_iter(["[", &self.label.to_string(), "]"]).style(style),
            area.width,
        );
    }
}

impl<'a> Widget for ButtonWidget<'a> {
    #[allow(clippy::cast_possible_truncation)]
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = match self.state {
            ButtonState::Normal => Style::new(),
            ButtonState::Selected => Style::new().bg(SLATE.c500),
            ButtonState::Active => Style::new(),
        };
        Paragraph::new(Line::raw(&self.label.to_string()).centered())
            .block(Block::bordered())
            .style(style)
            .render(area, buf);
    }
}