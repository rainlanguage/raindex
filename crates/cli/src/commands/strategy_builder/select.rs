//! Select and Input widgets rendered directly to a writer.
//! The caller manages the alternate screen lifecycle.

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, ClearType},
};
use std::io::Write;

pub struct SelectItem {
    pub key: String,
    pub description: String,
}

/// Text input rendered in the alt screen.
/// Shows header lines for context, a prompt, and an editable line.
pub fn input(
    w: &mut impl Write,
    prompt: &str,
    description: Option<&str>,
    default: Option<&str>,
    allow_empty: bool,
    header_lines: &[String],
) -> Result<String> {
    let mut buffer = String::new();

    loop {
        render_input(w, prompt, description, default, &buffer, header_lines)?;

        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            match code {
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                    anyhow::bail!("cancelled");
                }
                KeyCode::Char(c) => buffer.push(c),
                KeyCode::Backspace => {
                    buffer.pop();
                }
                KeyCode::Enter => {
                    if buffer.is_empty() {
                        if let Some(d) = default {
                            return Ok(d.to_string());
                        }
                        if allow_empty {
                            return Ok(String::new());
                        }
                        // else loop — require non-empty
                    } else {
                        return Ok(buffer);
                    }
                }
                KeyCode::Esc => anyhow::bail!("cancelled"),
                _ => {}
            }
        }
    }
}

fn render_input(
    w: &mut impl Write,
    prompt: &str,
    description: Option<&str>,
    default: Option<&str>,
    buffer: &str,
    header_lines: &[String],
) -> Result<()> {
    let (cols, _) = terminal::size()?;
    let cols = cols as usize;

    execute!(w, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;

    for line in header_lines {
        write!(w, "  {line}\r\n")?;
    }
    if !header_lines.is_empty() {
        write!(w, "\r\n")?;
    }

    write!(w, "  \x1b[1;4m{prompt}\x1b[0m\r\n\r\n")?;

    if let Some(desc) = description {
        // Simple word wrap for description
        let usable = cols.saturating_sub(4);
        let mut col = 0usize;
        write!(w, "  \x1b[2m")?;
        for word in desc.split_whitespace() {
            let wlen = word.len();
            if col == 0 {
                write!(w, "{word}")?;
                col = wlen;
            } else if col + 1 + wlen <= usable {
                write!(w, " {word}")?;
                col += 1 + wlen;
            } else {
                write!(w, "\x1b[0m\r\n  \x1b[2m{word}")?;
                col = wlen;
            }
        }
        write!(w, "\x1b[0m\r\n\r\n")?;
    }

    // The editable line
    write!(w, "  > \x1b[36m{buffer}\x1b[0m")?;
    if buffer.is_empty() {
        if let Some(d) = default {
            write!(w, "\x1b[2m{d}\x1b[0m")?;
        }
    }
    write!(w, "\x1b[?25h")?; // show cursor

    w.flush()?;
    Ok(())
}

/// Header lines to display above the select list.
/// These show prior selections / progress.
pub struct SelectContext<'a> {
    pub header_lines: &'a [String],
    pub description: Option<&'a str>,
}

impl<'a> SelectContext<'a> {
    pub fn new(header_lines: &'a [String]) -> Self {
        Self { header_lines, description: None }
    }
    pub fn with_description(mut self, desc: &'a str) -> Self {
        self.description = Some(desc);
        self
    }
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

    // Title (bold + underlined)
    write!(w, "  \x1b[1;4m{title}\x1b[0m\r\n")?;

    // Description under title (dim, word-wrapped), no underline
    let mut desc_lines = 0;
    if let Some(desc) = ctx.description {
        let usable = cols.saturating_sub(4);
        let mut col = 0usize;
        write!(w, "  \x1b[2m")?;
        for word in desc.split_whitespace() {
            let wlen = word.len();
            if col == 0 {
                write!(w, "{word}")?;
                col = wlen;
            } else if col + 1 + wlen <= usable {
                write!(w, " {word}")?;
                col += 1 + wlen;
            } else {
                write!(w, "\x1b[0m\r\n  \x1b[2m{word}")?;
                col = wlen;
                desc_lines += 1;
            }
        }
        write!(w, "\x1b[0m\r\n")?;
        desc_lines += 1;
    }
    write!(w, "\r\n")?;

    let header_rows = ctx.header_lines.len()
        + if ctx.header_lines.is_empty() { 0 } else { 1 }
        + 1 // title
        + desc_lines
        + 1; // blank before items
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
    if scroll_offset > 0 {
        execute!(w, cursor::MoveTo(cols as u16 - 3, header_rows as u16))?;
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
