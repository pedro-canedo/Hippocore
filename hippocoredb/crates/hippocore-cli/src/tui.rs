//! Interactive TUI for browsing and querying Hippocore DB.
//!
//! Launched via `hippocore studio --db <path>`. Uses ratatui + crossterm.
//!
//! ## Layout
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │  [1] Tenants   [2] Memories   [3] Documents   [4] Stats │  ← tab bar
//! ├─────────────────────────────────────────────────────────┤
//! │                                                         │
//! │                  main panel (scrollable list)           │
//! │                                                         │
//! ├─────────────────────────────────────────────────────────┤
//! │  Search: ___________________________    q=quit ↑↓=nav   │  ← status bar
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Keybindings
//!
//! | Key | Action |
//! |-----|--------|
//! | `q` / `Ctrl-C` | Quit |
//! | `1`–`4` | Switch tab |
//! | `Tab` / `Shift-Tab` | Next / previous tab |
//! | `↑` / `k` | Scroll up |
//! | `↓` / `j` | Scroll down |
//! | `/` | Focus search input (Memories tab) |
//! | `Esc` | Blur search input |
//! | `Enter` | Confirm search |
//! | `r` | Refresh data |

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use hippocore::{Config, Hippocore, RecallRequest};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs};
use ratatui::{backend::CrosstermBackend, Terminal};

/// Launch the interactive TUI studio, opening the database at `path`.
/// Blocks until the user quits (`q` or `Ctrl-C`).
pub fn run(path: &PathBuf) -> Result<(), String> {
    let db = Hippocore::open(Config::new(path)).map_err(|e| format!("{e}"))?;

    enable_raw_mode().map_err(|e| format!("terminal error: {e}"))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| format!("terminal error: {e}"))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| format!("terminal error: {e}"))?;

    let result = event_loop(&mut terminal, db);

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result
}

// ─── Application state ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Tenants = 0,
    Memories = 1,
    Documents = 2,
    Stats = 3,
}

impl Tab {
    fn next(self) -> Self {
        match self {
            Tab::Tenants => Tab::Memories,
            Tab::Memories => Tab::Documents,
            Tab::Documents => Tab::Stats,
            Tab::Stats => Tab::Tenants,
        }
    }
    fn prev(self) -> Self {
        match self {
            Tab::Tenants => Tab::Stats,
            Tab::Memories => Tab::Tenants,
            Tab::Documents => Tab::Memories,
            Tab::Stats => Tab::Documents,
        }
    }
    fn as_usize(self) -> usize {
        self as usize
    }
}

struct App {
    tab: Tab,
    db: Hippocore,
    list_state: ListState,
    search_focused: bool,
    search_input: String,
    search_results: Vec<String>,
    tenant_lines: Vec<String>,
    doc_lines: Vec<String>,
    stats_lines: Vec<String>,
}

impl App {
    fn new(db: Hippocore) -> Self {
        let mut app = App {
            tab: Tab::Tenants,
            db,
            list_state: ListState::default(),
            search_focused: false,
            search_input: String::new(),
            search_results: Vec::new(),
            tenant_lines: Vec::new(),
            doc_lines: Vec::new(),
            stats_lines: Vec::new(),
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        self.tenant_lines = build_tenant_lines(&self.db);
        self.doc_lines = build_doc_lines(&self.db);
        self.stats_lines = build_stats_lines(&self.db);
    }

    fn run_search(&mut self) {
        if self.search_input.trim().is_empty() {
            self.search_results.clear();
            return;
        }
        let tenants = self.db.tenants();
        let mut results = Vec::new();
        for tenant in &tenants {
            let mut req = RecallRequest::new(&tenant.id, &self.search_input);
            req.top_k = 20;
            if let Ok(hits) = self.db.recall(req) {
                for h in hits {
                    let snippet: String = h.text.chars().take(80).collect();
                    results.push(format!(
                        "[{:.3}] {}/{} — {snippet}{}",
                        h.score,
                        tenant.id,
                        h.collection,
                        if h.text.len() > 80 { "…" } else { "" }
                    ));
                }
            }
        }
        if results.is_empty() {
            results.push("(no results)".into());
        }
        self.search_results = results;
    }

    fn current_items(&self) -> &[String] {
        match self.tab {
            Tab::Tenants => &self.tenant_lines,
            Tab::Memories => &self.search_results,
            Tab::Documents => &self.doc_lines,
            Tab::Stats => &self.stats_lines,
        }
    }

    fn scroll_down(&mut self) {
        let len = self.current_items().len();
        if len == 0 {
            return;
        }
        let i = self
            .list_state
            .selected()
            .map(|i| (i + 1).min(len - 1))
            .unwrap_or(0);
        self.list_state.select(Some(i));
    }

    fn scroll_up(&mut self) {
        let i = self
            .list_state
            .selected()
            .map(|i| i.saturating_sub(1))
            .unwrap_or(0);
        self.list_state.select(Some(i));
    }

    fn switch_tab(&mut self, tab: Tab) {
        self.tab = tab;
        self.list_state.select(Some(0));
        self.search_focused = false;
    }
}

// ─── Data builders ───────────────────────────────────────────────────────────

fn build_tenant_lines(db: &Hippocore) -> Vec<String> {
    let mut lines = Vec::new();
    let tenants = db.tenants();
    for tenant in &tenants {
        lines.push(format!("▶ {} — {}", tenant.id, tenant.name));
        let cols = db.collections(&tenant.id);
        for col in &cols {
            let desc: String = if col.description.is_empty() {
                "no description".to_string()
            } else {
                col.description.chars().take(60).collect()
            };
            lines.push(format!("  └─ {} ({})", col.name, desc));
        }
        if cols.is_empty() {
            lines.push("  └─ (no collections)".into());
        }
    }
    if lines.is_empty() {
        lines.push("(no tenants — run `hippocore init` to create one)".into());
    }
    lines
}

fn build_doc_lines(db: &Hippocore) -> Vec<String> {
    let mut lines = Vec::new();
    for tenant in db.tenants() {
        for col in db.collections(&tenant.id) {
            for doc in db.list_documents(&tenant.id, Some(&col.name)) {
                let snippet: String = doc.text.chars().take(60).collect();
                lines.push(format!(
                    "[{}] {}/{} — {snippet}{}",
                    doc.id,
                    tenant.id,
                    col.name,
                    if doc.text.len() > 60 { "…" } else { "" }
                ));
            }
        }
    }
    if lines.is_empty() {
        lines.push("(no documents stored)".into());
    }
    lines
}

fn build_stats_lines(db: &Hippocore) -> Vec<String> {
    match db.stats() {
        Ok(s) => vec![
            String::new(),
            format!("  tenants:     {}", s.tenants),
            format!("  collections: {}", s.collections),
            format!("  memories:    {}", s.memories),
            format!("  documents:   {}", s.documents),
            format!("  chunks:      {}", s.chunks),
            format!("  records:     {}", s.records),
            format!("  files:       {}", s.files),
            String::new(),
            format!("  index entries: {}", s.indexed_entries),
            format!("  WAL entries:   {}", s.wal_entries),
            format!("  disk usage:    {} B", s.disk_bytes),
        ],
        Err(e) => vec![format!("error loading stats: {e}")],
    }
}

// ─── Event loop ───────────────────────────────────────────────────────────────

fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    db: Hippocore,
) -> Result<(), String> {
    let mut app = App::new(db);

    loop {
        terminal
            .draw(|f| render(f, &mut app))
            .map_err(|e| format!("render error: {e}"))?;

        if event::poll(Duration::from_millis(200)).map_err(|e| format!("event error: {e}"))? {
            if let Event::Key(key) = event::read().map_err(|e| format!("event error: {e}"))? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                    break;
                }

                if app.search_focused {
                    match key.code {
                        KeyCode::Esc => app.search_focused = false,
                        KeyCode::Enter => {
                            app.run_search();
                            app.search_focused = false;
                            app.list_state.select(Some(0));
                        }
                        KeyCode::Backspace => {
                            app.search_input.pop();
                        }
                        KeyCode::Char(c) => app.search_input.push(c),
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('1') => app.switch_tab(Tab::Tenants),
                        KeyCode::Char('2') => app.switch_tab(Tab::Memories),
                        KeyCode::Char('3') => app.switch_tab(Tab::Documents),
                        KeyCode::Char('4') => app.switch_tab(Tab::Stats),
                        KeyCode::Tab => app.switch_tab(app.tab.next()),
                        KeyCode::BackTab => app.switch_tab(app.tab.prev()),
                        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                        KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                        KeyCode::Char('/') if app.tab == Tab::Memories => {
                            app.search_focused = true;
                        }
                        KeyCode::Char('r') => app.refresh(),
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

// ─── Rendering ───────────────────────────────────────────────────────────────

fn render(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    // Tab bar.
    let tab_titles: Vec<Line> = vec![
        Line::from(" 1 Tenants "),
        Line::from(" 2 Memories "),
        Line::from(" 3 Documents "),
        Line::from(" 4 Stats "),
    ];
    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Hippocore Studio "),
        )
        .select(app.tab.as_usize())
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    // Main panel.
    let panel_title = match app.tab {
        Tab::Tenants => " Tenants & Collections ",
        Tab::Memories => " Memory Search Results ",
        Tab::Documents => " Documents ",
        Tab::Stats => " Database Statistics ",
    };

    let owned_items: Vec<String> = app.current_items().to_vec();
    let empty_hint = match app.tab {
        Tab::Memories => "(press / to search memories)",
        _ => "(empty)",
    };
    let items: Vec<ListItem> = if owned_items.is_empty() {
        vec![ListItem::new(empty_hint)]
    } else {
        owned_items
            .iter()
            .map(|s| ListItem::new(s.as_str()))
            .collect()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(panel_title))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, chunks[1], &mut app.list_state);

    // Status / search bar.
    let search_style = if app.search_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let (label, hint) = if app.tab == Tab::Memories {
        let l = format!(" Search: {}", app.search_input);
        let h = if app.search_focused {
            "Enter=search  Esc=cancel"
        } else {
            "/ to search  ↑↓/jk=nav  Tab=next  r=refresh  q=quit"
        };
        (l, h)
    } else {
        (
            String::new(),
            "↑↓/jk=nav  1-4 or Tab=switch tab  r=refresh  q=quit",
        )
    };

    let spans = vec![
        Span::styled(label, search_style),
        Span::styled(format!("  {hint}"), Style::default().fg(Color::DarkGray)),
    ];

    let status = Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);
}
