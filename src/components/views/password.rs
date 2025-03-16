use std::process::{Command, Stdio};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::*, style::{palette::tailwind::SLATE, Color, Style, Stylize}, text::*, widgets::{Block, BorderType, Padding, Paragraph, Widget}, Frame};
use color_eyre::eyre::Result;
use std::io::Write;

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
    update_active: bool,
    error_message: Option<String>,
}

#[allow(unused)]
enum InputMode {
    Normal,
    Editing,
}

impl PasswordView {
    pub fn init() -> Self {
        let user = whoami::username();
        // let user = String::from("Debian");
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
            update_active: false,
            error_message: None,
        }
    }

    fn move_focus_up(&mut self) {
        if self.update_active {
            return;
        }
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
        if self.update_active {
            return;
        }
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
                Focus::None => {
                    if self.update_active {
                        self.update_active = false;
                        return Ok(Some(Action::BackToMenu));
                    }
                },
                Focus::ShowPassword => self.toggle_show_password(),
                Focus::OldPassword | Focus::NewPassword => {
                    self.editing = !self.editing;
                }
                Focus::ApplyButton => {
                    self.update_active = true;
                    self.focus = Focus::None;
                
                    // Clear previous errors
                    self.error_message = None;
                
                    // Validate inputs
                    if self.old_input.is_empty() || self.new_input.is_empty() {
                        self.error_message = Some("Passwords cannot be empty".to_string());
                        self.clear_input();
                        return Ok(None);
                    }

                    // Create sudo command
                    let _ = match Command::new("sudo")
                        .arg("-k")  // Read password from stdin
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::piped())
                        .spawn() 
                    {
                        Ok(child) => child,
                        Err(e) => {
                            self.error_message = Some(format!("Failed to start sudo: {}", e));
                            self.clear_input();
                            return Ok(None);
                        }
                    };

                    // Create sudo command
                    let mut child = match Command::new("sudo")
                        .arg("-S")  // Read password from stdin
                        .arg("chpasswd")
                        .stdin(Stdio::piped())
                        .stdout(Stdio::null())
                        .stderr(Stdio::piped())
                        .spawn() 
                    {
                        Ok(child) => child,
                        Err(e) => {
                            self.error_message = Some(format!("Failed to start sudo: {}", e));
                            self.clear_input();
                            return Ok(None);
                        }
                    };
                
                    // Prepare input with proper newline separation
                    let input = format!(
                        "{}\n{}:{}\n",  // Note: \n after sudo password AND after chpasswd input
                        self.old_input,
                        self.username,
                        self.new_input
                    );
                
                    // Write to stdin
                    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
                    if let Err(e) = stdin.write_all(input.as_bytes()) {
                        self.error_message = Some(format!("Failed to write input: {}", e));
                        self.clear_input();
                        return Ok(None);
                    }
                
                    // Get command result
                    let output = child.wait_with_output().expect("Failed to wait for command");
                    
                    // Handle output
                    if output.status.success() {
                        self.clear_input();
                        self.update_active = true;
                        self.error_message = Some(String::from("Success"));
                    } else {
                        let error_msg = String::from_utf8_lossy(&output.stderr);
                        self.error_message = Some(format!("Error: {}", error_msg.trim()));
                    }
                    
                    self.clear_input();
                    return Ok(None);
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

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        if !self.update_active {
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
            ])).block(Block::new().padding(Padding::horizontal(1)));
            f.render_widget(user_block, user_area);
    
            // Update the password display in draw()
            let old_display = if self.show_password {
                &self.old_input
            } else {
                &"*".repeat(self.old_input.len())
            };

            let new_display = if self.show_password {
                &self.new_input
            } else {
                &"*".repeat(self.new_input.len())
            };
            match self.focus {
                Focus::OldPassword => Block::bordered().render(old_area, f.buffer_mut()),
                _ => Block::default().render(old_area, f.buffer_mut())
            }
            let old_area = Block::bordered().inner(old_area);
            let [text_area, input_area] = Layout::horizontal([
                Constraint::Length(14),
                Constraint::Min(1),
            ]).areas(old_area);
            Paragraph::new(Line::raw("Old Password")
                .style(Style::new().bold()))
                .render(text_area, f.buffer_mut());
            // Old password input
            let old_input = Paragraph::new(&**old_display)
                .style(if self.editing && self.focus == Focus::OldPassword {
                    Style::default().bg(SLATE.c200).fg(Color::Green)
                } else {
                    Style::default().bg(SLATE.c300)
                });
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
            let new_input = Paragraph::new(&**new_display)
                .style(if self.editing && self.focus == Focus::NewPassword {
                    Style::default().bg(SLATE.c200).fg(Color::Green)
                } else {
                    Style::default().bg(SLATE.c300)
                }); 
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
        } else {
            let area = Block::default().padding(Padding::horizontal(2)).inner(area);
            let [_, main_area, _] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Fill(2),
                    Constraint::Max(60),
                    Constraint::Fill(2),
                ]).areas(area);
            let [_, main_area, _] = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Fill(2),
                    Constraint::Length(8),
                    Constraint::Fill(2),
                ]).areas(main_area);

            let main_block = Block::bordered().border_type(BorderType::Double);
            f.render_widget(&main_block, main_area);
            let main_area = main_block.inner(main_area);
            
            let [con_area, but_area] = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(3),
                ]).areas(main_area);
            let content = Paragraph::new(Line::raw("Success")
                .centered())
                .block(Block::default().padding(Padding::vertical(1)));
            let button = ButtonWidget::new("Back to Menu").state(ButtonState::Selected);
            // f.render_widget(content, con_area);
            f.render_widget(button, but_area);

            // Render error message
            if let Some(err) = &self.error_message {
                let error = Paragraph::new(err.clone())
                    .style(Style::default().fg(Color::Red));
                f.render_widget(error, con_area);
            }
        }

        Ok(())
    }
}