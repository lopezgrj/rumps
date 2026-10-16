# RUMPS Specification

## 1. Purpose

RUMPS is an API-only Rust service intended to expose data and workflows across
all domains in a target VistA system. Its scope includes clinical,
administrative, operational, reference, and security domains; it is not limited
to patient demographics or clinical records.

VistA is a collection of applications and packages built on MUMPS and FileMan,
not one uniform schema. Compatibility means coverage of the selected target
installation's packages, data, and externally available operations, not merely
using MUMPS-like names or implementing a short list of file numbers.

This document specifies the intended system and records the prototype's
implementation status. Requirements here are not claims that the current code
already satisfies them.

## 2. Scope

### In scope

- Provide a machine-to-machine API as the application's only product
  interface.
- Expose all in-scope domains and workflows present in the target VistA
  installation, as identified by the coverage inventory in Section 4.5.
- Preserve the identities, relationships, workflows, permissions, and
  provenance required to represent those domains faithfully.
- Persist API data in SQLite for development and PostgreSQL for production.
- Integrate with the pinned VistA system through supported interfaces and
  document source-of-truth and synchronization behavior for each resource.

### Out of scope for the current specification

- Any user-facing interface other than the API, including graphical,
  command-line, and desktop applications. Operational command-line tooling
  used only to administer or develop the service is not a product interface.
- Replacing clinical judgment or inventing workflows not supported by the
  target VistA packages.
- Claiming compatibility with every VistA installation without versioned
  target-system inventory and validation.

## 3. Users and operating context

The API serves authorized applications and services; it is not intended for
direct end-user interaction. Its contract shall document supported operations,
data shapes, validation, authorization, errors, and versioning. The transport
shall be HTTP with JSON as the default and primary client interface. The API
shall support HL7 FHIR R4 (4.0.1) as its healthcare interoperability standard
and JSON as the default FHIR representation. Any additional transport,
serialization format, or FHIR release requires explicit documentation and
shall not replace the HTTP/JSON and FHIR R4 baseline.

The current implementation is an HTTP API foundation and does not yet
expose VistA clinical resources or satisfy the complete coverage requirements.

The service uses OIDC/OAuth2 JWT bearer authentication for machine-to-machine
access. Protected operations accept service principals only: each token must
have the configured issuer, audience, expiration, subject, and an allowlisted
`azp` or `client_id`. If both client claims are present, they must match. The
allowlist shall contain client-credentials service registrations; delegated
end-user/VistA identity is not supported. No operation representing an
individual user's action may be exposed until delegated identity mapping and
authorization are implemented.

The liveness, readiness, FHIR metadata, and OpenAPI discovery routes are public.
All other routes require a valid bearer token; if OIDC is not configured,
non-public routes fail closed. Production requires OIDC issuer, audience, JWKS
URL, and allowed client IDs, and uses HTTPS for issuer/JWKS access. Signing keys
are loaded from the configured JWKS at startup; deployment procedures must
restart the API after key rotation. This service-principal authentication does
not itself provide resource-level authorization, facility/patient scopes, or
VistA user audit attribution.

## 4. Data requirements

### 4.1 Patient

Each patient record has a required internal identifier (`dfn`), name, date of
birth, and sex. Social Security number, date of death, primary care provider,
next of kin, marital status, religion, contact information, veteran status,
POW status, and deceased-person details are optional in the domain model.

The patient record can also include the following related data:

| Category | Multiplicity |
| --- | --- |
| Addresses and phone numbers | Zero or more |
| Emergency contacts | Zero or more |
| Race and ethnicity entries | Zero or more |
| Visits, appointments, and admissions | Zero or more each |
| Allergies, immunizations, and problems | Zero or more each |
| Medications, laboratory results, and vital measurements | Zero or more each |
| Eligibility entries, insurance policies, and employers | Zero or more each |
| Patient flags and record flags | Zero or more each |
| Program enrollments and patient movements | Zero or more each |
| Inpatient and outpatient records | Zero or more each |
| Patient education entries, reminders, and preferences | Zero or more each |

Where a value is optional, the domain model represents it as optional rather
than requiring a placeholder value. The model currently uses strings for dates,
identifiers, status values, and many coded or descriptive values; it does not
yet define shared validation or canonical code systems for them.

### 4.2 Orders

An order has an identifier, patient identifier, order type, and status. It may
also have a provider, location, start and stop dates, order text, orderable
item, reason, signature, and verification value. An order may contain multiple
actions and activity-log entries.

Each order action records an action type and may record its date, actor, and
comment. Each activity entry records an activity type and may record its date,
actor, and details. The current code defines these domain structures only; it
does not define order persistence or order-processing behavior.

### 4.3 Clinical and administrative data domains

MUMPS is a programming language, not a single healthcare data model. The
existing MUMPS file-number references suggest VistA as the intended point of
comparison. The following domains are required wherever they exist in the
target installation. This is a domain checklist, not a claim that the current
code supports them or that every VistA installation has an identical schema.

| Domain | Additional data to represent |
| --- | --- |
| People, providers, and staff | Person identity; provider/staff roles and specialties; credentials and service affiliations; care-team membership |
| Institutions and locations | Facility, division, service/section, clinic, ward, room, and bed identities and their relationships |
| Encounters and clinical documentation | Encounter date/time, type, status, location, responsible providers, diagnoses, procedures, notes, and links to associated orders and results |
| Scheduling | Appointment requests and booked slots, clinic/resource availability, cancellations, check-ins, and appointment outcomes |
| Diagnoses and procedures | Coded diagnosis and procedure, coding system and code version, status, dates, encounter association, and responsible clinician |
| Pharmacy | Drug catalog identity, medication orders, prescriptions, dispensing/fill events, administration events, medication status, and links to orders and encounters |
| Laboratory | Test catalog, orders, collection/specimen details, result components, units, reference ranges, abnormal indicators, result status, and corrections |
| Radiology and imaging | Imaging orders, modality, body site, performed procedure, report/findings, result status, and references to externally stored images |
| Consults and referrals | Requesting and receiving services, reason, priority, status, dates, provider, and outcome |
| Benefits and billing | Eligibility/coverage determinations, authorizations, billable services, claims, and payment/adjustment status |
| Patient identity and record linkage | Source-system identifiers, identity matching, duplicate-record resolution, merge history, and retained prior identifiers |
| Privacy, consent, and access | Consent/directives, disclosure restrictions, sensitive-record indicators, user identity, access decisions, and access/change audit events |
| Clinical provenance | Author/source, entered and effective times, verification/signature state, corrections, and the relationship between a record and its source system |

### 4.4 Enterprise and package-specific data domains

The API shall cover the operational and package-specific domains present in
the target VistA installation, in addition to the domains in Section 4.3.
These include, where installed:

| Domain | Additional data to represent |
| --- | --- |
| Registration and ADT | Registration events, patient class, admission/discharge/transfer events, bed requests and assignments, movement history, and responsible facility |
| Nursing and care delivery | Nursing assessments, care plans, observations, interventions, handoffs, and completion/verification status |
| Mental and behavioral health | Assessments, diagnoses, treatment plans, clinical notes, screening results, and stricter confidentiality/access rules where applicable |
| Surgery and anesthesia | Surgical case, scheduling and procedure details, pre/post-operative documentation, anesthesia record, and outcomes |
| Dental care | Dental encounters, findings, treatment plans, procedures, and imaging references |
| Nutrition and dietetics | Nutrition assessments, diet orders, restrictions, meal services, and delivery status |
| Prosthetics and durable medical equipment | Requests, authorizations, devices/services provided, fitting and delivery events, maintenance, and return/disposition |
| Blood bank and transfusion | Blood product identity, compatibility testing, issue/administration events, reactions, and traceability |
| Women's health and reproductive care | Screenings, procedures, pregnancy-related records, results, follow-up, and restricted-access rules where applicable |
| Oncology | Cancer diagnoses, staging, treatment plans, therapies, tumor registry data, and longitudinal outcomes |
| Home and community-based care | Referrals, home visits, care plans, services delivered, equipment, and coordination with external providers |
| Long-term and extended care | Assessments, placement, care plans, facility/service episodes, and transitions of care |
| Occupational and environmental health | Exposure history, workplace/environmental assessments, surveillance, and follow-up |
| Suicide prevention and safety follow-up | Risk assessments, safety plans, outreach tasks, escalation, and completion outcomes |
| Materials and inventory | Supply and drug stock, locations, receipts, issues, lots, expiration dates, and inventory adjustments |
| Revenue cycle and accounts receivable | Charges, billing events, copayments, third-party claims, receivables, adjustments, and payment status |
| Beneficiary travel and reimbursements | Requests, eligibility, authorizations, trip details, mileage/expenses, payments, and reconciliation |
| Research and clinical studies | Study enrollment, consent, protocol-specific data, research encounters, and disclosure restrictions |
| Records management | Clinical documents, document types, signatures, amendments, retention class, release-of-information requests, and disclosure history |
| Workforce and resource scheduling | Staff identity and roles, shifts, clinic/resource availability, assignments, and coverage |
| Facility operations | Facility/service configuration, operating hours, service availability, and effective-dated organizational relationships |
| Identity and security administration | User accounts, roles, permission keys, menu/function access, facility scope, credential status, and access-review history |
| Quality, safety, and risk management | Quality measures, incident reports, safety events, review status, corrective actions, and outcomes |
| Procurement and financial operations | Vendors, purchase requests/orders, receipts, obligations, accounting classifications, and reconciliation status |
| Communications and task workflow | Alerts, tasks, routing, recipients, delivery/acknowledgment status, escalation, and completion history |

The system may also need shared operational and reference data that is not
owned by a patient, such as terminology/code sets, drug and procedure catalogs,
provider and facility directories, application parameters, workflow states,
notification/message status, and interface/import/export job history. For
system-to-system exchanges, this may include message identifiers, routing,
acknowledgments, mapping versions, retry state, and error outcomes. Credentials
and cryptographic secrets must not be stored as ordinary operational data.
Model these records according to their own ownership and lifecycle rather than
attaching them to a patient.

Every installed package and every site customization in the target installation
is in scope; the examples in Sections 4.3 and 4.4 do not limit coverage. For
each, define relevant actors and workflow states, required versus optional
fields, event history, retention rules, and relationships to patients,
encounters, orders, facilities, staff, and other packages. This includes
infrastructure packages and shared services (for example, identity, menu and
option access, messaging, task management, configuration, and FileMan
metadata), not only clinical packages.

### 4.5 Target VistA domain inventory

The project shall use the following pinned VistA source target:

| Target | Pinned source | Coverage treatment |
| --- | --- | --- |
| OSEHRA/WorldVistA | [`WorldVistA/VistA` commit `3cacca516ae3e0c1210aaa00b1c8fa23b87cfde6`](https://github.com/WorldVistA/VistA/commit/3cacca516ae3e0c1210aaa00b1c8fa23b87cfde6), dated 2026-09-23; commit message: “Update Packages.csv for August 2026 patches” | Sole compatibility target; inventory and test this exact source snapshot |

This is a source/patch-tree pin, not a claim that the repository alone is a
complete runnable VistA installation. Record the runnable test environment
separately, including installed packages and patches, underlying M
implementation/version, configuration, and any local customizations. VA FOIA
VistA may be used as upstream/reference material, but it is not a second
conformance target unless the scope is explicitly revised.

Maintain a version-controlled inventory for the pinned source and exact
runnable test installation. Record the repository revision, installed package
and patch levels, underlying M implementation/version, configuration, and
local customizations used for testing. Inventory every installed package and
version, including infrastructure and shared-service packages. For every
package, inventory its FileMan files/subfiles and fields, data dictionaries,
field types, pointers, multiple-valued fields, cross-references, triggers,
security rules, workflows, and package-defined side effects where relevant.

The runnable-installation manifest shall make the test target reproducible and
auditable. Record at minimum: source repository and commit; installation or
container image reference and immutable digest; operating system and
architecture; M implementation, version, and relevant configuration; FileMan
version; installed package/build and patch identifiers; initialization and
post-install configuration steps; enabled options, RPC contexts, and other
integration settings; relevant site parameters and custom routines; and the
hash/version of synthetic data fixtures. Record test accounts and their
assigned roles without recording passwords, tokens, or other secrets. Store
environment-specific secrets outside the inventory and load them through
deployment secret management. Regenerate and review the manifest whenever the
source, image, installed packages, patches, configuration, or fixtures change.
The manifest shall include a value or an explicit `unknown`, `not applicable`,
or `not yet provisioned` status for each required item; an empty or missing
value shall not be interpreted as verified absence. Until an installation is
provisioned and its manifest completed, target-specific integration and
conformance status shall remain `not yet tested`.

Inventory every operation supported by the target package, not only read/write
RPCs. This includes callable RPCs and other APIs, commands, queries, workflow
actions, batch or scheduled operations, and documented interface operations.
For each operation, record its package and version, operation identifier,
purpose, inputs and outputs, validation, side effects, read/write behavior,
authorization requirements, errors, transaction/concurrency behavior, API
mapping, source of truth, and conformance test. Clearly distinguish a package
operation that exists in VistA from an API operation that RUMPS exposes.

For every FileMan field and other package data element in the inventory, state
whether it is exposed, transformed, derived, or excluded. Maintain an explicit
exclusion register for internal, implementation-only, computed, or otherwise
unexposed fields. Each exclusion entry shall identify the package, file or
operation, field/element, classification, reason for exclusion, whether the
value is reproducible from exposed data, and the impact on clients or
round-trip fidelity. Example reasons may include security-sensitive internals,
transient implementation state, unsupported or unstable values, or redundant
computed values; provide a specific justification for each field rather than
relying on a blanket category.

Computed values that clients need shall be exposed with their derivation,
source fields, and relevant calculation/version semantics documented. An
excluded field shall not be silently dropped from API results or writes; where
its omission affects completeness, write round trips, or a package workflow,
document the limitation and account for it in the coverage status.

Every VistA-to-API and VistA-to-FHIR mapping shall be maintained as a
version-controlled mapping artifact. For each mapping, record the source
package/version and file/field or operation; the target API resource,
operation, element, profile, or extension; transformation and terminology
rules; cardinality and value-set handling; null/unknown behavior; whether the
mapping is lossless or lossy and how round trips behave; applicable security
and sensitivity rules; and linked conformance tests.

Mappings shall have explicit lifecycle status: `draft`, `in review`,
`approved`, `rejected`, or `superseded`. A mapping may not be used in a
production API contract or counted toward implemented coverage until approved.
Approval shall be recorded with approver identity, role, date, mapping version,
and rationale. At minimum, approval requires review by the API/data-model
owner, the responsible VistA package/domain owner, and the security/privacy
owner when the mapping affects access, sensitive data, or disclosure rules.
FHIR profiles and extensions shall additionally receive interoperability
review. Material changes to source fields, transformations, terminology
bindings, access behavior, or lossiness require a new mapping version and
approval; retain superseded versions for audit and reproducibility.

Each mapping artifact shall contain, at minimum:

| Mapping record field | Required content |
| --- | --- |
| Mapping identity | Stable mapping ID, version, status, creation date, and change history |
| Source definition | Target snapshot, package/version, FileMan file/subfile/field or operation, and source data type/cardinality |
| Target definition | API operation/resource and FHIR R4 resource/element/profile/extension, or the approved non-FHIR endpoint |
| Transformation | Mapping logic, terminology/code-system bindings and versions, normalization, and null/unknown handling |
| Fidelity and behavior | Lossless/lossy classification, round-trip behavior, computed-value derivation, and known limitations |
| Security and governance | Data sensitivity, authorization policy, required reviewer roles, approval decision/date, and rationale |
| Verification | Conformance test IDs, expected outcomes, latest result, and target-environment identity |

Approval is a recorded governance decision, not an inferred status from
implementation or passing tests. If required reviewers or evidence are
missing, the mapping remains `in review` and shall not be used as an approved
production mapping.

The API data model shall preserve native VistA identifiers and file/subfile
identity, pointers between records, multiple-valued fields, and distinctions
between stored and computed values where clients need them. It shall expose
validated, documented API representations rather than raw MUMPS globals or
uninterpreted FileMan storage encodings. Use supported VistA interfaces for
reads and writes; direct global access or updates are prohibited unless the
target package explicitly documents them as supported.

Assign every inventory entry a status: `not yet tested`, `implemented`,
`partial`, `unavailable in target`, or `deferred by explicit approval`.
`Not yet tested` applies until verified against the runnable target;
`unavailable in target` is valid only when verified against the pinned
installation. `Deferred by explicit approval` is a visible coverage gap and
does not count as complete coverage. A release shall not claim full target
coverage while any in-scope entry remains not yet tested, partial, deferred,
or unmapped. Assess newly discovered packages and local customizations for API
impact.

The API shall not silently omit a target-system field or operation. If an
operation is not exposed, its inventory entry and API documentation shall state
the reason and status. Every excluded field shall have a corresponding
justification in the exclusion register. Document read/write behavior,
validation, side effects, and transactional guarantees for each operation.

For these domains, requirements should define whether history is retained when
records are corrected, cancelled, discontinued, or superseded; how local codes
map to standard terminologies; and whether the system stores the source record
or only a reference to it. Dates and times should preserve enough information
to distinguish the event time from the entry time and, where applicable, the
time zone.

The exact fields, identifiers, code sets, and relationships must be selected
against the target VistA product/version and its installed package data
dictionaries. Avoid treating file numbers or field layouts as universal
contracts across MUMPS applications or VistA installations.

The inventory shall be maintained in a machine-readable format and used to
generate a release coverage report. For each package, operation, field, and
site customization, the report shall include its stable inventory identifier,
API mapping, authorization rule, implementation status, exclusion rationale
where applicable, and conformance-test result or approved exception. The
report shall be reviewed for completeness and included with every release;
manual summaries alone do not satisfy this requirement.

## 5. Functional requirements

The API shall:

1. Expose every in-scope resource and operation in the Section 4.5 target
   inventory using documented, stable contracts.
2. Preserve target identifiers, relationships, cardinality, validation,
   workflow states, and meaningful history.
3. Support the read/write capabilities required by each target package and
   document operations that are read-only or cannot safely be represented.
4. Apply authentication, authorization, facility and patient scope, and
   sensitive-record restrictions before returning data or performing actions.
5. Return structured errors that distinguish invalid requests,
   authentication/authorization failures, not-found conditions, conflicts,
   unavailable dependencies, and internal failures without leaking sensitive
   details.
6. Support pagination, filtering, and deterministic ordering for large
   collections; define request-size limits and rate controls.
7. Protect write operations against accidental duplicate execution and
   concurrent updates using idempotency or equivalent controls where
   appropriate.
8. Preserve source identifiers and provenance for imported or externally
   referenced records, including corrections and supersession history where
   required by the source system.
9. Identify the applicable terminology/code system and version for coded
   diagnoses, procedures, drugs, and other coded values.
10. Keep shared reference and operational data independently identifiable and
    versioned where applicable, recording the source/version used by dependent
    clinical records.
11. Preserve workflow status/event history sufficient to explain who performed
    an action, when it occurred, and whether it succeeded, failed, was
    cancelled, or was superseded.
12. Provide API versioning and a documented compatibility/deprecation policy;
    breaking changes shall be explicit.
13. Emit security and operational audit events for API access and mutations,
    while excluding credentials, secrets, and unnecessary sensitive health
    information from logs.
14. Implement HTTP/JSON endpoints and FHIR R4 (4.0.1) representations for all
    applicable API resources and operations. Publish a FHIR CapabilityStatement
    that declares supported resources, interactions, search parameters,
    profiles, formats, and security expectations.
15. Maintain a versioned mapping from every in-scope VistA package resource,
    field, and supported operation to its FHIR R4 resource, element, profile,
    extension, or documented non-FHIR API operation. Do not discard VistA
    semantics that FHIR core resources cannot represent; define and document
    governed profiles/extensions or explicit companion operations.
16. Validate FHIR requests and responses against the declared R4 specification
    and project profiles. Return FHIR OperationOutcome responses for FHIR
    request errors and preserve the API's documented error semantics.
17. Document FHIR search, paging, conditional interactions, versioning,
    concurrency, and transaction/batch behavior for each supported resource;
    do not advertise interactions that are not implemented.
18. Provide a reproducible runnable test environment for the pinned
    OSEHRA/WorldVistA source snapshot. Record and version the image/build
    identity, M implementation and version, installed package and patch list,
    configuration, startup procedure, and test data fixture version. Test data
    shall be synthetic and shall not contain real patient information.
19. Authenticate every API client using OIDC/OAuth2 bearer tokens and deny
    access by default. The current model permits allowlisted service principals
    only and does not support delegated end-user/VistA identity. Before
    exposing user-attributed operations or clinical data, implement and
    document the approved client/user-to-VistA identity mapping, privilege
    policy, facility and patient scope, sensitive-record restrictions, and
    corresponding audit attribution.
20. Define and test measurable service-level objectives before production
    release, including availability, response latency by operation class,
    throughput/concurrency, maximum request and response sizes, and dependency
    failure behavior. Define monitoring and alert thresholds, backup and
    restore expectations, recovery time objective (RTO), and recovery point
    objective (RPO). Do not claim an SLO without a measurement window and
    verification method.
21. Define API semantics for missing, null, empty, unknown, stale, and
    unavailable values; partial results; event and entry timestamps and time
    zones; and ordering/pagination stability. Document write retry and
    idempotency behavior, duplicate detection, optimistic concurrency/conflict
    handling, and which operations are atomic.
22. Generate and publish the machine-readable coverage report described in
    Section 4.5 for every release. The release shall fail its coverage gate if
    any in-scope inventory item is unmapped, lacks an authorization decision,
    lacks a required conformance test/result, or has an unapproved exception.

The current prototype does not satisfy these API requirements.

### 5.1 Security verification

Security behavior shall be validated with automated positive and negative
authorization tests for every API operation and resource class. Tests shall
include unauthenticated requests, invalid or expired credentials, role and
facility restrictions, patient-level restrictions, sensitive-record access,
and attempts to bypass controls through alternate identifiers, search,
pagination, batch requests, or indirect resource references.

OIDC verification shall pin the configured issuer and audience; require
expiration and subject claims; verify signatures only with asymmetric keys
from the configured JWKS; and reject clients not in the explicit service-client
allowlist. Production shall fail startup if required OIDC configuration is
missing or the JWKS cannot be loaded. Public operational/discovery endpoints
shall be explicitly enumerated; all other routes shall fail closed.

Security and access audits shall identify the API client, acting user when
applicable, action, target resource, outcome, and timestamp. Audit records
shall be access-controlled, tamper-evident, retained according to an explicit
policy, and must not contain credentials or unnecessary patient data.

### 5.2 Service-level objectives and operations

Define SLO values and measurement procedures for the production deployment
before release. Measure service availability and latency at the API boundary,
and separately track VistA dependency latency and errors so that dependency
failures are distinguishable from API failures. Define behavior during
maintenance, overload, dependency outage, and recovery, including whether
requests fail fast, may be retried, or can produce partial results.

Document operational ownership, health/readiness checks, metrics, structured
logs, alerting, incident response, backup scope, restore testing, and RTO/RPO.
Operational telemetry and logs shall be minimized to avoid unnecessary
disclosure of protected health information.

### 5.3 API value, time, and write semantics

For each API field, document whether omission means not requested, not known,
not applicable, inaccessible, or absent in the source. Distinguish null from
empty collections and empty strings; do not use any of these to conceal
authorization failures, source unavailability, or incomplete results. Define
how unknown or unsupported source values are represented without lossy
coercion.

Represent dates and times according to the FHIR R4 data type and the source
value's semantics. Preserve source precision and distinguish clinical event
time from record-entry or modification time. Preserve an offset or time-zone
identifier when available; do not infer a time zone or fabricate precision
that the source does not provide.

For writes, document idempotency keys or equivalent duplicate protection,
retry safety, concurrency/version checks, conflict responses, transaction
boundaries, and partial-success behavior. A client retry shall not silently
duplicate a non-idempotent clinical or administrative action.

### 5.4 Coverage reporting

The generated release coverage report shall be reproducible from the
machine-readable inventory and test results. It shall summarize coverage by
package and operation, list excluded fields with reasons, show API and
authorization mappings, identify passing, failing, missing, and excepted tests,
and identify the exact target snapshot and test-environment build. Publish the
report as a release artifact; do not represent an untested or excepted item as
implemented.

## 6. Persistence and data integrity

SQLite is the development persistence database. PostgreSQL is the required
production persistence database. The API shall use the same persistence
abstraction and domain behavior in both environments; SQLite is not permitted
for production deployments.

The service shall integrate with VistA through supported interfaces and
persist API data in the environment's configured database, including
synchronized or cached VistA data where needed. For every resource and field,
document whether VistA or the database is authoritative, whether the value is
synchronized or API-owned, and how consistency, synchronization, failure, and
recovery are handled. Neither development SQLite nor production PostgreSQL
shall silently become authoritative for a VistA-owned value merely because it
is cached locally.

Both database backends shall preserve the same externally observable API
semantics. Define a portable schema and query behavior, and avoid relying on
SQLite-specific type coercions or SQL features that would cause differences in
production. Test migrations, constraints, transaction boundaries, and
round-trip behavior against both SQLite and the supported PostgreSQL
production version. The production PostgreSQL persistence layer shall:

- Preserve native identifiers, record relationships, pointers, subfiles, and
  multiplicity for every domain, not only patient records.
- Maintain a clear distinction between authoritative source data, cached or
  synchronized copies, and API-owned data.
- Define consistency guarantees and behavior for stale data, partial outages,
  retries, conflicts, and recovery after synchronization failures.
- Use parameterized database operations and transactions for atomic
  multi-record changes in both environments.
- Manage schema evolution through versioned migrations for both backends.
  Migrations shall be ordered, safely guarded, tested against SQLite and
  supported PostgreSQL versions, and have a documented rollback or
  forward-recovery procedure.
- Enforce relational integrity with appropriate keys, constraints, and
  indexes while preserving native VistA identifiers and relationship
  semantics.
- Define environment-appropriate connection security. Production PostgreSQL
  shall use least-privilege database roles, encrypted transport, secure secret
  handling, pool sizing, and documented behavior when the database is
  unavailable. Development SQLite files shall have controlled filesystem
  permissions and shall not contain production or real patient data.
- Back up and restore PostgreSQL data according to the service RPO and RTO;
  perform and record restore tests.
- Preserve database errors for the caller to handle.
- Load all fields and related records required by the API contract; report
  unavailable or incomplete results explicitly rather than returning them as
  complete.

The current prototype has incomplete adapter code using SQLite and does not
yet meet the development-SQLite/production-PostgreSQL requirement. Its patient
adapter stores only a subset of patient fields, converts some structured data
into single strings, and initializes most related categories as empty on
retrieval. These adapters are not the persistence contract and shall be
completed or replaced as part of implementing both database backends.

## 7. MUMPS relationship

The README lists VistA/FileMan file numbers as domain references, including Patient
(#2), Person (#16), Patient Movement (#45), Drug (#50), Allergy/Adverse
Reaction (#120.8), Patient Movement (#405), and Patient Care Encounter
(#409.68). File numbers alone do not define complete API behavior: package
versions, data dictionaries, cross-references, callable operations, security
rules, and local customizations also determine the target contract.

## 8. Current implementation status

- The project contains Rust domain structures for patients and their related
  records, plus order, order-action, and order-activity structures.
- The executable runs an Axum API server with liveness/readiness, FHIR
  CapabilityStatement, and OpenAPI routes.
- SQLite is used for development and PostgreSQL for production. Startup creates
  only the migration tracking table; complete persistence migrations and
  domain persistence contracts remain unimplemented.
- OIDC JWT signature, issuer, audience, expiration, subject, and service-client
  allowlist validation are implemented for non-public routes. Production
  requires OIDC configuration and a loadable JWKS. Authentication does not
  provide VistA user delegation or clinical/resource authorization.
- The API has no clinical resource routes, and the CapabilityStatement
  advertises no clinical interactions. Target mappings, authorization policy,
  audit logging, and conformance testing remain unimplemented.

The domain structures and legacy adapters are not yet connected to the API
contract or persistence layer.

## 9. Acceptance criteria for a complete implementation

- An API client can invoke each operation using documented request/response
  schemas, authentication, authorization, and structured errors.
- The pinned OSEHRA/WorldVistA source commit and the runnable test
  configuration, including installed package/patch levels and underlying M
  implementation version, are recorded before conformance testing.
- The runnable test environment is reproducible from versioned build inputs,
  uses synthetic test data, and has a recorded environment/build identity.
- The target installation manifest records the source commit, immutable
  environment image digest, operating system/architecture, M and FileMan
  versions, installed packages and patches, configuration, test-account roles,
  and synthetic fixture version without containing secrets; unknown and
  not-yet-provisioned values are explicit.
- HTTP/JSON is the primary API interface, and FHIR R4 (4.0.1) is the declared
  interoperability baseline.
- A published FHIR CapabilityStatement and versioned VistA-to-FHIR mapping
  cover all applicable in-scope resources, fields, and operations, including
  documented profiles/extensions or non-FHIR operations for semantics not
  represented by FHIR core.
- FHIR interactions, search parameters, profiles, and error responses pass
  automated validation against the declared R4 specification and project
  profiles.
- Every installed package and version, package-supported operation, workflow,
  security rule, FileMan file/subfile, field, and site customization is
  accounted for in the inventory.
- Every package-supported operation is mapped to an API operation or explicitly
  documented as not exposed, with a reason, status, authorization requirements,
  side effects, and conformance test coverage.
- Every VistA-to-API and VistA-to-FHIR mapping used by the implementation has
  an approved, version-controlled mapping artifact describing transformations,
  value semantics, security, round-trip behavior, and conformance tests.
- Mapping approvals record required domain/API/security reviews and
  interoperability review for FHIR profiles/extensions; changed mappings are
  versioned, re-reviewed, and traceable to superseded versions.
- Every internal, computed, transformed, or excluded data element has an
  inventory disposition. Each excluded field is listed with its specific
  exclusion reason and impact; client-relevant computed values document their
  derivation and version semantics.
- Every in-scope inventory entry is implemented and passes mapped conformance
  tests. Entries verified as unavailable in the pinned target are documented;
  deferred, partial, and unmapped entries prevent a full-coverage claim.
- SQLite is used for development and PostgreSQL for production; both backends
  pass the same persistence contract, migration, constraint, transaction, and
  round-trip tests.
- Supported PostgreSQL server versions and required extensions are documented;
  migrations and automated tests pass against each supported version.
- Production PostgreSQL connection security, backup/restore, and data integrity
  requirements pass automated or operational verification.
- Every API operation has positive and negative authorization tests covering
  relevant client/user identities, facility and patient scope, and
  sensitive-record restrictions.
- Availability, operation-class latency, throughput/concurrency, request-size,
  dependency-failure, RTO, and RPO objectives have defined values, measurement
  windows, verification methods, and passing results for the target deployment.
- API documentation and tests define missing/null/empty/unknown/stale/
  unavailable value semantics, timestamp precision/time-zone behavior,
  partial-result behavior, idempotency, retry safety, concurrency, and
  transaction boundaries.
- A machine-readable inventory produces a release coverage report identifying
  target snapshot, API and authorization mappings, excluded fields and
  rationales, and conformance results; the release coverage gate passes.
- API reads and writes preserve target data, relationships, and workflows
  without loss or unapproved side effects.
- Not-found, validation, authorization, conflict, dependency, and internal
  failures are distinguishable without disclosing sensitive data.
- A round trip through persistence preserves the supported domain data rather
  than replacing unloaded or unsupported data with success-shaped empty
  collections.
- The project builds with its declared dependencies and its persistence and
  domain behavior are covered by automated tests.
- Every API mutation and sensitive-data access is authorized and produces the
  required audit event.
- Pagination, filtering, request limits, concurrency behavior, and API version
  compatibility are documented and tested.

## 10. References for VistA alignment

Use the VA Software Document Library's [FileMan documentation](https://www.va.gov/vdl/application.asp?appid=5),
including the [FileMan Technical Manual](https://www.va.gov/vdl/documents/Infrastructure/Fileman/fm22_2tm.pdf)
and [FileMan Developer's Guide](https://www.va.gov/vdl/documents/Infrastructure/Fileman/fm22_2dg.pdf),
to inspect the data dictionary, field definitions, pointers, and relationships
for the actual target installation. Also inspect documentation and interfaces
for each installed package, including callable operations and security
requirements. These references are a starting point, not a complete catalog
of every package or local customization; the Section 4.5 inventory is the
acceptance baseline for the target.

For the interoperability baseline, use the official [HL7 FHIR R4 (4.0.1)
specification](https://hl7.org/fhir/R4/), including its RESTful API,
resource, search, and conformance requirements.
