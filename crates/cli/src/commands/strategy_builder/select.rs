//! A Select widget rendered directly to a writer.
//! The caller manages the alternate screen lifecycle.

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};
use std::io::Write;

pub struct SelectItem {
    pub key: String,
    pub description: String,
}

/// Header lines to display above the select list.
/// These show prior selections / progress.
pub struct SelectContext<'a> {
    pub header_lines: &'a [String],
}

/// Run a select list on the given writer.
/// The caller is responsible for alternate screen and raw mode.
pub fn select(
    w: &mut impl Write,
    title: &str,
    items: &[SelectItem],
    ctx: &SelectContext,
) -> Result<usize> {
    if items.is_empty() {
        anyhow::bail!("no items to select from");
    }
    if items.len() == 1 {
        return Ok(0);
    }

    let mut selected: usize = 0;
    let mut scroll_offset: usize = 0;

    loop {
        render(w, title, items, selected, scroll_offset, ctx)?;

        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected > 0 {
                        selected -= 1;
                        if selected < scroll_offset {
                            scroll_offset = selected;
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected + 1 < items.len() {
                        selected += 1;
                        let (_, rows) = terminal::size()?;
                        let header_rows = ctx.header_lines.len() + 4; // header + title + gaps
                        let visible_rows = (rows as usize).saturating_sub(header_rows + 2);
                        let items_visible =
                            count_items_fitting(items, scroll_offset, visible_rows);
                        if selected >= scroll_offset + items_visible {
                            scroll_offset += 1;
                        }
                    }
                }
                KeyCode::Enter => return Ok(selected),
                KeyCode::Esc | KeyCode::Char('q') => {
                    anyhow::bail!("selection cancelled");
                }
                _ => {}
            }
        }
    }
}

fn count_items_fitting(items: &[SelectItem], offset: usize, max_rows: usize) -> usize {
    let (cols, _) = terminal::size().unwrap_or((80, 24));
    let cols = cols as usize;
    let mut rows_used = 0;
    let mut count = 0;

    for item in items.iter().skip(offset) {
        let h = item_height(item, cols);
        if rows_used + h > max_rows {
            break;
        }
        rows_used += h;
        count += 1;
    }

    count.max(1)
}

fn item_height(item: &SelectItem, term_cols: usize) -> usize {
    let usable = term_cols.saturating_sub(7); // "  ❯ " prefix + margin
    let name_lines = 1;
    let desc_lines = if item.description.is_empty() {
        0
    } else {
        let mut lines = 1usize;
        let mut col = 0usize;
        for word in item.description.split_whitespace() {
            let wlen = word.len();
            if col == 0 {
                col = wlen;
            } else if col + 1 + wlen <= usable {
                col += 1 + wlen;
            } else {
                lines += 1;
                col = wlen;
            }
        }
        lines
    };
    name_lines + desc_lines + 1 // +1 blank line between items
}

fn render(
    w: &mut impl Write,
    title: &str,
    items: &[SelectItem],
    selected: usize,
    scroll_offset: usize,
    ctx: &SelectContext,
) -> Result<()> {
    let (cols, rows) = terminal::size()?;
    let cols = cols as usize;
    let rows = rows as usize;

    execute!(w, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;

    // Header: show prior selections
    if !ctx.header_lines.is_empty() {
        for line in ctx.header_lines {
            write!(w, "  {line}\r\n")?;
        }
        write!(w, "\r\n")?;
    }

    // Title
    write!(w, "  \x1b[1;4m{title}\x1b[0m\r\n\r\n")?;

    let header_rows = ctx.header_lines.len() + 4;
    let max_rows = rows.saturating_sub(header_rows + 2);
    let mut rows_used = 0;

    for (idx, item) in items.iter().enumerate().skip(scroll_offset) {
        let h = item_height(item, cols);
        if rows_used + h > max_rows {
            break;
        }

        let is_selected = idx == selected;
        let prefix = if is_selected { "❯ " } else { "  " };
        let name_style = if is_selected { "\x1b[1;36m" } else { "\x1b[1m" };

        write!(w, "  {prefix}{name_style}{}\x1b[0m\r\n", item.key)?;

        if !item.description.is_empty() {
            let usable = cols.saturating_sub(7);
            let mut col = 0usize;
            write!(w, "      \x1b[2m")?;
            for word in item.description.split_whitespace() {
                let wlen = word.len();
                if col == 0 {
                    write!(w, "{word}")?;
                    col = wlen;
                } else if col + 1 + wlen <= usable {
                    write!(w, " {word}")?;
                    col += 1 + wlen;
                } else {
                    write!(w, "\x1b[0m\r\n      \x1b[2m{word}")?;
                    col = wlen;
                }
            }
            write!(w, "\x1b[0m\r\n")?;
        }

        write!(w, "\r\n")?;
        rows_used += h;
    }

    // Scroll indicators
    let first_item_row = header_rows;
    if scroll_offset > 0 {
        execute!(w, cursor::MoveTo(cols as u16 - 3, first_item_row as u16))?;
        write!(w, " ▲")?;
    }
    if scroll_offset + count_items_fitting(items, scroll_offset, max_rows) < items.len() {
        execute!(w, cursor::MoveTo(cols as u16 - 3, rows as u16 - 2))?;
        write!(w, " ▼")?;
    }

    // Bottom hint
    execute!(w, cursor::MoveTo(0, rows as u16 - 1))?;
    write!(w, "  \x1b[2m↑↓ navigate  ⏎ select  esc quit\x1b[0m")?;

    w.flush()?;
    Ok(())
}
