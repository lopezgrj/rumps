use rusqlite::{params, Connection};
use crate::domain::address::Address;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AddressDbError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Address not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, AddressDbError>;

pub fn insert_address(conn: &Connection, patient_dfn: &str, address: &Address) -> Result<usize> {
    Ok(conn.execute(
        "INSERT INTO address (patient_dfn, street, city, county, state, zip, country) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            patient_dfn,
            address.street,
            address.city,
            address.county,
            address.state,
            address.zip,
            address.country
        ],
    )?)
}

pub fn get_addresses(conn: &Connection, patient_dfn: &str) -> Result<Vec<Address>> {
    let mut stmt = conn.prepare(
        "SELECT street, city, county, state, zip, country FROM address WHERE patient_dfn = ?1"
    )?;
    let addresses = stmt
        .query_map([patient_dfn], |row| {
            Ok(Address {
                street: row.get(0)?,
                city: row.get(1)?,
                county: row.get(2)?,
                state: row.get(3)?,
                zip: row.get(4)?,
                country: row.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(addresses)
}
