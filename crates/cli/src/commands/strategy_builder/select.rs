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

const BOLD_UNDERLINE: &str = "\x1b[1;4m";
const BOLD: &str = "\x1b[1m";
const BOLD_CYAN: &str = "\x1b[1;36m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

pub struct SelectItem {
    pub key: String,
    pub description: String,
}

/// Context shown above a prompt: prior selections and optional description.
pub struct SelectContext<'a> {
    pub header_lines: &'a [String],
    pub description: Option<&'a str>,
}

impl<'a> SelectContext<'a> {
    pub fn new(header_lines: &'a [String]) -> Self {
        Self {
            header_lines,
            description: None,
        }
    }

    pub fn with_description(mut self, desc: &'a str) -> Self {
        self.description = Some(desc);
        self
    }
}

/// Word-wrap `text` to fit within `width` columns. Returns the number of lines.
fn wrap_lines(text: &str, width: usize) -> Vec<String> {
    if text.is_empty() || width == 0 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Write wrapped `text` with the given indent and ANSI style wrapper.
/// Returns number of lines written.
fn write_wrapped(
    w: &mut impl Write,
    text: &str,
    indent: &str,
    style: &str,
    width: usize,
) -> Result<usize> {
    let lines = wrap_lines(text, width);
    for line in &lines {
        write!(w, "{indent}{style}{line}{RESET}\r\n")?;
    }
    Ok(lines.len())
}

/// Write the header (prior selections) and return the number of rows used.
fn write_header(w: &mut impl Write, header_lines: &[String]) -> Result<usize> {
    for line in header_lines {
        write!(w, "  {line}\r\n")?;
    }
    if !header_lines.is_empty() {
        write!(w, "\r\n")?;
        Ok(header_lines.len() + 1)
    } else {
        Ok(0)
    }
}

/// Write bold-underlined title + optional dim description. Returns rows used.
fn write_title(
    w: &mut impl Write,
    title: &str,
    description: Option<&str>,
    cols: usize,
) -> Result<usize> {
    write!(w, "  {BOLD_UNDERLINE}{title}{RESET}\r\n")?;
    let mut rows = 1;
    if let Some(desc) = description {
        rows += write_wrapped(w, desc, "  ", DIM, cols.saturating_sub(2))?;
    }
    write!(w, "\r\n")?;
    Ok(rows + 1)
}

/// Height (in rendered terminal rows) of a single select item.
fn item_height(item: &SelectItem, cols: usize) -> usize {
    // "  ❯ " prefix (4) + trailing margin (1)
    let usable = cols.saturating_sub(5);
    let desc_lines = if item.description.is_empty() {
        0
    } else {
        wrap_lines(&item.description, usable).len()
    };
    1 + desc_lines + 1 // name + description + blank separator
}

/// Count how many items fit starting at `offset`, given `max_rows` available.
fn count_items_fitting(items: &[SelectItem], offset: usize, max_rows: usize, cols: usize) -> usize {
    let mut rows = 0;
    let mut count = 0;
    for item in items.iter().skip(offset) {
        let h = item_height(item, cols);
        if rows + h > max_rows {
            break;
        }
        rows += h;
        count += 1;
    }
    count.max(1)
}

/// Text input rendered in the alt screen.
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
        let (cols, _) = terminal::size()?;
        execute!(w, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;
        write_header(w, header_lines)?;
        write_title(w, prompt, description, cols as usize)?;

        write!(w, "  > {CYAN}{buffer}{RESET}")?;
        if buffer.is_empty() {
            if let Some(d) = default {
                write!(w, "{DIM}{d}{RESET}")?;
            }
        }
        write!(w, "\x1b[?25h")?; // show cursor
        w.flush()?;

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
                KeyCode::Enter => match (buffer.is_empty(), default, allow_empty) {
                    (true, Some(d), _) => return Ok(d.to_string()),
                    (true, None, true) => return Ok(String::new()),
                    (true, None, false) => {} // keep looping — require input
                    (false, _, _) => return Ok(buffer),
                },
                KeyCode::Esc => anyhow::bail!("cancelled"),
                _ => {}
            }
        }
    }
}

/// Scrollable select list.
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

    let mut selected = 0usize;
    let mut scroll = 0usize;

    loop {
        let (cols, rows) = terminal::size()?;
        let cols = cols as usize;
        let rows = rows as usize;

        let header_rows = render_select(w, title, items, selected, scroll, ctx)?;
        let max_rows = rows.saturating_sub(header_rows + 2);

        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected > 0 {
                        selected -= 1;
                        if selected < scroll {
                            scroll = selected;
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected + 1 < items.len() {
                        selected += 1;
                        let fitting = count_items_fitting(items, scroll, max_rows, cols);
                        if selected >= scroll + fitting {
                            scroll += 1;
                        }
                    }
                }
                KeyCode::Enter => return Ok(selected),
                KeyCode::Esc | KeyCode::Char('q') => anyhow::bail!("cancelled"),
                _ => {}
            }
        }
    }
}

/// Render the select list. Returns the number of rows used by the header
/// (everything above the items) so the caller can compute `max_rows`.
fn render_select(
    w: &mut impl Write,
    title: &str,
    items: &[SelectItem],
    selected: usize,
    scroll: usize,
    ctx: &SelectContext,
) -> Result<usize> {
    let (cols, rows) = terminal::size()?;
    let cols = cols as usize;
    let rows = rows as usize;
    execute!(w, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;

    let header_rows = write_header(w, ctx.header_lines)?;
    let title_rows = write_title(w, title, ctx.description, cols)?;
    let total_header = header_rows + title_rows;
    let max_rows = rows.saturating_sub(total_header + 2);

    let mut rows_used = 0;
    for (idx, item) in items.iter().enumerate().skip(scroll) {
        let h = item_height(item, cols);
        if rows_used + h > max_rows {
            break;
        }

        let is_selected = idx == selected;
        let (prefix, style) = if is_selected {
            ("❯ ", BOLD_CYAN)
        } else {
            ("  ", BOLD)
        };
        write!(w, "  {prefix}{style}{}{RESET}\r\n", item.key)?;
        if !item.description.is_empty() {
            write_wrapped(w, &item.description, "      ", DIM, cols.saturating_sub(5))?;
        }
        write!(w, "\r\n")?;
        rows_used += h;
    }

    // Scroll indicators
    let xpos = cols.saturating_sub(3) as u16;
    let ypos_bottom = rows.saturating_sub(2) as u16;
    let ypos_hint = rows.saturating_sub(1) as u16;
    if scroll > 0 {
        execute!(w, cursor::MoveTo(xpos, total_header as u16))?;
        write!(w, " ▲")?;
    }
    if scroll + count_items_fitting(items, scroll, max_rows, cols) < items.len() {
        execute!(w, cursor::MoveTo(xpos, ypos_bottom))?;
        write!(w, " ▼")?;
    }

    // Bottom hint
    execute!(w, cursor::MoveTo(0, ypos_hint))?;
    write!(w, "  {DIM}↑↓ navigate  ⏎ select  esc quit{RESET}")?;

    w.flush()?;
    Ok(total_header)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_empty_text_returns_no_lines() {
        assert!(wrap_lines("", 10).is_empty());
    }

    #[test]
    fn wrap_short_text_returns_single_line() {
        assert_eq!(wrap_lines("hello world", 20), vec!["hello world"]);
    }

    #[test]
    fn wrap_long_text_breaks_on_word_boundaries() {
        let result = wrap_lines("one two three four five", 10);
        assert_eq!(result, vec!["one two", "three four", "five"]);
    }

    #[test]
    fn wrap_very_long_word_goes_on_own_line() {
        let result = wrap_lines("a supercalifragilistic b", 10);
        assert_eq!(result, vec!["a", "supercalifragilistic", "b"]);
    }

    #[test]
    fn wrap_zero_width_returns_no_lines() {
        assert!(wrap_lines("hello", 0).is_empty());
    }

    #[test]
    fn item_height_with_empty_description() {
        let item = SelectItem {
            key: "key".into(),
            description: String::new(),
        };
        // name line + blank separator = 2
        assert_eq!(item_height(&item, 80), 2);
    }

    #[test]
    fn item_height_with_description_wraps() {
        let item = SelectItem {
            key: "key".into(),
            description: "one two three four five six seven eight nine ten".into(),
        };
        // usable width = 80 - 5 = 75 — fits on one line
        assert_eq!(item_height(&item, 80), 3);
    }

    #[test]
    fn item_height_with_description_narrow_terminal() {
        let item = SelectItem {
            key: "key".into(),
            description: "one two three four five six seven eight nine ten".into(),
        };
        // usable = 20 - 5 = 15 — "one two three" (13), "four five six" (13), etc.
        let h = item_height(&item, 20);
        assert!(h > 3, "expected wrapped description, got height {h}");
    }

    #[test]
    fn count_items_fitting_respects_max_rows() {
        let items = vec![
            SelectItem {
                key: "a".into(),
                description: String::new(),
            },
            SelectItem {
                key: "b".into(),
                description: String::new(),
            },
            SelectItem {
                key: "c".into(),
                description: String::new(),
            },
        ];
        // Each item = 2 rows; 5 rows max fits 2 items (4 rows), not 3 (6 rows)
        assert_eq!(count_items_fitting(&items, 0, 5, 80), 2);
    }

    #[test]
    fn count_items_fitting_always_returns_at_least_one() {
        let items = vec![SelectItem {
            key: "a".into(),
            description: "very long description that will wrap many times".into(),
        }];
        // Even with max_rows = 1, we return 1 to avoid an empty list
        assert_eq!(count_items_fitting(&items, 0, 1, 80), 1);
    }
}
