use rusqlite::{params, Connection};
use crate::domain::patient::patient::Patient;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PatientDbError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Patient not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, PatientDbError>;

pub fn insert_patient(conn: &Connection, patient: &Patient) -> Result<usize> {
    // NOTE: The Patient struct now uses addresses, phones, and emergency_contacts as Vecs.
    // This example only inserts the first address, phone, and emergency_contact for compatibility.
    Ok(conn.execute(
        "INSERT INTO patient (dfn, name, dob, sex, ssn, address, phone, date_of_death, primary_care_provider, next_of_kin, emergency_contact) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            patient.dfn,
            patient.name,
            patient.dob,
            patient.sex,
            patient.ssn,
            patient.addresses.get(0).map(|a| format!("{} {} {} {} {} {}", a.street, a.city, a.county.clone().unwrap_or_default(), a.state.clone().unwrap_or_default(), a.zip.clone().unwrap_or_default(), a.country.clone().unwrap_or_default())),
            patient.phones.get(0).map(|p| p.number.clone()),
            patient.date_of_death,
            patient.primary_care_provider,
            patient.next_of_kin,
            patient.emergency_contacts.get(0).map(|e| format!("{} {} {} {}", e.name, e.relationship.clone().unwrap_or_default(), e.phone.clone().unwrap_or_default(), e.address.clone().unwrap_or_default())),
        ],
    )?)
}

pub fn get_patient(conn: &Connection, dfn: &str) -> Result<Patient> {
    let mut stmt = conn.prepare(
        "SELECT dfn, name, dob, sex, ssn, address, phone, date_of_death, primary_care_provider, next_of_kin, emergency_contact FROM patient WHERE dfn = ?1"
    )?;
    let mut rows = stmt.query([dfn])?;
    if let Some(row) = rows.next()? {
        let address: Option<String> = row.get(5)?;
        let phone: Option<String> = row.get(6)?;
        let emergency_contact: Option<String> = row.get(10)?;
        Ok(Patient {
            dfn: row.get(0)?,
            name: row.get(1)?,
            dob: row.get(2)?,
            sex: row.get(3)?,
            ssn: row.get(4)?,
            addresses: address.map_or(vec![], |a| vec![Address { street: a, city: String::new(), county: None, state: None, zip: None, country: None }]),
            phones: phone.map_or(vec![], |p| vec![Phone { number: p, type_: None }]),
            date_of_death: row.get(7)?,
            primary_care_provider: row.get(8)?,
            next_of_kin: row.get(9)?,
            emergency_contacts: emergency_contact.map_or(vec![], |e| vec![EmergencyContact { name: e, relationship: None, phone: None, address: None }]),
            // ...initialize other fields as needed...
            race: vec![],
            ethnicity: vec![],
            visits: vec![],
            appointments: vec![],
            admissions: vec![],
            allergies: vec![],
            immunizations: vec![],
            problems: vec![],
            medications: vec![],
            lab_results: vec![],
            vitals: vec![],
            marital_status: None,
            religion: None,
            contact_info: None,
            eligibility: vec![],
            veteran_status: None,
            pow_status: None,
            insurance: vec![],
            employer: vec![],
            patient_flags: vec![],
            record_flags: vec![],
            enrollments: vec![],
            movements: vec![],
            inpatient_data: vec![],
            outpatient_data: vec![],
            deceased_info: None,
            education: vec![],
            reminders: vec![],
            preferences: vec![],
        })
    } else {
        Err(PatientDbError::NotFound)
    }
}
