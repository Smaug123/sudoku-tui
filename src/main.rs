use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    crossterm::{self, event::KeyCode},
    layout::{Constraint, Direction, Layout, Rect},
    prelude::{Alignment, CrosstermBackend},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use sorted_vec::SortedVec;
use std::{
    fs::File,
    io::{self, BufReader, Read},
    time::Duration,
};

#[derive(Clone, Copy, PartialEq)]
enum InputMode {
    Normal,
    Corner,
    Centre,
}

#[derive(Default, Clone)]
struct Cell {
    main_number: Option<u8>,
    corner_numbers: SortedVec<u8>,
    centre_numbers: SortedVec<u8>,
    is_fixed: bool,
}

struct App {
    grid: Vec<Vec<Cell>>,
    cursor_x: usize,
    cursor_y: usize,
    input_mode: InputMode,
}

impl App {
    fn new() -> Self {
        Self {
            grid: vec![vec![Cell::default(); 9]; 9],
            cursor_x: 0,
            cursor_y: 0,
            input_mode: InputMode::Normal,
        }
    }
    fn load_from_string(&mut self, content: &str) -> io::Result<()> {
        for (y, line) in content.lines().enumerate() {
            if y >= 9 {
                break;
            }
            for (x, ch) in line.chars().enumerate() {
                if x >= 9 {
                    break;
                }
                if let Some(num) = ch.to_digit(10) {
                    if num > 0 {
                        self.grid[y][x].main_number = Some(num as u8);
                        self.grid[y][x].is_fixed = true;
                    }
                }
            }
        }
        Ok(())
    }

    fn load_from_file(&mut self, filename: &str) -> io::Result<()> {
        let file = File::open(filename)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        self.load_from_string(&content)
    }

    fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Left => self.cursor_x = self.cursor_x.saturating_sub(1),
            KeyCode::Right => self.cursor_x = (self.cursor_x + 1).min(8),
            KeyCode::Up => self.cursor_y = self.cursor_y.saturating_sub(1),
            KeyCode::Down => self.cursor_y = (self.cursor_y + 1).min(8),
            KeyCode::Char('/') => self.input_mode = InputMode::Normal,
            KeyCode::Char(',') => self.input_mode = InputMode::Corner,
            KeyCode::Char('.') => self.input_mode = InputMode::Centre,
            KeyCode::Char(ch) => {
                if let Some(num) = ch.to_digit(10) {
                    if num == 0 {
                        return;
                    }
                    let num = num as u8;
                    let cell = &mut self.grid[self.cursor_y][self.cursor_x];
                    if cell.is_fixed {
                        return;
                    }
                    match self.input_mode {
                        InputMode::Normal => {
                            cell.corner_numbers.clear();
                            cell.centre_numbers.clear();
                            match cell.main_number {
                                None => {
                                    cell.main_number = Some(num);
                                }
                                Some(existing) => {
                                    if existing == num {
                                        cell.main_number = None;
                                    } else {
                                        cell.main_number = Some(num)
                                    }
                                }
                            }
                        }
                        InputMode::Corner => match cell.main_number {
                            Some(_) => {}
                            None => {
                                if cell.corner_numbers.remove_item(&num).is_none() {
                                    cell.corner_numbers.push(num);
                                }
                            }
                        },
                        InputMode::Centre => match cell.main_number {
                            Some(_) => {}
                            None => {
                                if cell.centre_numbers.remove_item(&num).is_none() {
                                    cell.centre_numbers.push(num);
                                }
                            }
                        },
                    }
                }
            }
            _ => {}
        }
    }
}

fn draw_cell(
    f: &mut Frame,
    cell: &Cell,
    area: Rect,
    is_selected: bool,
    is_right_thick: bool,
    is_bottom_thick: bool,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(if is_selected {
            Color::DarkGray
        } else {
            Color::Black
        }));

    f.render_widget(block.clone(), area);

    // Draw main number if it exists
    if let Some(num) = cell.main_number {
        let style = if cell.is_fixed {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        let text = vec![Line::from(vec![Span::styled(num.to_string(), style)])];

        let p = Paragraph::new(text).block(Block::default());
        let centre_area = centred_rect(1, 1, area);
        f.render_widget(p, centre_area);
    } else {
        // Corner number positions (clockwise from top-left)
        let corner_positions = [
            (area.x + 1, area.y + 1),                            // Top-left
            (area.x + area.width - 2, area.y + 1),               // Top-right
            (area.x + area.width - 2, area.y + area.height - 2), // Bottom-right
            (area.x + 1, area.y + area.height - 2),              // Bottom-left
        ];

        // Draw corner numbers
        for (idx, num) in cell.corner_numbers.iter().enumerate() {
            if idx >= 4 {
                break;
            }
            let (x, y) = corner_positions[idx];
            let text = vec![Line::from(vec![Span::styled(
                num.to_string(),
                Style::default().fg(Color::Yellow),
            )])];
            let p = Paragraph::new(text).block(Block::default());
            f.render_widget(p, Rect::new(x, y, 1, 1));
        }

        // Draw centre numbers
        if !cell.centre_numbers.is_empty() {
            let centre_text = cell
                .centre_numbers
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join("");

            let text = vec![Line::from(vec![Span::styled(
                centre_text,
                Style::default().fg(Color::Blue),
            )])];

            let p = Paragraph::new(text).block(Block::default());
            let centre_area = centred_rect(cell.centre_numbers.len() as u16, 1, area);
            f.render_widget(p, centre_area);
        }
    }

    // Draw thick borders manually
    if is_right_thick {
        let right_border = Block::default()
            .style(Style::default().fg(Color::White))
            .borders(Borders::LEFT);
        f.render_widget(
            right_border,
            Rect::new(area.x + area.width, area.y, 1, area.height),
        );
    }
    if is_bottom_thick {
        let bottom_border = Block::default()
            .style(Style::default().fg(Color::White))
            .borders(Borders::TOP);
        f.render_widget(
            bottom_border,
            Rect::new(area.x, area.y + area.height, area.width, 1),
        );
    }
}

const HELP_LINES: [&[(&str, Option<Color>)]; 12] = [
    &[
        ("Movement: ", Some(Color::White)),
        ("↑ ↓ ← →", Some(Color::Yellow)),
        (" arrow keys to navigate the grid", None),
    ],
    &[("Modes: ", Some(Color::White))],
    &[
        ("/", Some(Color::Yellow)),
        (" - Normal mode (enter numbers directly)", None),
    ],
    &[
        (",", Some(Color::Yellow)),
        (" - Corner mode (small numbers in corners)", None),
    ],
    &[
        (".", Some(Color::Yellow)),
        (" - Centre mode (small numbers in centre)", None),
    ],
    &[
        ("Numbers: ", Some(Color::White)),
        ("Use keys 1-9 to enter values", None),
    ],
    &[("Color coding:", Some(Color::White))],
    &[
        ("Green", Some(Color::Green)),
        (" - Fixed numbers (unchangeable)", None),
    ],
    &[
        ("White", Some(Color::White)),
        (" - User-entered numbers", None),
    ],
    &[
        ("Yellow", Some(Color::Yellow)),
        (" - Corner numbers (up to 4)", None),
    ],
    &[
        ("Blue", Some(Color::Blue)),
        (" - Centre numbers (up to 3)", None),
    ],
    &[
        ("Exit: ", Some(Color::White)),
        ("q", Some(Color::Yellow)),
        (" to quit the application", None),
    ],
];

fn draw_help_text(f: &mut Frame, area: Rect) {
    let help_text = HELP_LINES
        .iter()
        .map(|line| {
            let spans = line
                .iter()
                .map(|(text, color)| {
                    let style = match color {
                        Some(color) => Style::default().fg(*color),
                        None => Style::default(),
                    };
                    Span::styled(*text, style)
                })
                .collect::<Vec<_>>();
            Line::from(spans)
        })
        .collect::<Vec<_>>();

    let help_paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(Style::default());

    f.render_widget(help_paragraph, area);
}

fn centred_rect(width: u16, height: u16, parent: Rect) -> Rect {
    let x = parent.x + (parent.width.saturating_sub(width)) / 2;
    let y = parent.y + (parent.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(parent.width), height.min(parent.height))
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.area();

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((9 * 5 + 2) as u16), // Grid height + dividers
            Constraint::Length(1),                  // Mode indicator
            Constraint::Min(HELP_LINES.len() as u16), // Help text
        ])
        .split(size);

    // Create a layout for the 9x9 grid with dividers
    let cell_width = 8;
    let cell_height = 5;
    let divider_width = 1;
    let divider_height = 1;
    let total_width = (cell_width * 9) + (divider_width * 2); // Add space for 2 vertical dividers
    let total_height = (cell_height * 9) + (divider_height * 2); // Add space for 2 horizontal dividers

    let grid_area = Rect::new(
        (size.width.saturating_sub(total_width)) / 2,
        main_layout[0].y,
        total_width,
        total_height,
    );

    // Draw horizontal dividers
    for div_y in 1..=2 {
        let y_pos = grid_area.y + (div_y * 3 * cell_height);
        for x in 0..total_width {
            let x_pos = grid_area.x + x;
            f.render_widget(
                Block::default()
                    .style(Style::default().fg(Color::White))
                    .borders(Borders::TOP),
                Rect::new(x_pos, y_pos, 1, 1),
            );
        }
    }

    // Draw vertical dividers
    for div_x in 1..=2 {
        let x_pos = grid_area.x + (div_x * 3 * cell_width);
        for y in 0..total_height {
            let y_pos = grid_area.y + y;
            f.render_widget(
                Block::default()
                    .style(Style::default().fg(Color::White))
                    .borders(Borders::LEFT),
                Rect::new(x_pos, y_pos, 1, 1),
            );
        }
    }

    // Draw each cell
    for y in 0..9 {
        for x in 0..9 {
            // Calculate position accounting for dividers
            let extra_x_dividers = (x / 3) as u16;
            let extra_y_dividers = (y / 3) as u16;

            let cell_area = Rect::new(
                grid_area.x + (x as u16 * cell_width) + extra_x_dividers,
                grid_area.y + (y as u16 * cell_height) + extra_y_dividers,
                cell_width,
                cell_height,
            );

            let is_right_thick = x % 3 == 2;
            let is_bottom_thick = y % 3 == 2;

            draw_cell(
                f,
                &app.grid[y][x],
                cell_area,
                x == app.cursor_x && y == app.cursor_y,
                is_right_thick,
                is_bottom_thick,
            );
        }
    }

    // Draw mode indicator
    let mode_text = match app.input_mode {
        InputMode::Normal => "Mode: Normal (/)",
        InputMode::Corner => "Mode: Corner (,)",
        InputMode::Centre => "Mode: Centre (.)",
    };

    let mode_paragraph = Paragraph::new(Line::from(vec![Span::styled(
        mode_text,
        Style::default().fg(Color::White),
    )]))
    .alignment(Alignment::Center);

    f.render_widget(mode_paragraph, main_layout[1]);

    // Draw help text
    let help_area = Rect::new(
        grid_area.x,
        main_layout[2].y,
        total_width,
        HELP_LINES.len() as u16,
    );

    draw_help_text(f, help_area);
}

fn main() -> io::Result<()> {
    // Initialize terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    // Create app state
    let mut app = App::new();
    app.load_from_file("sudoku.txt")?;

    // Main loop
    let result = loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break Ok(()),
                        key_code => app.handle_input(key_code),
                    }
                }
            }
        }
    };

    // Cleanup
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};

    fn setup_test_terminal() -> Terminal<TestBackend> {
        let backend = TestBackend::new(80, 120);
        Terminal::new(backend).unwrap()
    }

    #[test]
    fn test_render_1() {
        let mut app = App::new();
        app.input_mode = InputMode::Normal;
        // Read content from sudoku.txt
        let content = include_str!("../sudoku.txt");
        app.load_from_string(content).unwrap();

        let mut terminal = setup_test_terminal();

        terminal
            .draw(|f| {
                ui(f, &app); // Call the ui function to render the app
            })
            .unwrap();

        let buffer = terminal.backend();

        // You might want to format the buffer in a specific way
        let rendered = buffer
            .buffer()
            .content
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let c = cell.symbol().to_string();
                if i % 80 == 79 {
                    format!("{}\n", c)
                } else {
                    c
                }
            })
            .collect::<String>();

        assert_snapshot!(rendered);
    }
}
