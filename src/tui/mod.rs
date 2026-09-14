//! Mdulo TUI - Interfaz grfica en terminal

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::stdout;
use std::path::Path;

use crate::cache::Cache;

/// Estado de la aplicacin TUI
struct App {
    query: String,
    results: Vec<SearchResultView>,
    selected: usize,
    list_state: ListState,
    cache_loaded: bool,
}

#[derive(Clone)]
struct SearchResultView {
    path: String,
    matches: usize,
    size: u64,
    preview: String,
}

impl App {
    fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            query: String::new(),
            results: Vec::new(),
            selected: 0,
            list_state,
            cache_loaded: false,
        }
    }

    fn search(&mut self, path: &str) -> Result<()> {
        let cache_path = Path::new(".semantic-index.json");

        if !cache_path.exists() {
            self.results = vec![SearchResultView {
                path: " No hay cach indexada. Sal y ejecuta 'index' primero.".to_string(),
                matches: 0,
                size: 0,
                preview: String::new(),
            }];
            return Ok(());
        }

        let cache = Cache::load(cache_path)?;
        self.cache_loaded = true;

        if self.query.is_empty() {
            self.results = cache
                .entries
                .iter()
                .take(50)
                .map(|(p, e)| SearchResultView {
                    path: p.display().to_string(),
                    matches: 0,
                    size: e.size,
                    preview: e.content.lines().take(10).collect::<Vec<_>>().join("\n"),
                })
                .collect();
        } else {
            let query_lower = self.query.to_lowercase();
            let mut results: Vec<SearchResultView> = cache
                .entries
                .iter()
                .filter_map(|(p, e)| {
                    let content_lower = e.content.to_lowercase();
                    if content_lower.contains(&query_lower) {
                        let matches = content_lower.matches(&query_lower).count();
                        Some(SearchResultView {
                            path: p.display().to_string(),
                            matches,
                            size: e.size,
                            preview: e.content.lines().take(15).collect::<Vec<_>>().join("\n"),
                        })
                    } else {
                        None
                    }
                })
                .collect();

            results.sort_by_key(|r| std::cmp::Reverse(r.matches));
            results.truncate(100);
            self.results = results;
        }

        self.selected = 0;
        self.list_state.select(Some(0));
        let _ = path;
        Ok(())
    }

    fn next(&mut self) {
        if self.results.is_empty() {
            return;
        }
        self.selected = (self.selected + 1).min(self.results.len() - 1);
        self.list_state.select(Some(self.selected));
    }

    fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.list_state.select(Some(self.selected));
        }
    }

    fn current(&self) -> Option<&SearchResultView> {
        self.results.get(self.selected)
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            " semcode-search",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled("v3.0.0", Style::default().fg(Color::Green)),
        Span::raw("  "),
        Span::styled("(TUI mode)", Style::default().fg(Color::DarkGray)),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(header, chunks[0]);

    // Search input
    let input = Paragraph::new(app.query.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title("  Search (Enter to search) "),
        );
    f.render_widget(input, chunks[1]);

    // Results + Preview
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[2]);

    // Lista de resultados
    let items: Vec<ListItem> = app
        .results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let is_selected = i == app.selected;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if is_selected { " " } else { "  " };
            ListItem::new(format!("{}{} ({} coinc.)", prefix, r.path, r.matches)).style(style)
        })
        .collect();

    let results_list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(format!("  Results ({}) ", app.results.len())),
    );
    f.render_stateful_widget(results_list, main_chunks[0], &mut app.list_state);

    // Preview
    let preview_content = if let Some(result) = app.current() {
        format!(
            " {}\n\n Coincidencias: {}\n Tamao: {}\n\n{}",
            result.path,
            result.matches,
            format_size(result.size),
            result.preview
        )
    } else {
        "No hay resultados.\n\nEscribe una query y pulsa Enter.".to_string()
    };

    let preview = Paragraph::new(preview_content)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title("  Preview "),
        );
    f.render_widget(preview, main_chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" / ", Style::default().fg(Color::Green)),
        Span::raw("navegar  "),
        Span::styled(" Enter ", Style::default().fg(Color::Green)),
        Span::raw("buscar  "),
        Span::styled(" Esc/q ", Style::default().fg(Color::Green)),
        Span::raw("salir  "),
        Span::styled(" Escribe ", Style::default().fg(Color::Green)),
        Span::raw("para filtrar"),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(footer, chunks[3]);
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App, path: &str) -> Result<()> {
    app.search(path)?;

    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Esc => {
                    return Ok(());
                }
                KeyCode::Char('q') => {
                    if app.query.is_empty() {
                        return Ok(());
                    } else {
                        app.query.clear();
                        app.search(path)?;
                    }
                }
                KeyCode::Char(c) => {
                    app.query.push(c);
                }
                KeyCode::Backspace => {
                    app.query.pop();
                }
                KeyCode::Enter => {
                    app.search(path)?;
                }
                KeyCode::Down => {
                    app.next();
                }
                KeyCode::Up => {
                    app.previous();
                }
                _ => {}
            }
        }
    }
}

pub fn run_tui(path: &str) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app, path);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}
