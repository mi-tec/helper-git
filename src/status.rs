mod diff;

use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use git2::{Repository, Status, StatusOptions};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph},
};
use std::io::stdout;

#[derive(PartialEq)]
enum Focus {
    Left,
    Right,
}

pub fn status(repo: &Repository) -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    // ---------- Load Git Status ----------
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut opts))?;

    let mut items: Vec<ListItem> = Vec::new();
    let mut files: Vec<String> = Vec::new();

    for entry in statuses.iter() {
        let path = match entry.path() {
            Some(p) => p.to_string(),
            None => continue,
        };

        let (label, color) = match entry.status() {
            s if s.contains(Status::WT_NEW) => ("New", Color::Red),
            s if s.contains(Status::WT_MODIFIED) => ("Modified", Color::Yellow),
            s if s.contains(Status::INDEX_NEW)
                || s.contains(Status::WT_RENAMED)
                || s.contains(Status::INDEX_MODIFIED) =>
            {
                ("Added", Color::Green)
            }
            _ => continue,
        };

        files.push(path.clone());

        let line = Line::from(vec![
            Span::styled(
                label,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::raw(path),
        ]);

        items.push(ListItem::new(line));
    }

    if items.is_empty() {
        items.push(ListItem::new("Working tree clean"));
    }

    let mut list_state = ListState::default();
    list_state.select(Some(0));

    // ---------- UI State ----------
    let mut focus = Focus::Left;
    let mut diff_scroll: u16 = 0;
    let mut current_diff: Vec<Line<'static>> = Vec::new();
    let mut last_selected: Option<usize> = None;

    // ---------- Main Loop ----------
    loop {
        // Recalculate diff only if selection changed
        if let Some(selected) = list_state.selected() {
            if Some(selected) != last_selected {
                if let Some(path) = files.get(selected) {
                    current_diff = diff::show_file_diff(repo, path)
                        .unwrap_or_else(|e| vec![Line::from(format!("Error: {}", e))]);
                }
                diff_scroll = 0;
                last_selected = Some(selected);
            }
        }

        // ---------- Helper line ----------
        let help_line = Line::from(vec![
            Span::raw(" ↑↓ / j k "),
            Span::styled("navigate", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" • "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" actions "),
            Span::raw(" • "),
            Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" switch focus "),
            Span::raw(" • "),
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" quit"),
        ]);

        terminal.draw(|frame| {
            let area = frame.area();

            // ---------- Reserve bottom line for helper ----------
            let outer_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),    // top: main panels
                    Constraint::Length(1), // bottom: help line
                ])
                .split(area);

            // ---------- Horizontal panels ----------
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(outer_chunks[0]); // top section

            // ---------- Left Panel ----------
            let left_block = Block::default()
                .title(" Git Status ")
                .borders(Borders::ALL)
                .border_style(if focus == Focus::Left {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                });

            let list = List::new(items.clone())
                .block(left_block)
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("➜ ")
                .highlight_spacing(HighlightSpacing::Always);

            frame.render_stateful_widget(list, chunks[0], &mut list_state);

            // ---------- Right Panel ----------
            let right_block = Block::default()
                .title(" Diff ")
                .borders(Borders::ALL)
                .border_style(if focus == Focus::Right {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                });

            let paragraph = Paragraph::new(current_diff.clone())
                .block(right_block)
                .scroll((diff_scroll, 0));

            frame.render_widget(paragraph, chunks[1]);

            // ---------- Helper Line ----------
            let help_paragraph = Paragraph::new(help_line)
                .alignment(Alignment::Center)
                .style(Style::default().dim());

            frame.render_widget(help_paragraph, outer_chunks[1]);
        })?;

        // ---------- Input Handling ----------
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,

                KeyCode::Tab => {
                    focus = if focus == Focus::Left {
                        Focus::Right
                    } else {
                        Focus::Left
                    };
                }

                KeyCode::Up | KeyCode::Char('k') => match focus {
                    Focus::Left => {
                        if let Some(i) = list_state.selected() {
                            if i > 0 {
                                list_state.select(Some(i - 1));
                            }
                        }
                    }
                    Focus::Right => {
                        diff_scroll = diff_scroll.saturating_sub(1);
                    }
                },

                KeyCode::Down | KeyCode::Char('j') => match focus {
                    Focus::Left => {
                        if let Some(i) = list_state.selected() {
                            if i < items.len().saturating_sub(1) {
                                list_state.select(Some(i + 1));
                            }
                        }
                    }
                    Focus::Right => {
                        diff_scroll = diff_scroll.saturating_add(1);
                    }
                },

                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
