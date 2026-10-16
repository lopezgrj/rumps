use super::race::RaceInfo;
use super::ethnicity::EthnicityInfo;
use super::visit::Visit;
use super::appointment::Appointment;
use super::admission::Admission;
use super::allergy::Allergy;
use super::immunization::Immunization;
use super::problem::Problem;
use super::medication::Medication;
use super::lab_result::LabResult;
use super::vital::Vital;
use super::marital_status::MaritalStatus;
use super::religion::Religion;
use super::address::Address;
use super::contact_info::ContactInfo;
use super::eligibility::Eligibility;
use super::veteran_status::VeteranStatus;
use super::pow_status::POWStatus;
use super::insurance_info::InsuranceInfo;
use super::employer_info::EmployerInfo;
use super::patient_flag::PatientFlag;
use super::record_flag::RecordFlag;
use super::enrollment::Enrollment;
use super::movement::Movement;
use super::inpatient_data::InpatientData;
use super::outpatient_data::OutpatientData;
use super::deceased_info::DeceasedInfo;
use super::patient_education::PatientEducation;
use super::patient_reminder::PatientReminder;
use super::patient_preference::PatientPreference;
use super::phone::Phone;
use super::emergency_contact::EmergencyContact;

// Patient struct using a MUMPS-like structure
// Common MUMPS patient fields and subnodes: DFN, Name, DOB, Sex, SSN, Address, Phone, Date of Death, PCP, Next of Kin, Emergency Contact, Race, Ethnicity, Visits, Appointments, Admissions, Allergies, Immunizations, Problems, Medications, Lab Results, Vitals

#[derive(Debug, Clone)]
pub struct Patient {
    pub dfn: String,         // Internal patient identifier (DFN)
    pub name: String,        // Patient name
    pub dob: String,         // Date of birth (YYYY-MM-DD)
    pub sex: String,         // Sex (M/F/O)
    pub ssn: Option<String>, // Social Security Number (optional)
    pub addresses: Vec<Address>, // Addresses (multiple)
    pub phones: Vec<Phone>,      // Phone numbers (multiple)
    pub date_of_death: Option<String>, // Date of death (optional)
    pub primary_care_provider: Option<String>, // PCP (optional)
    pub next_of_kin: Option<String>, // Next of kin (optional)
    pub emergency_contacts: Vec<EmergencyContact>, // Emergency contacts (multiple)
    pub race: Vec<RaceInfo>, // Race information (multiple)
    pub ethnicity: Vec<EthnicityInfo>, // Ethnicity information (multiple)
    pub visits: Vec<Visit>, // Visits (multiple)
    pub appointments: Vec<Appointment>, // Appointments (multiple)
    pub admissions: Vec<Admission>, // Admissions (multiple)
    pub allergies: Vec<Allergy>, // Allergies (multiple)
    pub immunizations: Vec<Immunization>, // Immunizations (multiple)
    pub problems: Vec<Problem>, // Problems (multiple)
    pub medications: Vec<Medication>, // Medications (multiple)
    pub lab_results: Vec<LabResult>, // Lab results (multiple)
    pub vitals: Vec<Vital>, // Vitals (multiple)
    pub marital_status: Option<MaritalStatus>,
    pub religion: Option<Religion>,
    pub contact_info: Option<ContactInfo>,
    pub eligibility: Vec<Eligibility>,
    pub veteran_status: Option<VeteranStatus>,
    pub pow_status: Option<POWStatus>,
    pub insurance: Vec<InsuranceInfo>,
    pub employer: Vec<EmployerInfo>,
    pub patient_flags: Vec<PatientFlag>,
    pub record_flags: Vec<RecordFlag>,
    pub enrollments: Vec<Enrollment>,
    pub movements: Vec<Movement>,
    pub inpatient_data: Vec<InpatientData>,
    pub outpatient_data: Vec<OutpatientData>,
    pub deceased_info: Option<DeceasedInfo>,
    pub education: Vec<PatientEducation>,
    pub reminders: Vec<PatientReminder>,
    pub preferences: Vec<PatientPreference>,
}
