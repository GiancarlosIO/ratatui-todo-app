use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

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
    Adding,
    Confirming,
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
    // for add
    input_buffer: String,
    show_confirmation: bool,
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
            input_buffer: String::new(),
            show_confirmation: false,
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

    fn add_todo(&mut self) {
        if !self.input_buffer.is_empty() {
            self.todos.push(self.input_buffer.clone());
            self.input_buffer.clear();
            self.filter_todos(); // refresh filtered list
        }
    }

    fn delete_selected_todo(&mut self) {
        if let Some(selected_index) = self.selected_index {
            // find the corresponding index in the original todos list
            if let Some(selected_todo) = self.filtered_todos.get(selected_index) {
                if let Some(original_index) = self.todos.iter().position(|x| x == selected_todo) {
                    self.todos.remove(original_index);
                    self.filter_todos(); // refresh filtered list

                    // adjust selection
                    if self.filtered_todos.is_empty() {
                        self.selected_index = None
                    } else {
                        self.selected_index =
                            Some(selected_index.min(self.filtered_todos.len() - 1))
                    }
                }
            }
        }
    }

    fn start_delete_confirmation(&mut self) {
        if self.selected_index.is_some() {
            self.input_mode = InputMode::Confirming;
            self.show_confirmation = true
        }
    }

    fn cancel_delete(&mut self) {
        self.input_mode = InputMode::Normal;
        self.show_confirmation = false;
    }

    fn start_editing(&mut self) {
        if let Some(selected_index) = self.selected_index {
            if let Some(todo) = self.filtered_todos.get(selected_index) {
                self.input_buffer = todo.clone();
                self.input_mode = InputMode::Editing;
            }
        }
    }

    fn save_edit(&mut self) {
        if let Some(selected_index) = self.selected_index {
            if let Some(selected_todo) = self.filtered_todos.get(selected_index) {
                if let Some(original_index) = self.todos.iter().position(|x| x == selected_todo) {
                    if !self.input_buffer.is_empty() {
                        self.todos[original_index] = self.input_buffer.clone();
                        self.filter_todos();
                    }
                }
            }
        }
    }

    fn cancel_edit(&mut self) {
        self.input_buffer.clear();
        self.input_mode = InputMode::Normal;
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

            // =========== Render input area (search or add input) ================ //
            let input_text = match app.input_mode {
                InputMode::Searching => format!("Search: {}", app.search_input),
                InputMode::Adding => format!("New todo: {}", app.input_buffer),
                InputMode::Editing => format!("Edit todo: {}", app.input_buffer),
                _ => format!("Press '/' to search (Filter: {})", app.search_input),
            };

            let input_block_title = match app.input_mode {
                InputMode::Adding => "Add todo",
                InputMode::Editing => "Edit todo",
                _ => "Search",
            };

            let input_area = Paragraph::new(Line::from(input_text))
                .style(Style::default())
                .block(
                    Block::default()
                        .title(input_block_title)
                        .borders(Borders::ALL),
                );

            frame.render_widget(input_area, main_layout[0]);

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
                InputMode::Normal => {
                    "Normal Mode | q/esc: quit, /: search, a: add, i: edit, r/d: remove, j/k: move"
                }
                InputMode::Searching => "Search Mode | Enter: apply filter, Esc: clear filter",
                InputMode::Adding => "Add Mode | Enter: save todo, Esc: cancel",
                InputMode::Confirming => "Delete? | y: continue, n/Esc: cancel",
                InputMode::Editing => "Edit Mode | Enter: save changes, Esc: cancel",
            };
            let status_bar = Paragraph::new(Line::from(mode_text))
                .style(Style::default())
                .block(Block::default().title("Status").borders(Borders::ALL));

            frame.render_widget(status_bar, main_layout[2]);

            // render configuration dialog if needed
            if app.show_confirmation {
                // create a temporal string that lives long enough to be used in the Line::from function
                let fallback_string = String::new();
                let selected_todo = app
                    .selected_index
                    .and_then(|i| app.filtered_todos.get(i))
                    .unwrap_or(&fallback_string);

                let popup_area = centered_rect(60, 30, frame.area());
                let confirmation = Paragraph::new(vec![
                    Line::from("Delete this todo?"),
                    Line::from(""),
                    Line::from(selected_todo.as_str()),
                    Line::from(""),
                    Line::from("Press 'y' to confirm or 'n'/Esc to cancel"),
                ])
                .alignment(ratatui::layout::Alignment::Center)
                .block(
                    Block::default()
                        .title("Confirm delete")
                        .borders(Borders::ALL),
                );

                frame.render_widget(
                    Block::default()
                        .style(Style::default().bg(Color::Black))
                        .borders(Borders::ALL),
                    frame.area(),
                );
                frame.render_widget(confirmation, popup_area);
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
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('/') => {
                        app.input_mode = InputMode::Searching;
                        // app.search_input.clear();
                    }
                    KeyCode::Char('a') => {
                        app.input_mode = InputMode::Adding;
                        app.input_buffer.clear();
                    }
                    KeyCode::Char('r') | KeyCode::Char('d') => {
                        app.start_delete_confirmation();
                    }
                    KeyCode::Char('i') => {
                        if app.selected_index.is_some() {
                            app.start_editing();
                        }
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
                InputMode::Adding => match code {
                    KeyCode::Enter => {
                        app.add_todo();
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                        app.input_buffer.clear();
                    }
                    KeyCode::Char(c) => {
                        app.input_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input_buffer.pop();
                    }
                    _ => {}
                },
                InputMode::Confirming => match code {
                    KeyCode::Char('y') => {
                        app.delete_selected_todo();
                        app.cancel_delete();
                    }
                    KeyCode::Char('n') | KeyCode::Esc => {
                        app.cancel_delete();
                    }
                    _ => {}
                },
                InputMode::Editing => match code {
                    KeyCode::Enter => {
                        app.save_edit();
                    }
                    KeyCode::Esc => {
                        app.cancel_edit();
                    }
                    KeyCode::Char(c) => {
                        app.input_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input_buffer.pop();
                    }
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
