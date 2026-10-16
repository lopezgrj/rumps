use rusqlite::{params, Connection};
use crate::domain::allergy::Allergy;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AllergyDbError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Allergy not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, AllergyDbError>;

pub fn insert_allergy(conn: &Connection, patient_dfn: &str, allergy: &Allergy) -> Result<usize> {
    Ok(conn.execute(
        "INSERT INTO allergy (patient_dfn, substance, reaction) VALUES (?1, ?2, ?3)",
        params![
            patient_dfn,
            allergy.substance,
            allergy.reaction
        ],
    )?)
}

pub fn get_allergies(conn: &Connection, patient_dfn: &str) -> Result<Vec<Allergy>> {
    let mut stmt = conn.prepare(
        "SELECT substance, reaction FROM allergy WHERE patient_dfn = ?1"
    )?;
    let allergies = stmt
        .query_map([patient_dfn], |row| {
            Ok(Allergy {
                substance: row.get(0)?,
                reaction: row.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(allergies)
}
