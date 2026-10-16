use rusqlite::{params, Connection};
use crate::domain::enrollment::Enrollment;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnrollmentDbError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Enrollment not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, EnrollmentDbError>;

pub fn insert_enrollment(conn: &Connection, patient_dfn: &str, enrollment: &Enrollment) -> Result<usize> {
    Ok(conn.execute(
        "INSERT INTO enrollment (patient_dfn, program, enrollment_date, status) VALUES (?1, ?2, ?3, ?4)",
        params![
            patient_dfn,
            enrollment.program,
            enrollment.enrollment_date,
            enrollment.status
        ],
    )?)
}

pub fn get_enrollments(conn: &Connection, patient_dfn: &str) -> Result<Vec<Enrollment>> {
    let mut stmt = conn.prepare(
        "SELECT program, enrollment_date, status FROM enrollment WHERE patient_dfn = ?1"
    )?;
    let enrollments = stmt
        .query_map([patient_dfn], |row| {
            Ok(Enrollment {
                program: row.get(0)?,
                enrollment_date: row.get(1)?,
                status: row.get(2)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(enrollments)
}
