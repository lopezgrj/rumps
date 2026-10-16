use rusqlite::{params, Connection};
use crate::domain::employer_info::EmployerInfo;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmployerInfoDbError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Employer info not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, EmployerInfoDbError>;

pub fn insert_employer_info(conn: &Connection, patient_dfn: &str, info: &EmployerInfo) -> Result<usize> {
    Ok(conn.execute(
        "INSERT INTO employer_info (patient_dfn, name, address, phone) VALUES (?1, ?2, ?3, ?4)",
        params![
            patient_dfn,
            info.name,
            info.address,
            info.phone
        ],
    )?)
}

pub fn get_employer_infos(conn: &Connection, patient_dfn: &str) -> Result<Vec<EmployerInfo>> {
    let mut stmt = conn.prepare(
        "SELECT name, address, phone FROM employer_info WHERE patient_dfn = ?1"
    )?;
    let infos = stmt
        .query_map([patient_dfn], |row| {
            Ok(EmployerInfo {
                name: row.get(0)?,
                address: row.get(1)?,
                phone: row.get(2)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(infos)
}
