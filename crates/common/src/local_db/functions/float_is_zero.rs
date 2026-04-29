use rain_math_float::Float;
use rusqlite::{functions::FunctionFlags, Connection, Error, Result};

pub fn register(conn: &Connection) -> Result<()> {
    conn.create_scalar_function(
        "FLOAT_IS_ZERO",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let hex: String = ctx
                .get_raw(0)
                .as_str()
                .map_err(|e| Error::UserFunctionError(e.into()))?
                .to_owned();

            let float = Float::from_hex(&hex).map_err(|e| Error::UserFunctionError(e.into()))?;
            let is_zero = float
                .is_zero()
                .map_err(|e| Error::UserFunctionError(e.into()))?;

            Ok(is_zero)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_zero() {
        let conn = Connection::open_in_memory().expect("open memory db");
        register(&conn).expect("register float_is_zero");

        let zero_hex = Float::zero().unwrap().as_hex();
        let one_hex = Float::parse("1".to_string()).unwrap().as_hex();

        let is_zero: bool = conn
            .query_row("SELECT FLOAT_IS_ZERO(?1)", [&zero_hex], |row| row.get(0))
            .expect("query zero");
        let is_one_zero: bool = conn
            .query_row("SELECT FLOAT_IS_ZERO(?1)", [&one_hex], |row| row.get(0))
            .expect("query non-zero");

        assert!(is_zero);
        assert!(!is_one_zero);
    }
}
