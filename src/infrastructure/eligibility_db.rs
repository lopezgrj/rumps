use rusqlite::{params, Connection};
use crate::domain::eligibility::Eligibility;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EligibilityDbError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Eligibility not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, EligibilityDbError>;

pub fn insert_eligibility(conn: &Connection, patient_dfn: &str, eligibility: &Eligibility) -> Result<usize> {
    Ok(conn.execute(
        "INSERT INTO eligibility (patient_dfn, code, description) VALUES (?1, ?2, ?3)",
        params![
            patient_dfn,
            eligibility.code,
            eligibility.description
        ],
    )?)
}

pub fn get_eligibilities(conn: &Connection, patient_dfn: &str) -> Result<Vec<Eligibility>> {
    let mut stmt = conn.prepare(
        "SELECT code, description FROM eligibility WHERE patient_dfn = ?1"
    )?;
    let eligibilities = stmt
        .query_map([patient_dfn], |row| {
            Ok(Eligibility {
                code: row.get(0)?,
                description: row.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(eligibilities)
}
