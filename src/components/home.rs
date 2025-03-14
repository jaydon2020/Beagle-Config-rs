use color_eyre::{eyre::Ok, Result};
use crossterm::event::KeyEventKind;
use ratatui::{prelude::*, style::palette::tailwind::SLATE, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::{views::{password::PasswordView, ssh::SshView, test::TestViewComponent, ViewComponent, locale::LocaleView}, Component};
use crate::{action::Action, config::Config, widgets::{ButtonState, TextButtonWidget}};

// #[derive(Default)]
pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    menu_list: Vec<MenuGroup>,
    menu_state: ListState,
    button_state: ButtonState,
    active: bool,
}

struct MenuGroup {
    name: String,
    component: Vec<Box<dyn ViewComponent>>,
    state: ListState,
}

impl Home {
    pub fn new() -> Self {
        Self {
            command_tx: None, 
            config: Config::default(),
            menu_list: vec![
                MenuGroup {
                    name: String::from("System"),
                    component: vec![
                        Box::new(PasswordView::init()),
                        Box::new(SshView::init()),
                        Box::new(LocaleView::init()),
                    ],
                    state: ListState::default(),
                },
                MenuGroup {
                    name: String::from("Audio"),
                    component: vec![
                        Box::new(TestViewComponent::new("Item4")),
                        Box::new(TestViewComponent::new("Item5")),
                        Box::new(TestViewComponent::new("Item6")),
                    ],
                    state: ListState::default(),
                },
                MenuGroup {
                    name: String::from("Network"),
                    component: vec![
                        Box::new(TestViewComponent::new("Item7")),
                        Box::new(TestViewComponent::new("Item8")),
                        Box::new(TestViewComponent::new("Item9")),
                    ],
                    state: ListState::default(),
                },
                MenuGroup {
                    name: String::from("Experiment"),
                    component: vec![
                        Box::new(TestViewComponent::new("Item10")),
                        Box::new(TestViewComponent::new("Item11")),
                        Box::new(TestViewComponent::new("Item12")),
                    ],
                    state: ListState::default(),
                }
            ],
            menu_state: ListState::default(),
            button_state: ButtonState::Selected,
            active: false,
        }
    }

    fn reset_select(&mut self) {
        self.button_state = ButtonState::Selected;
        self.menu_state.select(None);
        for i in self.menu_list.iter_mut() {
            i.state.select(None);
        }
    }

    fn select_previous(&mut self) {
        match self.menu_state.selected() {
            Some(idx) => {
                let current_menu = &mut self.menu_list[idx];

                if current_menu.state.selected().unwrap_or(0) > 0 {
                    current_menu.state.select_previous();
                    return;
                }

                if idx > 0 {
                    current_menu.state.select(None);
                    self.menu_state.select_previous();
                    let prev_idx = idx -1;
                    self.menu_list[prev_idx].state.select_last();
                }
                else {
                    self.reset_select();
                }
            }
            None => {

            }
        }
    }

    fn select_next(&mut self) {
        match self.menu_state.selected() {
            Some(current_idx) => {
                let menu_len = self.menu_list.len();
                let current_menu = &mut self.menu_list[current_idx];
                let item_count = current_menu.component.len();
                
                // Try to move within current menu
                if current_menu.state.selected().unwrap_or(0) < item_count - 1 {
                    current_menu.state.select_next();
                    return;
                }
                
                // Move to next menu
                if current_idx < menu_len - 1 {
                    current_menu.state.select(None);
                    self.menu_state.select_next();
                    let next_idx = current_idx + 1;
                    self.menu_list[next_idx].state.select_first();
                } else {

                }
            }
            None => {
                // Start from first menu if nothing selected
                self.menu_state.select_first();
                self.menu_list[0].state.select_first();
                self.button_state = ButtonState::Normal;
            }
        }
    }
}

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    // "-",
    // env!("VERGEN_GIT_DESCRIBE"),
    // " (",
    // env!("VERGEN_BUILD_DATE"),
    // ")"
);

impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                // add any logic here that should run on every tick
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            Action::BackToMenu => {
                self.active = false;
                // return Ok(Some(Action::Quit));
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }
        if self.active {
            // if key.code == crossterm::event::KeyCode::Backspace { self.active = false };
            if let Some(selected_group) = self.menu_state.selected() {
                if let Some(selected_item) = self.menu_list[selected_group].state.selected() {
                    return self.menu_list[selected_group].component[selected_item]
                        .handle_key_events(key);
                }
            }
        }
        match key.code {
            crossterm::event::KeyCode::Up => self.select_previous(),
            crossterm::event::KeyCode::Down => self.select_next(),
            crossterm::event::KeyCode::Enter => {
                // Handle menu activation first
                if let Some(selected_group) = self.menu_state.selected() {
                    let group = &mut self.menu_list[selected_group];
                    if group.state.selected().is_some() {
                        self.active = true;
                        return Ok(None);
                    }
                }
                
                // Then handle button quit action
                if self.button_state == ButtonState::Selected {
                    return Ok(Some(Action::Quit));
                }
            },
            crossterm::event::KeyCode::Esc => self.button_state = ButtonState::Normal,
            _ => {}
        };

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let [header_area, main_area, footer_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Fill(0),
                Constraint::Length(2)
            ]).areas(area);

        let header = Paragraph::new(
            Line::from_iter([
                "Beagle-Config ".into(),
                VERSION_MESSAGE,
            ]).centered());
        frame.render_widget(&header, header_area);

        let [menu_area, config_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Fill(5),
            ]).areas(main_area);
        
        let menu_block = if !self.active {
            Block::bordered()
                .border_type(BorderType::Double)
        } else {
            Block::bordered()
                .border_type(BorderType::Rounded)
                .dim()
        };
        let config_block = if self.active {
            Block::bordered()
                .border_type(BorderType::Double)
        } else {
            Block::bordered()
                .border_type(BorderType::Rounded)
                .dim()
        };
        frame.render_widget(&menu_block, menu_area);
        frame.render_widget(&config_block, config_area);

        let instruction = 
            Paragraph::new("Use ↓↑ to move, BackSpace to unselect, Enter to change status, g/G to go top/bottom.")
                .centered();
        frame.render_widget(&instruction, footer_area);

        let menu_inner = menu_block.inner(menu_area);
        let config_inner = menu_block.inner(config_area);

        let mut constraints = Vec::new();
        constraints.push(Constraint::Length(1));
        for menu in self.menu_list.iter() {
            constraints.push(Constraint::Length(menu.component.len() as u16 + 1));
        }
        constraints.push(Constraint::Length(1));
        let chunks = Layout::default().direction(Direction::Vertical).constraints(constraints).split(menu_inner);
        
        let [m_h_area, m_b_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(0),
                Constraint::Length(7),
            ]).areas(chunks[0]);
        let menu_header = Paragraph::new(" Menu");
        // .block(Block::default().borders(Borders::BOTTOM));
        frame.render_widget(&menu_header, m_h_area);

        let exit_button = TextButtonWidget::new("Exit").state(self.button_state);
        frame.render_widget(exit_button, m_b_area);

        for (i, menu) in self.menu_list.iter_mut().enumerate() {
            let items: Vec<String> = menu.component
                .iter()
                .map(|f| format!(" {}", f.title()))
                .collect();
            let menu_list = List::default().items(items)
                .block(Block::default().borders(Borders::TOP).title(format!("─{}", menu.name)))
                .highlight_style(Style::new().bg(SLATE.c500).add_modifier(Modifier::BOLD))
                .highlight_symbol(">")
                .highlight_spacing(HighlightSpacing::Always);
            frame.render_stateful_widget(&menu_list, chunks[i + 1], &mut menu.state);
        }
        frame.render_widget(Block::default().borders(Borders::BOTTOM), *chunks.last().unwrap());
        
        let [c_header_area, c_main_area] = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2),
                    Constraint::Fill(0)
                ]).areas(config_inner);
            
        if let Some(selected) = self.menu_state.selected() {
            let state = self.menu_list[selected].state.clone();
            let component = &mut self.menu_list[selected].component[state.selected().unwrap()];
            Paragraph::new(Line::from_iter([" ", component.title()]))
                .block(Block::new().borders(Borders::BOTTOM))
                .render(c_header_area, frame.buffer_mut());
            component.draw(frame, c_main_area)?;
        }
        
        Ok(())  
    }
}
