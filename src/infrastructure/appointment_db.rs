use rusqlite::{params, Connection};
use crate::domain::appointment::Appointment;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppointmentDbError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Appointment not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, AppointmentDbError>;

pub fn insert_appointment(conn: &Connection, patient_dfn: &str, appointment: &Appointment) -> Result<usize> {
    Ok(conn.execute(
        "INSERT INTO appointment (patient_dfn, date, department, status) VALUES (?1, ?2, ?3, ?4)",
        params![
            patient_dfn,
            appointment.date,
            appointment.department,
            appointment.status
        ],
    )?)
}

pub fn get_appointments(conn: &Connection, patient_dfn: &str) -> Result<Vec<Appointment>> {
    let mut stmt = conn.prepare(
        "SELECT date, department, status FROM appointment WHERE patient_dfn = ?1"
    )?;
    let appointments = stmt
        .query_map([patient_dfn], |row| {
            Ok(Appointment {
                date: row.get(0)?,
                department: row.get(1)?,
                status: row.get(2)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(appointments)
}
