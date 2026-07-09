use super::{SqlStatement, SqlStatementBatch};

#[derive(Debug, Clone, Copy)]
pub struct SqlScript {
    sql: &'static str,
}

impl SqlScript {
    pub const fn new(sql: &'static str) -> Self {
        Self { sql }
    }

    pub fn statement_batch(self) -> SqlStatementBatch {
        SqlStatementBatch::with_statements(
            split_statements(self.sql)
                .into_iter()
                .map(SqlStatement::new)
                .collect(),
        )
    }

    pub fn statements(self) -> Vec<&'static str> {
        split_statements(self.sql)
    }
}

fn split_statements(sql: &'static str) -> Vec<&'static str> {
    let mut statements = Vec::new();
    let mut start = 0usize;
    let mut chars = sql.char_indices().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut has_executable_sql = false;

    while let Some((idx, ch)) = chars.next() {
        let next = chars.peek().map(|(_, ch)| *ch);

        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            continue;
        }

        if in_block_comment {
            if ch == '*' && next == Some('/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if in_single_quote {
            if ch == '\'' {
                if next == Some('\'') {
                    chars.next();
                } else {
                    in_single_quote = false;
                }
            }
            continue;
        }

        if in_double_quote {
            if ch == '"' {
                if next == Some('"') {
                    chars.next();
                } else {
                    in_double_quote = false;
                }
            }
            continue;
        }

        match (ch, next) {
            ('-', Some('-')) => {
                chars.next();
                in_line_comment = true;
            }
            ('/', Some('*')) => {
                chars.next();
                in_block_comment = true;
            }
            ('\'', _) => {
                has_executable_sql = true;
                in_single_quote = true;
            }
            ('"', _) => {
                has_executable_sql = true;
                in_double_quote = true;
            }
            (';', _) => {
                let end = idx + ch.len_utf8();
                let statement = sql[start..end].trim();
                if has_executable_sql && !statement.is_empty() {
                    statements.push(statement);
                }
                start = end;
                has_executable_sql = false;
            }
            _ if !ch.is_whitespace() => has_executable_sql = true,
            _ => {}
        }
    }

    let statement = sql[start..].trim();
    if has_executable_sql && !statement.is_empty() {
        statements.push(statement);
    }

    statements
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_statement_semicolons() {
        assert_eq!(
            split_statements("BEGIN;\nSELECT 1;\nCOMMIT;"),
            vec!["BEGIN;", "SELECT 1;", "COMMIT;"]
        );
    }

    #[test]
    fn ignores_semicolons_inside_quotes_and_comments() {
        assert_eq!(
            split_statements(
                "SELECT ';' AS single_quote;\n\
                 SELECT \";\" AS double_quote;\n\
                 -- comment;\n\
                 SELECT 1;\n\
                 /* block; comment */\n\
                 SELECT 2;"
            ),
            vec![
                "SELECT ';' AS single_quote;",
                "SELECT \";\" AS double_quote;",
                "-- comment;\nSELECT 1;",
                "/* block; comment */\nSELECT 2;"
            ]
        );
    }

    #[test]
    fn keeps_trailing_statement_without_semicolon() {
        assert_eq!(split_statements("SELECT 1"), vec!["SELECT 1"]);
    }

    #[test]
    fn skips_comment_only_fragments() {
        assert_eq!(split_statements("SELECT 1; -- note"), vec!["SELECT 1;"]);
        assert_eq!(split_statements("SELECT 1; /* note */"), vec!["SELECT 1;"]);
        assert!(split_statements("-- note;\n/* also note; */").is_empty());
    }
}
