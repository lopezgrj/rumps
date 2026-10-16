use rusqlite::{params, Connection};
use crate::domain::deceased_info::DeceasedInfo;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeceasedInfoDbError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Deceased info not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, DeceasedInfoDbError>;

pub fn insert_deceased_info(conn: &Connection, patient_dfn: &str, info: &DeceasedInfo) -> Result<usize> {
    Ok(conn.execute(
        "INSERT INTO deceased_info (patient_dfn, date_time, location, cause, autopsy, notified, notified_date, notified_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            patient_dfn,
            info.date_time,
            info.location,
            info.cause,
            info.autopsy,
            info.notified,
            info.notified_date,
            info.notified_by
        ],
    )?)
}

pub fn get_deceased_info(conn: &Connection, patient_dfn: &str) -> Result<DeceasedInfo> {
    let mut stmt = conn.prepare(
        "SELECT date_time, location, cause, autopsy, notified, notified_date, notified_by FROM deceased_info WHERE patient_dfn = ?1"
    )?;
    let mut rows = stmt.query([patient_dfn])?;
    if let Some(row) = rows.next()? {
        Ok(DeceasedInfo {
            date_time: row.get(0)?,
            location: row.get(1)?,
            cause: row.get(2)?,
            autopsy: row.get(3)?,
            notified: row.get(4)?,
            notified_date: row.get(5)?,
            notified_by: row.get(6)?,
        })
    } else {
        Err(DeceasedInfoDbError::NotFound)
    }
}
