use std::{io, time::Duration};

use anyhow::Result;
use tokio::sync::oneshot;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::{app::{App, Mode}, git, models::Repo};

pub fn run(app: &mut App, rx: oneshot::Receiver<Result<Vec<Repo>>>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, app, rx);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    mut rx: oneshot::Receiver<Result<Vec<Repo>>>,
) -> Result<()> {
    while !app.should_quit {
        if let Ok(result) = rx.try_recv() {
            match result {
                Ok(repos) => app.load_repos(repos),
                Err(e) => app.status = format!("Error: {e}"),
            }
        }
        terminal.draw(|frame| draw(frame, app))?;

        if event::poll(Duration::from_millis(200))? {
            let event = event::read()?;
            if let Event::Key(key) = event {
                if key.kind == KeyEventKind::Press {
                    match &app.mode {
                        Mode::Normal => match key.code {
                            KeyCode::Char('q') => app.should_quit = true,
                            KeyCode::Down => app.next(),
                            KeyCode::Up => app.previous(),
                            KeyCode::Enter => {
                                if let Some(repo) = app.selected_repo().cloned() {
                                    if git::dest_exists(&repo.name) {
                                        app.status = format!(
                                            "'{}' exists – enter new folder name (Esc cancel):",
                                            repo.name
                                        );
                                        app.mode =
                                            Mode::Renaming(repo.ssh_url.clone(), repo.name.clone());
                                    } else {
                                        app.status = format!("Cloning {}...", repo.name);
                                        match git::clone_repo(&repo.ssh_url, &repo.name) {
                                            Ok(_) => app.status = format!("Cloned {}", repo.name),
                                            Err(e) => app.status = format!("Clone failed: {e}"),
                                        }
                                    }
                                }
                            }
                            _ => {}
                        },

                        Mode::Renaming(url, _) => {
                            let url = url.clone();
                            match key.code {
                                KeyCode::Esc => {
                                    app.mode = Mode::Normal;
                                    app.status = "↑/↓ move, Enter clone, q quit".to_string();
                                }
                                KeyCode::Enter => {
                                    let dest =
                                        if let Mode::Renaming(_, buf) = &app.mode { buf.clone() } else { String::new() };
                                    if dest.is_empty() {
                                        app.status = "Folder name cannot be empty".to_string();
                                    } else if git::dest_exists(&dest) {
                                        app.status = format!("'{}' also exists – pick another name:", dest);
                                        app.mode = Mode::Renaming(url, dest);
                                    } else {
                                        app.status = format!("Cloning into {}...", dest);
                                        match git::clone_repo(&url, &dest) {
                                            Ok(_) => app.status = format!("Cloned into '{}'", dest),
                                            Err(e) => app.status = format!("Clone failed: {e}"),
                                        }
                                        app.mode = Mode::Normal;
                                    }
                                }
                                KeyCode::Backspace => {
                                    if let Mode::Renaming(u, buf) = &app.mode {
                                        let (mut b, u) = (buf.clone(), u.clone());
                                        b.pop();
                                        app.mode = Mode::Renaming(u, b);
                                    }
                                }
                                KeyCode::Char(c) => {
                                    if let Mode::Renaming(u, buf) = &app.mode {
                                        let (mut b, u) = (buf.clone(), u.clone());
                                        b.push(c);
                                        app.mode = Mode::Renaming(u, b);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn draw(frame: &mut Frame, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(frame.area());

    let items: Vec<ListItem> = app
        .repos
        .iter()
        .map(|repo| ListItem::new(repo.name.clone()))
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Repositories").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::White))
        .highlight_symbol("> ");

    let mut state = ListState::default();
    if !app.repos.is_empty() {
        state.select(Some(app.selected));
    }

    frame.render_stateful_widget(list, layout[0], &mut state);

    let status_text = match &app.mode {
        Mode::Normal => app.status.clone(),
        Mode::Renaming(_, buf) => format!("{} {}", app.status, buf),
    };
    let status = Paragraph::new(status_text)
        .block(Block::default().title("Status").borders(Borders::ALL));
    frame.render_widget(status, layout[1]);
}
