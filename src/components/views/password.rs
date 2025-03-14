use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::*, style::{Color, Style, Stylize}, text::*, widgets::{Block, Padding, Paragraph, Widget}, Frame};
use color_eyre::eyre::Result;

use crate::{action::Action, widgets::{ButtonState, ButtonWidget}};

use super::ViewComponent;

#[derive(Debug, PartialEq, Eq)]
enum Focus {
    None,
    OldPassword,
    NewPassword,
    ShowPassword,
    ApplyButton,
    BackButton,
}

pub struct PasswordView {
    title: String,
    username: String,
    old_input: String,
    new_input: String,
    focus: Focus,
    editing: bool,
    button_state: ButtonState,
    show_password: bool,
    back_button_state: ButtonState,
}

enum InputMode {
    Normal,
    Editing,
}

impl PasswordView {
    pub fn init() -> Self {
        // let user = whoami::username();
        let user = String::from("Debian");
        Self {
            title: String::from("Password"),
            username: user,
            old_input: String::new(),
            new_input: String::new(),
            focus: Focus::None,
            editing: false,
            button_state: ButtonState::Normal,
            show_password: false,
            back_button_state: ButtonState::Normal,
        }
    }

    fn move_focus_up(&mut self) {
        self.focus = match self.focus {
            Focus::BackButton => Focus::ApplyButton,
            Focus::ApplyButton => Focus::ShowPassword,
            Focus::ShowPassword => Focus::NewPassword,
            Focus::NewPassword => Focus::OldPassword,
            Focus::OldPassword => Focus::OldPassword, // Wrap around
            Focus::None => Focus::OldPassword,
        };
        self.update_states();
    }

    fn move_focus_down(&mut self) {
        self.focus = match self.focus {
            Focus::None => Focus::OldPassword,
            Focus::OldPassword => Focus::NewPassword,
            Focus::NewPassword => Focus::ShowPassword,
            Focus::ShowPassword => Focus::ApplyButton,
            Focus::ApplyButton => Focus::BackButton, // Wrap around
            Focus::BackButton => Focus::BackButton
        };
        self.update_states();
    }
    fn update_states(&mut self) {
        self.editing = matches!(self.focus, Focus::OldPassword | Focus::NewPassword);
        self.button_state = if self.focus == Focus::ApplyButton {
            ButtonState::Selected
        } else {
            ButtonState::Normal
        };
        self.back_button_state = if self.focus == Focus::BackButton {
            ButtonState::Selected
        } else {
            ButtonState::Normal
        };
    }
    fn toggle_show_password(&mut self) {
        self.show_password = !self.show_password;
    }
    fn clear_input(&mut self) {
        self.new_input.clear();
        self.old_input.clear();
    }
}

impl ViewComponent for PasswordView {
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
                Focus::ShowPassword => self.toggle_show_password(),
                Focus::OldPassword | Focus::NewPassword => {
                    self.editing = !self.editing;
                }
                Focus::ApplyButton => {
                    // Handle password change logic here
                    // return Ok(Some(Action::PasswordChange(
                    //     self.old_input.clone(),
                    //     self.new_input.clone(),
                    // )));
                }
                Focus::BackButton => {
                    self.clear_input();
                    self.focus = Focus::None;
                    self.update_states();
                    return Ok(Some(Action::BackToMenu));
                }
            },
            KeyCode::Esc => {
                self.editing = false;
                self.button_state = ButtonState::Normal;
            }
            KeyCode::Char(c) if self.editing => match self.focus {
                Focus::OldPassword => self.old_input.push(c),
                Focus::NewPassword => self.new_input.push(c),
                _ => {}
            },
            KeyCode::Backspace if self.editing => match self.focus {
                Focus::OldPassword => {
                    self.old_input.pop();
                }
                Focus::NewPassword => {
                    self.new_input.pop();
                }
                _ => {
                    // self.command_tx.as_ref().unwrap().send(Action::BackToMenu);
                    return Ok(Some(Action::BackToMenu));
                }
            },
            _ => {}
        }

        // Update button state
        self.button_state = if self.focus == Focus::ApplyButton {
            ButtonState::Selected
        } else {
            ButtonState::Normal
        };

        Ok(None)
    }

    fn draw(&self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let area = Block::new().padding(Padding::horizontal(2)).inner(area);
        let [
            user_area, 
            old_area, 
            new_area, 
            tick_area, 
            button_area, 
        ] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(3),
            ])
            .areas(area);

        // User display
        let user_block = Paragraph::new(Line::from_iter([
            Span::styled("User: ", Style::new().bold()),
            Span::raw(&self.username),
        ]));
        f.render_widget(user_block, user_area);

        let [text_area, input_area] = Layout::horizontal([
            Constraint::Length(14),
            Constraint::Min(1),
        ]).areas(old_area);
        let text_area = Block::bordered().inner(text_area);
        Paragraph::new(Line::raw("Old Password")
            .style(Style::new().bold()))
            .render(text_area, f.buffer_mut());

        // Old password input
        let old_style = match self.focus {
            Focus::OldPassword => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        };
        let old_input = Paragraph::new(self.old_input.as_str())
            .style(if self.editing && self.focus == Focus::OldPassword {
                old_style.fg(Color::Green)
            } else {
                old_style
            })
            .block(
                Block::bordered()
                    // .title(" Old Password ")
                    .border_style(if self.focus == Focus::OldPassword {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    }),
            );
        f.render_widget(old_input, input_area);
        
        match self.focus {
            Focus::NewPassword => Block::bordered().render(new_area, f.buffer_mut()),
            _ => Block::default().render(new_area, f.buffer_mut())
        }
        let new_area = Block::bordered().inner(new_area);
        let [text_area, input_area] = Layout::horizontal([
            Constraint::Length(14),
            Constraint::Min(1),
        ]).areas(new_area);
        Paragraph::new(Line::raw("New Password")
            .style(Style::new().bold()))
            .render(text_area, f.buffer_mut());
        // New password input
        let new_style = match self.focus {
            Focus::NewPassword => Style::default().bg(Color::LightYellow),
            _ => Style::default(),
        };
        let new_input = Paragraph::new(self.new_input.as_str())
            .style(if self.editing && self.focus == Focus::NewPassword {
                new_style.fg(Color::Green)
            } else {
                new_style
            }); 
            // .block(
            //     Block::new()
            //         // .title(" New Password ")
            //         .border_style(if self.focus == Focus::NewPassword {
            //             Style::default().bg(Color::Yellow)
            //         } else {
            //             Style::default()
            //         }),
            // );
        f.render_widget(new_input, input_area);
        
        let [checkbox_area, label_area] = Layout::horizontal([
            Constraint::Length(3),
            Constraint::Min(1),
        ]).areas(tick_area);

        let checkbox = Paragraph::new(if self.show_password { "[x] " } else { "[ ] " })
        .style(if self.focus == Focus::ShowPassword {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

        let label = Paragraph::new("Show passwords")
            .style(if self.focus == Focus::ShowPassword {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        f.render_widget(checkbox, checkbox_area);
        f.render_widget(label, label_area);
        
        let [apply_area, back_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]).areas(button_area);

        // Apply button
        let button = ButtonWidget::new("Apply")
            .state(match self.button_state {
                ButtonState::Normal => ButtonState::Normal,
                ButtonState::Selected => ButtonState::Selected,
                ButtonState::Active => ButtonState::Active,
            });
        f.render_widget(button, apply_area);
        
        let back_button = ButtonWidget::new("Back")
        .state(match self.back_button_state {
            ButtonState::Normal => ButtonState::Normal,
            ButtonState::Selected => ButtonState::Selected,
            ButtonState::Active => ButtonState::Active,
        });
        f.render_widget(back_button, back_area);

        Ok(())
    }
}