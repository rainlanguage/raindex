//! A Select widget that uses crossterm's alternate screen buffer.
//! This avoids dialoguer's cursor-math issues with multi-line items.

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stderr, Write};

pub struct SelectItem {
    pub key: String,
    pub description: String,
}

/// Show a scrollable select list in the alternate screen.
/// Returns the index of the selected item.
pub fn select(title: &str, items: &[SelectItem]) -> Result<usize> {
    if items.is_empty() {
        anyhow::bail!("no items to select from");
    }
    if items.len() == 1 {
        return Ok(0);
    }

    let mut stderr = stderr();
    terminal::enable_raw_mode()?;
    execute!(stderr, EnterAlternateScreen, cursor::Hide)?;

    let result = run_select_loop(&mut stderr, title, items);

    execute!(stderr, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;

    result
}

fn run_select_loop(
    w: &mut impl Write,
    title: &str,
    items: &[SelectItem],
) -> Result<usize> {
    let mut selected: usize = 0;
    let mut scroll_offset: usize = 0;

    loop {
        render(w, title, items, selected, scroll_offset)?;

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
                        // Adjust scroll so selected item stays visible
                        let (_, rows) = terminal::size()?;
                        let visible_rows = visible_item_rows(rows as usize);
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

fn visible_item_rows(term_rows: usize) -> usize {
    // Reserve: 2 for title + blank, 2 for bottom hint + margin
    term_rows.saturating_sub(4)
}

/// Count how many items fit in `max_rows` starting from `offset`,
/// accounting for each item's rendered height.
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

    count.max(1) // always show at least one
}

fn item_height(item: &SelectItem, term_cols: usize) -> usize {
    let usable = term_cols.saturating_sub(4); // indent
    let name_lines = 1;
    let desc_lines = if item.description.is_empty() {
        0
    } else {
        // word-wrap description
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
    name_lines + desc_lines + 1 // +1 for blank line between items
}

fn render(
    w: &mut impl Write,
    title: &str,
    items: &[SelectItem],
    selected: usize,
    scroll_offset: usize,
) -> Result<()> {
    let (cols, rows) = terminal::size()?;
    let cols = cols as usize;
    let rows = rows as usize;

    execute!(w, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;

    // Title
    write!(w, "\x1b[1;4m{title}\x1b[0m\r\n\r\n")?;

    let max_rows = visible_item_rows(rows);
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
            let usable = cols.saturating_sub(6);
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

        write!(w, "\r\n")?; // blank line between items
        rows_used += h;
    }

    // Scroll indicators
    if scroll_offset > 0 {
        execute!(w, cursor::MoveTo(cols as u16 - 3, 2))?;
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
