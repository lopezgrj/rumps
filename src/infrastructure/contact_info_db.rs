use rusqlite::{params, Connection};
use crate::domain::contact_info::ContactInfo;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContactInfoDbError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Contact info not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, ContactInfoDbError>;

pub fn insert_contact_info(conn: &Connection, patient_dfn: &str, info: &ContactInfo) -> Result<usize> {
    Ok(conn.execute(
        "INSERT INTO contact_info (patient_dfn, phone, email) VALUES (?1, ?2, ?3)",
        params![
            patient_dfn,
            info.phone,
            info.email
        ],
    )?)
}

pub fn get_contact_info(conn: &Connection, patient_dfn: &str) -> Result<ContactInfo> {
    let mut stmt = conn.prepare(
        "SELECT phone, email FROM contact_info WHERE patient_dfn = ?1"
    )?;
    let mut rows = stmt.query([patient_dfn])?;
    if let Some(row) = rows.next()? {
        Ok(ContactInfo {
            phone: row.get(0)?,
            email: row.get(1)?,
        })
    } else {
        Err(ContactInfoDbError::NotFound)
    }
}
