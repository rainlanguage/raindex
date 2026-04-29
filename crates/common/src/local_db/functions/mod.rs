pub mod float_is_zero;
pub mod float_negate;
pub mod float_sum;
pub mod float_zero_hex;

use rusqlite::Connection;

pub fn register_all(conn: &Connection) -> Result<(), rusqlite::Error> {
    float_negate::register(conn)?;
    float_is_zero::register(conn)?;
    float_sum::register(conn)?;
    float_zero_hex::register(conn)?;
    Ok(())
}
