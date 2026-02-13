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

pub fn status(repo: &Repository) -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    let statuses = repo.statuses(Some(&mut opts))?;

    let mut items: Vec<ListItem> = Vec::new();
    let mut files: Vec<String> = Vec::new();

    for entry in statuses.iter() {
        let path = entry.path().unwrap_or_default().to_string();

        let (label, color) = match entry.status() {
            s if s.contains(Status::WT_NEW) => ("New     ", Color::Red),
            s if s.contains(Status::WT_MODIFIED) => ("Modified", Color::Yellow),
            s if s.contains(Status::WT_TYPECHANGE) => ("TypeChange", Color::Rgb(255, 165, 0)),
            s if s.contains(Status::INDEX_NEW)
                || s.contains(Status::WT_RENAMED)
                || s.contains(Status::INDEX_MODIFIED) =>
            {
                ("Added   ", Color::Green)
            }
            _ => continue,
        };

        files.push(path.clone());

        let styled_label = Span::styled(
            label,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        );
        let separator = Span::raw(" | ");
        let path_span = Span::raw(path);

        let line = Line::from(vec![styled_label, separator, path_span]);
        let list_item = ListItem::new(line);

        items.push(list_item);
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "Working tree clean",
            Style::default().add_modifier(Modifier::ITALIC | Modifier::DIM),
        ))));
    }

    let mut list_state = ListState::default();
    if !items.is_empty() {
        list_state.select(Some(0));
    }

    let highlight_style = Style::default()
        .bg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            let block = Block::default().title(" git status ").borders(Borders::ALL);

            let list = List::new(items.clone())
                .block(block)
                .highlight_style(highlight_style)
                .highlight_symbol("➜ ")
                .highlight_spacing(HighlightSpacing::Always);

            frame.render_stateful_widget(list, area, &mut list_state);

            let help_line = Line::from(vec![
                Span::raw(" ↑↓ "),
                Span::styled("navigate", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("  •  "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" quit"),
            ]);

            let help = Paragraph::new(help_line)
                .alignment(Alignment::Center)
                .style(Style::default().dim());

            let help_area = Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1);
            frame.render_widget(help, help_area);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Some(i) = list_state.selected() {
                        if i > 0 {
                            list_state.select(Some(i.saturating_sub(1)));
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if let Some(i) = list_state.selected() {
                        if i < items.len().saturating_sub(1) {
                            list_state.select(Some(i + 1));
                        }
                    } else if !items.is_empty() {
                        list_state.select(Some(0));
                    }
                }
                KeyCode::Home | KeyCode::Char('g') => {
                    if !items.is_empty() {
                        list_state.select(Some(0));
                    }
                }
                KeyCode::End | KeyCode::Char('G') => {
                    if !items.is_empty() {
                        list_state.select(Some(items.len().saturating_sub(1)));
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
