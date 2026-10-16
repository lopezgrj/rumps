use std::{env, net::SocketAddr, num::NonZeroU32, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseKind {
    Sqlite,
    Postgres,
}

impl std::fmt::Display for DatabaseKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite => formatter.write_str("sqlite"),
            Self::Postgres => formatter.write_str("postgres"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub kind: DatabaseKind,
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OidcConfig {
    pub issuer: String,
    pub audience: String,
    pub jwks_url: String,
    pub allowed_client_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
    pub database: DatabaseConfig,
    pub oidc: Option<OidcConfig>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    InvalidEnvironment(String),
    MissingProductionDatabaseUrl,
    InvalidDatabaseUrl,
    InvalidBindAddress(String),
    InvalidPoolSize(String),
    MissingProductionOidcConfig,
    IncompleteOidcConfig,
    InvalidOidcConfig,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidEnvironment(value) => {
                write!(formatter, "unsupported RUMPS_ENV value: {value}")
            }
            Self::MissingProductionDatabaseUrl => {
                formatter.write_str("DATABASE_URL is required in production")
            }
            Self::InvalidDatabaseUrl => formatter.write_str(
                "database URL scheme is invalid for this environment (expected SQLite in development or PostgreSQL in production)",
            ),
            Self::InvalidBindAddress(value) => {
                write!(formatter, "invalid RUMPS_BIND_ADDR: {value}")
            }
            Self::InvalidPoolSize(value) => {
                write!(formatter, "invalid RUMPS_DB_MAX_CONNECTIONS: {value}")
            }
            Self::MissingProductionOidcConfig => formatter.write_str(
                "OIDC_ISSUER, OIDC_AUDIENCE, OIDC_JWKS_URL, and OIDC_ALLOWED_CLIENT_IDS are required in production",
            ),
            Self::IncompleteOidcConfig => formatter.write_str(
                "OIDC_ISSUER, OIDC_AUDIENCE, OIDC_JWKS_URL, and OIDC_ALLOWED_CLIENT_IDS must be configured together",
            ),
            Self::InvalidOidcConfig => formatter.write_str(
                "OIDC configuration must contain HTTPS issuer/JWKS URLs, an audience, and at least one allowed service client ID",
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = env::var("RUMPS_ENV").unwrap_or_else(|_| "development".to_owned());
        let database_url = env::var("DATABASE_URL").ok();
        let bind_addr = env::var("RUMPS_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());
        let max_connections =
            env::var("RUMPS_DB_MAX_CONNECTIONS").unwrap_or_else(|_| "5".to_owned());
        let oidc = oidc_config_from_env()?;

        Self::from_values(
            &environment,
            database_url.as_deref(),
            &bind_addr,
            &max_connections,
            oidc,
        )
    }

    pub fn from_values(
        environment: &str,
        database_url: Option<&str>,
        bind_addr: &str,
        max_connections: &str,
        oidc: Option<OidcConfig>,
    ) -> Result<Self, ConfigError> {
        let max_connections = max_connections
            .parse::<NonZeroU32>()
            .map_err(|_| ConfigError::InvalidPoolSize(max_connections.to_owned()))?
            .get();
        let bind_addr = SocketAddr::from_str(bind_addr)
            .map_err(|_| ConfigError::InvalidBindAddress(bind_addr.to_owned()))?;

        let (kind, url) = match environment {
            "development" | "test" => {
                let url = database_url.unwrap_or("sqlite://rumps-dev.db?mode=rwc");
                if !url.starts_with("sqlite:") {
                    return Err(ConfigError::InvalidDatabaseUrl);
                }
                if let Some(oidc) = &oidc {
                    validate_oidc_config(oidc, false)?;
                }
                (DatabaseKind::Sqlite, url.to_owned())
            }
            "production" => {
                let url = database_url.ok_or(ConfigError::MissingProductionDatabaseUrl)?;
                if !(url.starts_with("postgres://") || url.starts_with("postgresql://")) {
                    return Err(ConfigError::InvalidDatabaseUrl);
                }
                let oidc = oidc
                    .as_ref()
                    .ok_or(ConfigError::MissingProductionOidcConfig)?;
                validate_oidc_config(oidc, true)?;
                (DatabaseKind::Postgres, url.to_owned())
            }
            other => return Err(ConfigError::InvalidEnvironment(other.to_owned())),
        };

        Ok(Self {
            bind_addr,
            oidc,
            database: DatabaseConfig {
                kind,
                url,
                max_connections,
            },
        })
    }
}

fn oidc_config_from_env() -> Result<Option<OidcConfig>, ConfigError> {
    let issuer = env::var("OIDC_ISSUER").ok();
    let audience = env::var("OIDC_AUDIENCE").ok();
    let jwks_url = env::var("OIDC_JWKS_URL").ok();
    let allowed_client_ids = env::var("OIDC_ALLOWED_CLIENT_IDS").ok();
    let configured = [
        issuer.is_some(),
        audience.is_some(),
        jwks_url.is_some(),
        allowed_client_ids.is_some(),
    ];

    if configured.iter().all(|value| !value) {
        return Ok(None);
    }
    if configured.iter().any(|value| !value) {
        return Err(ConfigError::IncompleteOidcConfig);
    }

    let allowed_client_ids = allowed_client_ids
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|client_id| !client_id.is_empty())
        .map(str::to_owned)
        .collect();

    Ok(Some(OidcConfig {
        issuer: issuer.unwrap_or_default(),
        audience: audience.unwrap_or_default(),
        jwks_url: jwks_url.unwrap_or_default(),
        allowed_client_ids,
    }))
}

fn validate_oidc_config(config: &OidcConfig, production: bool) -> Result<(), ConfigError> {
    if config.issuer.trim().is_empty()
        || config.audience.trim().is_empty()
        || !is_secure_oidc_url(&config.issuer, production)
        || !is_secure_oidc_url(&config.jwks_url, production)
        || config.allowed_client_ids.is_empty()
        || config
            .allowed_client_ids
            .iter()
            .any(|client_id| client_id.trim().is_empty())
    {
        return Err(ConfigError::InvalidOidcConfig);
    }

    Ok(())
}

fn is_secure_oidc_url(value: &str, production: bool) -> bool {
    let Ok(url) = reqwest::Url::parse(value) else {
        return false;
    };
    let loopback_http = !production
        && url.scheme() == "http"
        && matches!(
            url.host_str(),
            Some("localhost" | "127.0.0.1" | "[::1]" | "::1")
        );

    (url.scheme() == "https" || loopback_http)
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_defaults_to_sqlite() {
        let config = AppConfig::from_values("development", None, "127.0.0.1:3000", "5", None);
        assert!(matches!(
            config,
            Ok(AppConfig {
                database: DatabaseConfig {
                    kind: DatabaseKind::Sqlite,
                    ..
                },
                ..
            })
        ));
    }

    #[test]
    fn production_rejects_sqlite() {
        let config = AppConfig::from_values(
            "production",
            Some("sqlite://production.db"),
            "127.0.0.1:3000",
            "5",
            None,
        );
        assert_eq!(config.err(), Some(ConfigError::InvalidDatabaseUrl));
    }

    #[test]
    fn production_requires_database_url() {
        let config = AppConfig::from_values("production", None, "127.0.0.1:3000", "5", None);
        assert_eq!(
            config.err(),
            Some(ConfigError::MissingProductionDatabaseUrl)
        );
    }

    #[test]
    fn production_accepts_postgres_url() {
        let config = AppConfig::from_values(
            "production",
            Some("postgres://user:password@localhost/rumps"),
            "127.0.0.1:3000",
            "5",
            Some(test_oidc_config()),
        );
        assert!(matches!(
            config,
            Ok(AppConfig {
                database: DatabaseConfig {
                    kind: DatabaseKind::Postgres,
                    ..
                },
                ..
            })
        ));
    }

    #[test]
    fn invalid_bind_address_is_rejected() {
        let config = AppConfig::from_values("development", None, "not-an-address", "5", None);
        assert_eq!(
            config.err(),
            Some(ConfigError::InvalidBindAddress("not-an-address".to_owned()))
        );
    }

    #[test]
    fn production_requires_oidc_configuration() {
        let config = AppConfig::from_values(
            "production",
            Some("postgres://user:pass@localhost/rumps"),
            "127.0.0.1:3000",
            "5",
            None,
        );
        assert_eq!(config.err(), Some(ConfigError::MissingProductionOidcConfig));
    }

    fn test_oidc_config() -> OidcConfig {
        OidcConfig {
            issuer: "https://identity.example.test/".to_owned(),
            audience: "rumps-api".to_owned(),
            jwks_url: "https://identity.example.test/jwks".to_owned(),
            allowed_client_ids: vec!["rumps-service".to_owned()],
        }
    }
}
