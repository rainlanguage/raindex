use rain_math_float::Float;
use rusqlite::{functions::FunctionFlags, Connection, Error, Result};

pub fn register(conn: &Connection) -> Result<()> {
    conn.create_aggregate_function(
        "FLOAT_SUM",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        FloatSum,
    )
}

struct FloatSum;

impl rusqlite::functions::Aggregate<Float, Option<String>> for FloatSum {
    fn init(&self, _: &mut rusqlite::functions::Context<'_>) -> Result<Float> {
        Float::zero().map_err(|e| Error::UserFunctionError(e.into()))
    }

    fn step(&self, ctx: &mut rusqlite::functions::Context<'_>, acc: &mut Float) -> Result<()> {
        if ctx.len() != 1 {
            return Err(Error::UserFunctionError(
                "FLOAT_SUM() requires exactly 1 argument".into(),
            ));
        }

        let hex: String = ctx.get::<String>(0)?;

        let value = Float::from_hex(&hex).map_err(|e| Error::UserFunctionError(e.into()))?;
        *acc = (*acc + value).map_err(|e| Error::UserFunctionError(e.into()))?;
        Ok(())
    }

    fn finalize(
        &self,
        _: &mut rusqlite::functions::Context<'_>,
        acc: Option<Float>,
    ) -> Result<Option<String>> {
        acc.map(|f| Ok(f.as_hex())).transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_hex_values() {
        let conn = Connection::open_in_memory().expect("open memory db");
        register(&conn).expect("register float_sum");

        let one_hex = Float::parse("1".to_string()).unwrap().as_hex();
        let two_point_five_hex = Float::parse("2.5".to_string()).unwrap().as_hex();

        let single: Option<String> = conn
            .query_row("SELECT FLOAT_SUM(?1)", [&one_hex], |row| row.get(0))
            .expect("sum single");
        assert_eq!(
            single.unwrap(),
            Float::parse("1".to_string()).unwrap().as_hex()
        );

        let total: Option<String> = conn
            .query_row(
                "SELECT FLOAT_SUM(x) FROM (SELECT ?1 AS x UNION ALL SELECT ?2 AS x)",
                [&one_hex, &two_point_five_hex],
                |row| row.get(0),
            )
            .expect("sum two");

        let expected = (Float::from_hex(&one_hex).unwrap()
            + Float::from_hex(&two_point_five_hex).unwrap())
        .unwrap()
        .as_hex();
        assert_eq!(total.unwrap(), expected);
    }

    #[test]
    fn returns_null_when_no_rows() {
        let conn = Connection::open_in_memory().expect("open memory db");
        register(&conn).expect("register float_sum");

        let result: Option<String> = conn
            .query_row(
                "SELECT FLOAT_SUM(x) FROM (SELECT NULL AS x WHERE 1 = 0)",
                [],
                |row| row.get(0),
            )
            .expect("sum empty");
        assert!(result.is_none());
    }
}
