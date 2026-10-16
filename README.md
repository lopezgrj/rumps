# RUMPS

RUMPS is an API-only Rust service intended to provide HTTP/JSON and HL7 FHIR
R4 (4.0.1) access to the pinned OSEHRA/WorldVistA target, with SQLite for
development and PostgreSQL for production.

## Current status

The current implementation is an API foundation, not a complete VistA API. It
provides liveness/readiness routes, a FHIR CapabilityStatement, and
environment-based SQLite/PostgreSQL connection selection. Protected routes
validate OIDC JWT bearer tokens against a configured issuer and JWKS and accept
only allowlisted service clients. The API does not establish a delegated VistA
user identity or implement clinical authorization. The CapabilityStatement
intentionally advertises no clinical resources because VistA package mappings
and conformance testing are not yet implemented. Do not use this prototype
with real patient information.

The target source snapshot and intended coverage requirements are documented
in [SPECIFICATION.md](./SPECIFICATION.md).

## Run in development

Requirements: Rust toolchain and SQLite.

```sh
cargo run
```

Development defaults:

- `RUMPS_ENV=development`
- `DATABASE_URL=sqlite://rumps-dev.db?mode=rwc`
- `RUMPS_BIND_ADDR=127.0.0.1:3000`
- `RUMPS_DB_MAX_CONNECTIONS=5`

Override these with environment variables as needed. Development uses only
SQLite URLs. OIDC settings may be omitted in development; only the public
health, FHIR metadata, and OpenAPI routes remain available in that case. All
other routes fail closed with `503` until OIDC is configured.

Set these variables together to enable protected routes:

- `OIDC_ISSUER` — exact JWT issuer claim.
- `OIDC_AUDIENCE` — required API audience.
- `OIDC_JWKS_URL` — HTTPS JWKS endpoint (HTTP is permitted only for loopback
  development issuers).
- `OIDC_ALLOWED_CLIENT_IDS` — comma-separated allowlist of service client IDs.

The API validates token signatures, issuer, audience, expiration, and subject.
Only asymmetric signing algorithms are accepted. Tokens must identify an
allowlisted client through `azp` or `client_id`; if both are present, they must
match. Configure the allowlist with client-credentials service registrations.
The JWKS is fetched at startup, so restart the service after signing-key
rotation. Health, FHIR metadata, and OpenAPI discovery routes are public;
unknown and future non-public routes require a valid bearer token.

## Run in production

Set `RUMPS_ENV=production`, provide a PostgreSQL `DATABASE_URL`, configure a
production bind address, and provide all four OIDC variables above. Production
requires HTTPS for the issuer and JWKS URLs and refuses to start if OIDC is not
configured or its JWKS cannot be loaded, for example:

```sh
RUMPS_ENV=production \
DATABASE_URL=postgres://user:password@localhost/rumps \
RUMPS_BIND_ADDR=0.0.0.0:3000 \
OIDC_ISSUER=https://identity.example.org/issuer \
OIDC_AUDIENCE=rumps-api \
OIDC_JWKS_URL=https://identity.example.org/.well-known/jwks.json \
OIDC_ALLOWED_CLIENT_IDS=rumps-service \
cargo run --release
```

Supply credentials through deployment secret management rather than committing
them to source control.

## Endpoints currently implemented

- `GET /health/live` — process liveness.
- `GET /health/ready` — database connectivity check.
- `GET /fhir/metadata` — FHIR R4 CapabilityStatement; currently declares no
  clinical resource interactions.

## Validation

```sh
cargo test
cargo fmt --check
```

## License

MIT
