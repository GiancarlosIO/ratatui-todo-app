use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{f32::consts::E, io};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

enum InputMode {
    Normal,
    Editing,
    Searching,
}
impl Default for InputMode {
    fn default() -> Self {
        InputMode::Normal // this will be our default mode
    }
}

#[derive(Default)]
struct App {
    input_mode: InputMode,
    search_input: String,
    todos: Vec<String>, // we'll make this more sophisticated later
    filtered_todos: Vec<String>,
    selected_index: Option<usize>,
}

impl App {
    fn new() -> Self {
        let todos = vec![
            "Learn Rust".to_string(),
            "Build a TUI app".to_string(),
            "Share with others".to_string(),
            "Write documentation".to_string(),
            "Add more features".to_string(),
        ];
        let filtered_todos = todos.clone();

        Self {
            input_mode: InputMode::Normal,
            search_input: String::new(),
            todos,
            filtered_todos,
            selected_index: Some(0),
        }
    }

    fn move_selection_up(&mut self) {
        self.selected_index = match self.selected_index {
            Some(i) => {
                if i > 0 {
                    Some(i - 1)
                } else {
                    // go to the bottom
                    Some(self.filtered_todos.len() - 1)
                }
            }
            None => {
                if !self.filtered_todos.is_empty() {
                    Some(0)
                } else {
                    None
                }
            }
        };
    }

    fn move_selection_down(&mut self) {
        let len = self.filtered_todos.len();
        self.selected_index = match self.selected_index {
            Some(i) => {
                if i < len - 1 {
                    Some(i + 1)
                } else {
                    // to go the start
                    Some(0)
                }
            }
            None => {
                if !self.filtered_todos.is_empty() {
                    Some(0)
                } else {
                    None
                }
            }
        }
    }

    fn filter_todos(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_todos = self.todos.clone();
        } else {
            let search_term = self.search_input.to_lowercase();
            self.filtered_todos = self
                .todos
                .iter()
                .filter(|todo| todo.to_lowercase().contains(&search_term))
                .cloned()
                .collect();
        }

        // reset selection if it's now out of bounds
        // todo: check if its better to reset the selected_index value every time a todo is searched
        if let Some(selected) = self.selected_index {
            if selected >= self.filtered_todos.len() {
                self.selected_index = if self.filtered_todos.is_empty() {
                    Some(0)
                } else {
                    Some(self.filtered_todos.len() - 1)
                }
            }
        }
    }
}

fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state

    // Run the application
    let res = run_app(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut app = App::new();
    loop {
        terminal.draw(|frame| {
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(frame.area());

            // =========== Search bar ================ //
            let search_text = match app.input_mode {
                InputMode::Searching => format!("Search: {}", app.search_input),
                _ => format!("Press '/' to search (Filter: {})", app.search_input),
            };

            let search_bar = Paragraph::new(Line::from(search_text))
                .style(Style::default())
                .block(Block::default().title("Search").borders(Borders::ALL));

            frame.render_widget(search_bar, main_layout[0]);

            // render the todo list with selection highlight
            let todos: Vec<ListItem> = app
                .filtered_todos
                .iter()
                .enumerate()
                .map(|(i, todo)| {
                    let style = if Some(i) == app.selected_index {
                        Style::default().fg(Color::Blue)
                    } else {
                        Style::default()
                    };
                    let symbol = if Some(i) == app.selected_index {
                        "-> "
                    } else {
                        "- "
                    };
                    let todo_str = format!("{}{}", symbol, todo.as_str());
                    ListItem::new(Line::from(todo_str)).style(style)
                })
                .collect();

            let todos_list = List::new(todos)
                .block(
                    Block::default()
                        .title(format!("Todos ({} shown)", app.filtered_todos.len()))
                        .borders(Borders::ALL),
                )
                .style(Style::default());

            frame.render_widget(todos_list, main_layout[1]);

            // update status bar to show search instructions
            let mode_text = match app.input_mode {
                InputMode::Normal => "Normal Mode | q: quit, /: search, j/k: move, a: add",
                InputMode::Searching => "Search Mode | Enter: apply filter, Esc: clear filter",
                InputMode::Editing => "Edit Mode | Enter: confirm, Esc: cancel",
            };
            let status_bar = Paragraph::new(Line::from(mode_text))
                .style(Style::default())
                .block(Block::default().title("Status").borders(Borders::ALL));

            frame.render_widget(status_bar, main_layout[2]);

            // 4. render the popup if needed (we'll implement this later)
            if false {
                // this will be a condition for showing the popup
                frame.render_widget(
                    Paragraph::new("Popup content")
                        .block(Block::default().title("Popup").borders(Borders::ALL)),
                    centered_rect(50, 50, frame.area()),
                );
            }
        })?;

        // handle events
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            match app.input_mode {
                InputMode::Normal => match code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('/') => {
                        app.input_mode = InputMode::Searching;
                        app.search_input.clear();
                    }
                    KeyCode::Char('j') | KeyCode::Down => app.move_selection_down(),
                    KeyCode::Char('k') | KeyCode::Up => app.move_selection_up(),
                    _ => {}
                },
                InputMode::Searching => match code {
                    KeyCode::Enter => app.input_mode = InputMode::Normal,
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                        app.search_input.clear();
                        // it will reset because the search input is now cleared (empty)
                        app.filter_todos(); // reset to show all todos
                    }
                    KeyCode::Char(c) => {
                        app.search_input.push(c);
                        app.filter_todos(); // update filtered todos on each keystroke
                    }
                    KeyCode::Backspace => {
                        app.search_input.pop();
                        app.filter_todos(); // update filtered todos on backspace
                    }
                    _ => {}
                },
                InputMode::Editing => match code {
                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                    _ => {}
                },
            }
        }
    }
}

// helper function to create a centered rect using percentage of the available area
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // then cut the middle vertical piece into three horizontal pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // return the middle chunk
}
