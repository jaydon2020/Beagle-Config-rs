use crossterm::event::KeyCode;
use ratatui::{
    layout::*, prelude::{Style, Stylize}, style::{palette::tailwind, *}, text::*, widgets::*, Frame
};
use color_eyre::Result;

use crate::action::Action;

use super::ViewComponent;

#[derive(Debug, Clone, Copy, PartialEq)]
enum SelectedTable {
    Left,
    Right,
}

pub struct PinOut {
    title: String,
    state_l: TableState,
    state_r: TableState,
    items: Vec<PinInfo>,
    selected_table: SelectedTable, // Add this
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PinType {
    Power3v3,
    Power5v,
    Ground,
    GPIO,
    SPI,
    I2C,
    UART,
    PCM,
    Special,
}

impl PinOut {
    pub fn init() -> Self {
        PinOut {
            title: String::from("pinout"),
            state_l: TableState::default().with_selected(0),
            state_r: TableState::default().with_selected(None),
            items: load_pin_data(),
            selected_table: SelectedTable::Left, // Initial selection
        }
    }

    // Update navigation methods to handle both tables
    pub fn next_row(&mut self) {
        let state = match self.selected_table {
            SelectedTable::Left => &self.state_l,
            SelectedTable::Right => &self.state_r,
        };

        let i = match state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        if self.selected_table == SelectedTable::Left {
            self.state_l.select(Some(i));
        } else {
            self.state_r.select(Some(i));
        }
        // self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous_row(&mut self) {
        let state = match self.selected_table {
            SelectedTable::Left => &self.state_l,
            SelectedTable::Right => &self.state_r,
        };

        let i = match state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        
        if self.selected_table == SelectedTable::Left {
            self.state_l.select(Some(i));
        } else {
            self.state_r.select(Some(i));
        }
        // self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    // Helper methods to get filtered items
    fn switch_left(&mut self) {
        if self.selected_table == SelectedTable::Right {
            let _ = match self.state_r.selected() {
                Some(i) => {
                    self.state_l.select(Some(i));
                }
                None => self.state_l.select(None),
            };

            self.state_r.select(None);
            self.selected_table = SelectedTable::Left;
        }
    }

    fn switch_right(&mut self) {
        if self.selected_table == SelectedTable::Left {
            let i = match self.state_l.selected() {
                Some(i) => {
                    self.state_r.select(Some(i));
                }
                None => self.state_r.select(None),
            };

            self.state_l.select(None);
            self.selected_table = SelectedTable::Right;
        }
    }

    // Helper methods to get filtered items
    fn left_items(&self) -> Vec<&PinInfo> {
        self.items.iter()
            .filter(|data| data.number % 2 != 0)
            .collect()
    }

    fn right_items(&self) -> Vec<&PinInfo> {
        self.items.iter()
            .filter(|data| data.number % 2 == 0)
            .collect()
    }
    fn render_info(&self, frame: &mut Frame, area: Rect, pin: &PinInfo) -> Result<()> {
        let info = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Pin Number: ", Style::new().fg(tailwind::SLATE.c300)),
                Span::styled(pin.number.to_string(), Style::new().fg(tailwind::SLATE.c100)),
            ]),
            Line::from(vec![
                Span::styled("Name: ", Style::new().fg(tailwind::SLATE.c300)),
                Span::styled(&pin.name, Style::new().fg(pin.type_color())),
            ]),
            Line::from(vec![
                Span::styled("Function: ", Style::new().fg(tailwind::SLATE.c300)),
                Span::styled(&pin.function, Style::new().fg(tailwind::SLATE.c100)),
            ]),
            Line::from(vec![
                Span::styled("Type: ", Style::new().fg(tailwind::SLATE.c300)),
                Span::styled(
                    format!("{:?}", pin.pin_type),
                    Style::new().fg(pin.type_color()).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(
                pin.note.as_ref().map_or_else(|| Line::default(), |note| Line::from(vec![
                    Span::styled("Note: ", Style::new().fg(tailwind::SLATE.c300)),
                    Span::styled(note, Style::new().fg(tailwind::SLATE.c100).italic()),
                ]))
            ),
        ])
        .block(
            Block::default()
                .title(" Pin Details ")
                .borders(Borders::ALL)
                .border_style(Style::new().fg(pin.type_color()))
                .padding(Padding::uniform(1)),
        );

        frame.render_widget(info, area);
        Ok(())
    }
}

impl ViewComponent for PinOut {
    fn title(&self) -> &str {
        &self.title
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Backspace => { return Ok(Some(Action::BackToMenu)); },
            KeyCode::Char('j') | KeyCode::Down => self.next_row(),
            KeyCode::Char('k') | KeyCode::Up => self.previous_row(),
            KeyCode::Char('h') | KeyCode::Left => self.switch_left(),
            KeyCode::Char('l') | KeyCode::Right => self.switch_right(),
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let area = area.inner(Margin { horizontal: 1, vertical: 0 });

        let [pin_area, info_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(38),
            Constraint::Min(1),
        ]).areas(area);
        
        let [pin_area, lengend_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(22),
            Constraint::Min(1),
        ]).areas(pin_area);

        let pin_block = Block::bordered().title("Pin");
        f.render_widget(&pin_block, pin_area);
        // let lengen_block = Block::bordered().title("Legend");
        // f.render_widget(&lengen_block, lengend_area);
        let info_block = Block::bordered().title("Info");
        f.render_widget(&info_block, info_area);

        let [left_area, right_area, _] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(18),
                Constraint::Length(18),
                Constraint::Min(1),
            ]).areas(pin_block.inner(pin_area));
        
        self.draw_table_left(f, left_area)?;
        self.draw_table_right(f, right_area)?;
        self.render_legend(f, lengend_area)?;

        let binding_left = self.left_items();
        let binding_right = self.right_items();
        let selected_pin = match self.selected_table {
            SelectedTable::Left => binding_left.get(self.state_l.selected().unwrap_or(0)),
            SelectedTable::Right => binding_right.get(self.state_r.selected().unwrap_or(0)),
        };
    
        if let Some(pin) = selected_pin {
            self.render_info(f, info_area, pin)?;
        }

        Ok(())
    }
}

impl PinOut {
    fn draw_table_left(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Filter and sort odd-numbered pins
        let mut odd_pins: Vec<_> = self.items.iter()
            .filter(|data| data.number % 2 != 0)
            .collect();
        odd_pins.sort_by_key(|d| d.number);
    
        let rows = odd_pins.into_iter().map(|data| {
            let item = data.ref_info();
            Row::new(vec![
                Cell::from(Text::from(format!(" {}", item.1))),
                Cell::from(Text::from(format!("{}", item.0)).alignment(Alignment::Right)),
                Cell::from(Text::from("◉")
                    .bg(tailwind::SLATE.c900)
                    .fg(item.2)
                    .centered()),
            ])
        });
    
        let t = Table::new(
            rows,
            [
                Constraint::Min(11),
                Constraint::Length(2),
                Constraint::Length(3),
            ],
        )
        .bg(tailwind::CYAN.c400)
        .row_highlight_style(Style::new().bg(palette::tailwind::AMBER.c100));
    
        f.render_stateful_widget(t, area, &mut self.state_l);
        Ok(())
    }
    
    fn draw_table_right(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Filter and sort even-numbered pins
        let mut even_pins: Vec<_> = self.items.iter()
            .filter(|data| data.number % 2 == 0)
            .collect();
        even_pins.sort_by_key(|d| d.number);
    
        let rows = even_pins.into_iter().map(|data| {
            let item = data.ref_info();
            Row::new(vec![
                Cell::from(Text::from("◉")
                    .bg(tailwind::SLATE.c900)
                    .fg(item.2)
                    .centered()),
                Cell::from(Text::from(format!("{}", item.0))),
                Cell::from(Text::from(format!("{}", item.1))),
            ])
        });
    
        let t = Table::new(
            rows,
            [
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Min(11),
            ],
        )
        .bg(tailwind::CYAN.c400)
        .row_highlight_style(Style::new().bg(palette::tailwind::AMBER.c100));
    
        f.render_stateful_widget(t, area, &mut self.state_r);
        Ok(())
    }

    fn render_legend(&mut self, frame: &mut Frame, area: Rect) -> Result<()>{
        let legend = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("◉ GPIO ", Style::new().fg(tailwind::LIME.c500)),
                Span::raw("(General Purpose IO)"),
            ]),
            Line::from(vec![
                Span::styled("◉ SPI", Style::new().fg(tailwind::PINK.c500)),
                Span::raw("(Serial Peripheral Interface)"),
            ]),
            Line::from(vec![
                Span::styled("◉ I2C", Style::new().fg(tailwind::SKY.c500)),
                Span::raw("(Inter-integrated Circuit)"),
            ]),
            Line::from(vec![
                Span::styled("◉ UART", Style::new().fg(tailwind::VIOLET.c500)),
                Span::raw("(Universal Asynchronous Receiver/Transmitter)"),
            ]),
            Line::from(vec![
                Span::styled("◉ PCM", Style::new().fg(tailwind::TEAL.c500)),
                Span::raw("(Pulse Code Modulation)"),
            ]),
            Line::from(vec![
                Span::styled("◉ Ground", Style::new().fg(tailwind::WHITE)),
                Span::raw(""),
            ]),
            Line::from(vec![
                Span::styled("◉ 5v", Style::new().fg(tailwind::RED.c500)),
                Span::raw("(Power)"),
            ]),
            Line::from(vec![
                Span::styled("◉ 3.3v", Style::new().fg(tailwind::AMBER.c500)),
                Span::raw("((Power)"),
            ]),
        ])
        .block(Block::default().title(" Legend ").borders(Borders::ALL).padding(Padding::horizontal(2)))
        .alignment(Alignment::Left);
    
        frame.render_widget(legend, area);
        Ok(())
    }
}

// Sample data implementation
fn load_pin_data() -> Vec<PinInfo> {
    vec![
        // Bottom row (left side)
        PinInfo {
            number: 1,
            name: "3v3 Power".to_string(),
            function: "Buck 1 (3.5A max)".to_string(),
            pin_type: PinType::Power3v3,
            note: Some("Shared rail - 500mA usable".to_string()),
        },
        PinInfo {
            number: 3,
            name: "GPIO 2".to_string(),
            function: "I2C1 SDA".to_string(),
            pin_type: PinType::I2C,
            note: Some("SoC pin E11".to_string()),
        },
        PinInfo {
            number: 5,
            name: "GPIO 3".to_string(),
            function: "I2C1 SCL".to_string(),
            pin_type: PinType::I2C,
            note: Some("SoC pin B13".to_string()),
        },
        PinInfo {
            number: 7,
            name: "GPIO 4".to_string(),
            function: String::new(),
            pin_type: PinType::GPIO,
            note: Some("SoC pin W26".to_string()),
        },
        PinInfo {
            number: 9,
            name: "Ground".to_string(),
            function: String::new(),
            pin_type: PinType::Ground,
            note: None,
        },
        PinInfo {
            number: 11,
            name: "GPIO 17".to_string(),
            function: String::new(),
            pin_type: PinType::GPIO,
            note: Some("SoC pin A26".to_string()),
        },
        PinInfo {
            number: 13,
            name: "GPIO 27".to_string(),
            function: String::new(),
            pin_type: PinType::GPIO,
            note: Some("SoC pin N22".to_string()),
        },
        PinInfo {
            number: 15,
            name: "GPIO 22".to_string(),
            function: String::new(),
            pin_type: PinType::GPIO,
            note: Some("SoC pin R27".to_string()),
        },
        PinInfo {
            number: 17,
            name: "3v3 Power".to_string(),
            function: String::new(),
            pin_type: PinType::Power3v3,
            note: None,
        },
        PinInfo {
            number: 19,
            name: "GPIO 10".to_string(),
            function: "SPI0 MOSI".to_string(),
            pin_type: PinType::SPI,
            note: Some("SoC pin B12".to_string()),
        },
        PinInfo {
            number: 21,
            name: "GPIO 9".to_string(),
            function: "SPI0 MISO".to_string(),
            pin_type: PinType::SPI,
            note: Some("SoC pin C11".to_string()),
        },
        PinInfo {
            number: 23,
            name: "GPIO 11".to_string(),
            function: "SPI0 SCLK".to_string(),
            pin_type: PinType::SPI,
            note: Some("SoC pin A9".to_string()),
        },
        PinInfo {
            number: 25,
            name: "Ground".to_string(),
            function: String::new(),
            pin_type: PinType::Ground,
            note: None,
        },
        PinInfo {
            number: 27,
            name: "GPIO 0".to_string(),
            function: "EEPROM SDA".to_string(),
            pin_type: PinType::I2C,
            note: Some("SoC pin D11".to_string()),
        },
        PinInfo {
            number: 29,
            name: "GPIO 5".to_string(),
            function: String::new(),
            pin_type: PinType::GPIO,
            note: Some("SoC pin B20".to_string()),
        },
        PinInfo {
            number: 31,
            name: "GPIO 6".to_string(),
            function: String::new(),
            pin_type: PinType::GPIO,
            note: Some("SoC pin D20".to_string()),
        },
        PinInfo {
            number: 33,
            name: "GPIO 13".to_string(),
            function: "PWM1".to_string(),
            pin_type: PinType::GPIO,
            note: Some("SoC pin E19".to_string()),
        },
        PinInfo {
            number: 35,
            name: "GPIO 19".to_string(),
            function: "PCM FS".to_string(),
            pin_type: PinType::PCM,
            note: Some("SoC pin C26".to_string()),
        },
        PinInfo {
            number: 37,
            name: "GPIO 26".to_string(),
            function: String::new(),
            pin_type: PinType::GPIO,
            note: Some("SoC pin P26".to_string()),
        },
        PinInfo {
            number: 39,
            name: "Ground".to_string(),
            function: String::new(),
            pin_type: PinType::Ground,
            note: None,
        },
        // Top row (right side)
        PinInfo {
            number: 2,
            name: "5v Power".to_string(),
            function: String::new(),
            pin_type: PinType::Power5v,
            note: None,
        },
        PinInfo {
            number: 4,
            name: "5v Power".to_string(),
            function: String::new(),
            pin_type: PinType::Power5v,
            note: None,
        },
        PinInfo {
            number: 6,
            name: "Ground".to_string(),
            function: String::new(),
            pin_type: PinType::Ground,
            note: None,
        },
        PinInfo {
            number: 8,
            name: "GPIO 14".to_string(),
            function: "UART TX".to_string(),
            pin_type: PinType::UART,
            note: Some("SoC pin F24".to_string()),
        },
        PinInfo {
            number: 10,
            name: "GPIO 15".to_string(),
            function: "UART RX".to_string(),
            pin_type: PinType::UART,
            note: Some("SoC pin C27".to_string()),
        },
        PinInfo {
            number: 12,
            name: "GPIO 18".to_string(),
            function: "PCM CLK".to_string(),
            pin_type: PinType::PCM,
            note: Some("SoC pin D25".to_string()),
        },
        PinInfo {
            number: 14,
            name: "Ground".to_string(),
            function: String::new(),
            pin_type: PinType::Ground,
            note: None,
        },
        PinInfo {
            number: 16,
            name: "GPIO 23".to_string(),
            function: String::new(),
            pin_type: PinType::GPIO,
            note: Some("SoC pin B5".to_string()),
        },
        PinInfo {
            number: 18,
            name: "GPIO 24".to_string(),
            function: String::new(),
            pin_type: PinType::GPIO,
            note: Some("SoC pin C8".to_string()),
        },
        PinInfo {
            number: 20,
            name: "Ground".to_string(),
            function: String::new(),
            pin_type: PinType::Ground,
            note: None,
        },
        PinInfo {
            number: 22,
            name: "GPIO 25".to_string(),
            function: String::new(),
            pin_type: PinType::GPIO,
            note: Some("SoC pin P21".to_string()),
        },
        PinInfo {
            number: 24,
            name: "GPIO 8".to_string(),
            function: "SPI0 CE0".to_string(),
            pin_type: PinType::SPI,
            note: Some("SoC pin C12".to_string()),
        },
        PinInfo {
            number: 26,
            name: "GPIO 7".to_string(),
            function: "SPI0 CE1".to_string(),
            pin_type: PinType::SPI,
            note: Some("SoC pin B3".to_string()),
        },
        PinInfo {
            number: 28,
            name: "GPIO 1".to_string(),
            function: "EEPROM SCL".to_string(),
            pin_type: PinType::I2C,
            note: Some("SoC pin B9".to_string()),
        },
        PinInfo {
            number: 30,
            name: "Ground".to_string(),
            function: String::new(),
            pin_type: PinType::Ground,
            note: None,
        },
        PinInfo {
            number: 32,
            name: "GPIO 12".to_string(),
            function: "PWM0".to_string(),
            pin_type: PinType::GPIO,
            note: Some("SoC pin C20".to_string()),
        },
        PinInfo {
            number: 34,
            name: "Ground".to_string(),
            function: String::new(),
            pin_type: PinType::Ground,
            note: None,
        },
        PinInfo {
            number: 36,
            name: "GPIO 16".to_string(),
            function: String::new(),
            pin_type: PinType::GPIO,
            note: Some("SoC pin A25".to_string()),
        },
        PinInfo {
            number: 38,
            name: "GPIO 20".to_string(),
            function: "PCM DIN".to_string(),
            pin_type: PinType::PCM,
            note: Some("SoC pin F23".to_string()),
        },
        PinInfo {
            number: 40,
            name: "GPIO 21".to_string(),
            function: "PCM DOUT".to_string(),
            pin_type: PinType::PCM,
            note: Some("SoC pin B25".to_string()),
        },
    ]
}

struct PinInfo {
    number: u16,
    name: String,
    function: String,
    pin_type: PinType,
    note: Option<String>,
}

impl PinInfo {
    fn empty() -> Self {
        Self {
            number: 0,
            name: String::new(),
            function: String::new(),
            pin_type: PinType::Ground,
            note: None,
        }
    }

    fn ref_info(&self) -> (u16, &str, Color) {
        (self.number, &self.name, self.type_color())
    }

    const fn type_color(&self) -> Color {
        match self.pin_type {
            PinType::Power3v3 => tailwind::AMBER.c500,
            PinType::Power5v => tailwind::RED.c500,
            PinType::Ground => tailwind::WHITE,
            PinType::GPIO => tailwind::LIME.c500,
            PinType::SPI => tailwind::PINK.c500,
            PinType::I2C => tailwind::SKY.c500,
            PinType::UART => tailwind::VIOLET.c500,
            PinType::PCM => tailwind::TEAL.c500,
            PinType::Special => tailwind::SKY.c500,
        }
    }
}