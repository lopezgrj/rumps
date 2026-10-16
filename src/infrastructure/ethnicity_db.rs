use rusqlite::{params, Connection};
use crate::domain::ethnicity::EthnicityInfo;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EthnicityDbError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Ethnicity not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, EthnicityDbError>;

pub fn insert_ethnicity(conn: &Connection, patient_dfn: &str, ethnicity: &EthnicityInfo) -> Result<usize> {
    Ok(conn.execute(
        "INSERT INTO ethnicity (patient_dfn, code, description) VALUES (?1, ?2, ?3)",
        params![
            patient_dfn,
            ethnicity.code,
            ethnicity.description
        ],
    )?)
}

pub fn get_ethnicities(conn: &Connection, patient_dfn: &str) -> Result<Vec<EthnicityInfo>> {
    let mut stmt = conn.prepare(
        "SELECT code, description FROM ethnicity WHERE patient_dfn = ?1"
    )?;
    let ethnicities = stmt
        .query_map([patient_dfn], |row| {
            Ok(EthnicityInfo {
                code: row.get(0)?,
                description: row.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(ethnicities)
}
