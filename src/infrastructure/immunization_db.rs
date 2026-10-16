use rusqlite::{params, Connection};
use crate::domain::immunization::Immunization;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImmunizationDbError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Immunization not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, ImmunizationDbError>;

pub fn insert_immunization(conn: &Connection, patient_dfn: &str, immunization: &Immunization) -> Result<usize> {
    Ok(conn.execute(
        "INSERT INTO immunization (patient_dfn, name, date) VALUES (?1, ?2, ?3)",
        params![
            patient_dfn,
            immunization.name,
            immunization.date
        ],
    )?)
}

pub fn get_immunizations(conn: &Connection, patient_dfn: &str) -> Result<Vec<Immunization>> {
    let mut stmt = conn.prepare(
        "SELECT name, date FROM immunization WHERE patient_dfn = ?1"
    )?;
    let immunizations = stmt
        .query_map([patient_dfn], |row| {
            Ok(Immunization {
                name: row.get(0)?,
                date: row.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(immunizations)
}
