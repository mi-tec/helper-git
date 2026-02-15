use anyhow::Result;
use git2::{DiffFormat, DiffOptions, Repository, Status};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use std::fs;
use std::path::Path;

pub fn show_file_diff(repo: &Repository, path: &str) -> Result<Vec<Line<'static>>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    // ---------- Check file status ----------
    let status = repo.status_file(Path::new(path))?;

    // ---------- If untracked (WT_NEW) ----------
    if status.contains(Status::WT_NEW) {
        let full_path = repo.workdir().unwrap().join(path);

        let content = fs::read_to_string(full_path)?;

        lines.push(Line::from(Span::styled(
            format!("New file: {}\n", path),
            Style::default().fg(Color::Blue),
        )));

        for line in content.lines() {
            lines.push(Line::from(Span::styled(
                format!("+{}\n", line),
                Style::default().fg(Color::Green),
            )));
        }

        return Ok(lines);
    }

    // ---------- Otherwise normal diff ----------
    let mut opts = DiffOptions::new();
    opts.pathspec(path);
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(true);

    let head = repo.head().ok();
    let tree = head.and_then(|h| h.peel_to_tree().ok());

    let diff = repo.diff_tree_to_workdir_with_index(tree.as_ref(), Some(&mut opts))?;

    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let content = std::str::from_utf8(line.content())
            .unwrap_or("")
            .to_string();

        let span = match line.origin() {
            '+' => Span::styled(content, Style::default().fg(Color::Green)),
            '-' => Span::styled(content, Style::default().fg(Color::Red)),
            'F' => Span::styled(content, Style::default().fg(Color::Blue)),
            _ => Span::raw(content),
        };

        lines.push(Line::from(span));
        true
    })?;

    if lines.is_empty() {
        lines.push(Line::from("No changes"));
    }

    Ok(lines)
}
