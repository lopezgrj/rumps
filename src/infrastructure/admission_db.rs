use rusqlite::{params, Connection};
use crate::domain::admission::Admission;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdmissionDbError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Admission not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, AdmissionDbError>;

pub fn insert_admission(conn: &Connection, patient_dfn: &str, admission: &Admission) -> Result<usize> {
    Ok(conn.execute(
        "INSERT INTO admission (patient_dfn, date, ward, discharge_date) VALUES (?1, ?2, ?3, ?4)",
        params![
            patient_dfn,
            admission.date,
            admission.ward,
            admission.discharge_date
        ],
    )?)
}

pub fn get_admissions(conn: &Connection, patient_dfn: &str) -> Result<Vec<Admission>> {
    let mut stmt = conn.prepare(
        "SELECT date, ward, discharge_date FROM admission WHERE patient_dfn = ?1"
    )?;
    let admissions = stmt
        .query_map([patient_dfn], |row| {
            Ok(Admission {
                date: row.get(0)?,
                ward: row.get(1)?,
                discharge_date: row.get(2)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(admissions)
}
