use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Widget},
    symbols::{self},
};
use crossterm::event::{KeyCode, MouseEventKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchState {
    On,
    Off,
}

#[derive(Debug, Clone)]
pub struct Switch<'a> {
    state: SwitchState,
    labels: (String, String),
    styles: SwitchStyles,
    is_focused: bool,
    is_hovered: bool,
    is_disabled: bool,
    block: Option<Block<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct SwitchStyles {
    pub(crate) active_track: Style,
    pub(crate) inactive_track: Style,
    pub(crate) thumb: Style,
    pub(crate) focused_border: Style,
    pub(crate) hovered_thumb: Style,
    pub(crate) disabled: Style,
}

impl Default for SwitchStyles {
    fn default() -> Self {
        Self {
            active_track: Style::default()
                .fg(Color::DarkGray)
                .bg(Color::LightCyan),
            inactive_track: Style::default()
                .fg(Color::DarkGray)
                .bg(Color::LightCyan)
                .dim(),
            thumb: Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
            focused_border: Style::default().fg(Color::LightBlue),
            hovered_thumb: Style::default()
                .fg(Color::Black)
                .bg(Color::LightBlue),
            disabled: Style::default()
                .fg(Color::DarkGray)
                .bg(Color::Rgb(50, 50, 50)),
        }
    }
}

impl<'a> Switch<'a> {
    const MIN_WIDTH: u16 = 7; // Minimum width for basic visualization
    
    pub fn new(initial_state: SwitchState) -> Self {
        Self {
            state: initial_state,
            labels: ("ON".into(), "OFF".into()),
            styles: SwitchStyles::default(),
            is_focused: false,
            is_hovered: false,
            is_disabled: false,
            block: None,
        }
    }

    #[allow(dead_code)]
    pub fn state(&self) -> SwitchState {
        self.state
    }

    #[allow(dead_code)]
    pub fn set_state(&mut self, state: SwitchState) -> &mut Self {
        if !self.is_disabled {
            self.state = state;
        }
        self
    }

    #[allow(dead_code)]
    pub fn toggle(&mut self) -> &mut Self {
        if !self.is_disabled {
            self.state = match self.state {
                SwitchState::On => SwitchState::Off,
                SwitchState::Off => SwitchState::On,
            };
        }
        self
    }

    pub fn labels(mut self, on: impl Into<String>, off: impl Into<String>) -> Self {
        self.labels = (on.into(), off.into());
        self
    }

    #[allow(dead_code)]
    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    #[allow(dead_code)]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.is_disabled = disabled;
        self
    }

    #[allow(dead_code)]
    pub fn styles(mut self, styles: SwitchStyles) -> Self {
        self.styles = styles;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    #[allow(dead_code)]
    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        if self.is_disabled {
            return false;
        }

        match key {
            KeyCode::Char(' ') | KeyCode::Enter => {
                self.toggle();
                true
            }
            KeyCode::Left => {
                self.set_state(SwitchState::Off);
                true
            }
            KeyCode::Right => {
                self.set_state(SwitchState::On);
                true
            }
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn handle_mouse(&mut self, mouse: MouseEventKind, x: u16, y: u16, area: ratatui::prelude::Rect) -> bool {
        if self.is_disabled {
            return false;
        }

        let is_inside = x >= area.x && x < area.x + area.width &&
            y >= area.y && y < area.y + area.height;

        self.is_hovered = is_inside;

        if let MouseEventKind::Down(_) = mouse {
            if is_inside {
                self.toggle();
                return true;
            }
        }
        false
    }

    #[allow(dead_code)]
    fn calculate_base_width(&self) -> u16 {
        let (on, off) = &self.labels;
        let text_width = on.chars().count().max(off.chars().count()) as u16;
        text_width + 4 // Account for thumb and spacing
    }

    fn calculate_total_width(&self) -> u16 {
        let (on, off) = &self.labels;
        let text_width = on.chars().count().max(off.chars().count()) as u16;
        text_width + 4 // Account for thumb and spacing
    }
}

impl<'a> Widget for Switch<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        if area.width < Self::MIN_WIDTH || area.height < 1 {
            return;
        }

        let block = self.block.clone().unwrap_or_default();
        let inner_area = block.inner(area);
        
        // Calculate required width based on content
        let content_width = self.calculate_total_width();
        let available_width = inner_area.width;
        let render_width = content_width.min(available_width);
        
        // Center the switch in the available space
        let x_offset = (available_width - render_width) / 2;
        let centered_area = ratatui::prelude::Rect {
            x: inner_area.x + x_offset,
            y: inner_area.y,
            width: render_width,
            height: 1,
        };

        // Render block first
        block.render(area, buf);

        let is_active = self.state == SwitchState::On;
        let (on_label, off_label) = &self.labels;

        // Add the missing style definitions
        let base_style = if self.is_disabled {
            self.styles.disabled
        } else if self.is_focused {
            self.styles.focused_border
        } else {
            Style::default()
        };

        let thumb_style = if self.is_hovered && !self.is_disabled {
            self.styles.hovered_thumb
        } else {
            self.styles.thumb
        };

        let track_style = if is_active {
            self.styles.active_track
        } else {
            self.styles.inactive_track
        };

        // Draw track using centered_area
        buf.set_style(centered_area, track_style);

        // Calculate thumb position relative to centered area
        let thumb_width = 3;
        let thumb_position = if is_active {
            centered_area.width - thumb_width
        } else {
            0
        };

        // Draw thumb
        let thumb_area = ratatui::prelude::Rect {
            x: centered_area.x + thumb_position,
            y: centered_area.y,
            width: thumb_width.min(centered_area.width),
            height: 1,
        };

        let thumb_symbol = match self.state {
            SwitchState::On => symbols::DOT.to_string(),
            SwitchState::Off => symbols::DOT.to_string(),
        };

        buf.set_string(
            thumb_area.x + 1,
            thumb_area.y,
            thumb_symbol,
            thumb_style,
        );

        // Draw labels within centered area
        let label = if is_active { on_label } else { off_label };
        let label_x = if is_active {
            centered_area.x + 1
        } else {
            centered_area.x + centered_area.width.saturating_sub(label.len() as u16 + 1)
        };

        buf.set_string(
            label_x,
            centered_area.y,
            label,
            base_style.add_modifier(Modifier::REVERSED),
        );
    }
}