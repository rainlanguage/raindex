use rain_math_float::Float;
use rusqlite::{functions::FunctionFlags, Connection, Error, Result};

pub fn register(conn: &Connection) -> Result<()> {
    conn.create_scalar_function(
        "FLOAT_NEGATE",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let hex: String = ctx
                .get_raw(0)
                .as_str()
                .map_err(|e| Error::UserFunctionError(e.into()))?
                .to_owned();

            let neg_one =
                Float::parse("-1".to_string()).map_err(|e| Error::UserFunctionError(e.into()))?;
            let float = Float::from_hex(&hex).map_err(|e| Error::UserFunctionError(e.into()))?;
            let neg = (neg_one * float).map_err(|e| Error::UserFunctionError(e.into()))?;

            Ok(neg.as_hex())
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negates_hex_value() {
        let conn = Connection::open_in_memory().expect("open memory db");
        register(&conn).expect("register float_negate");

        let one_hex = Float::parse("1".to_string()).unwrap().as_hex();
        let expected = Float::parse("-1".to_string()).unwrap().as_hex();

        let value: String = conn
            .query_row("SELECT FLOAT_NEGATE(?1)", [&one_hex], |row| row.get(0))
            .expect("query FLOAT_NEGATE");

        assert_eq!(value, expected);
    }
}
