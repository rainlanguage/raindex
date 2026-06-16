use rain_math_float::Float;
use rusqlite::{functions::FunctionFlags, Connection, Error, Result};

pub fn register(conn: &Connection) -> Result<()> {
    conn.create_scalar_function(
        "FLOAT_ZERO_HEX",
        0,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |_ctx| {
            Float::zero()
                .map(|f| f.as_hex())
                .map_err(|e| Error::UserFunctionError(e.into()))
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_zero_hex() {
        let conn = Connection::open_in_memory().expect("open memory db");
        register(&conn).expect("register float_zero_hex");

        let value: String = conn
            .query_row("SELECT FLOAT_ZERO_HEX()", [], |row| row.get(0))
            .expect("query FLOAT_ZERO_HEX()");

        let expected = Float::zero().unwrap().as_hex();
        assert_eq!(value, expected);
    }
}
