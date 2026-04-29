use rain_math_float::Float;
use rusqlite::{functions::FunctionFlags, Connection, Error, Result};

pub fn register(conn: &Connection) -> Result<()> {
    conn.create_scalar_function(
        "FLOAT_GT_ZERO",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let hex: String = ctx
                .get_raw(0)
                .as_str()
                .map_err(|e| Error::UserFunctionError(e.into()))?
                .to_owned();

            let float = Float::from_hex(&hex).map_err(|e| Error::UserFunctionError(e.into()))?;
            let zero = Float::zero().map_err(|e| Error::UserFunctionError(e.into()))?;
            let is_positive = float
                .gt(zero)
                .map_err(|e| Error::UserFunctionError(e.into()))?;

            Ok(is_positive)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_positive_values_only() {
        let conn = Connection::open_in_memory().expect("open memory db");
        register(&conn).expect("register float_gt_zero");

        let zero_hex = Float::zero().unwrap().as_hex();
        let one_hex = Float::parse("1".to_string()).unwrap().as_hex();
        let negative_hex = Float::parse("-1".to_string()).unwrap().as_hex();

        let zero_positive: bool = conn
            .query_row("SELECT FLOAT_GT_ZERO(?1)", [&zero_hex], |row| row.get(0))
            .expect("query zero");
        let one_positive: bool = conn
            .query_row("SELECT FLOAT_GT_ZERO(?1)", [&one_hex], |row| row.get(0))
            .expect("query positive");
        let negative_positive: bool = conn
            .query_row("SELECT FLOAT_GT_ZERO(?1)", [&negative_hex], |row| {
                row.get(0)
            })
            .expect("query negative");

        assert!(!zero_positive);
        assert!(one_positive);
        assert!(!negative_positive);
    }
}
