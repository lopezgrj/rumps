use std::{collections::HashSet, error::Error, time::Duration};

use jsonwebtoken::{
    Algorithm, DecodingKey, TokenData, Validation, decode, decode_header,
    jwk::JwkSet,
    jwk::{AlgorithmParameters, KeyAlgorithm, KeyOperations, PublicKeyUse},
};
use serde::Deserialize;

use crate::application::OidcConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    pub client_id: String,
    pub subject: String,
}

pub struct OidcAuthenticator {
    issuer: String,
    audience: String,
    allowed_client_ids: HashSet<String>,
    jwks: JwkSet,
}

#[derive(Debug, Deserialize)]
struct ServiceClaims {
    sub: String,
    client_id: Option<String>,
    azp: Option<String>,
}

impl OidcAuthenticator {
    pub async fn load(config: &OidcConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        let jwks = client
            .get(&config.jwks_url)
            .send()
            .await?
            .error_for_status()?
            .json::<JwkSet>()
            .await?;

        if jwks.keys.is_empty() {
            return Err("OIDC JWKS contains no signing keys".into());
        }

        Ok(Self {
            issuer: config.issuer.clone(),
            audience: config.audience.clone(),
            allowed_client_ids: config.allowed_client_ids.iter().cloned().collect(),
            jwks,
        })
    }

    pub fn verify(&self, token: &str) -> Result<Principal, TokenError> {
        let header = decode_header(token).map_err(|_| TokenError)?;
        if !is_supported_algorithm(header.alg) {
            return Err(TokenError);
        }

        let kid = header.kid.as_deref().ok_or(TokenError)?;
        let jwk = self
            .jwks
            .keys
            .iter()
            .find(|key| key.common.key_id.as_deref() == Some(kid))
            .ok_or(TokenError)?;
        if !jwk_supports_algorithm(jwk, header.alg) {
            return Err(TokenError);
        }
        let decoding_key = DecodingKey::from_jwk(jwk).map_err(|_| TokenError)?;

        let claims: TokenData<ServiceClaims> =
            decode(token, &decoding_key, &self.validation(header.alg)).map_err(|_| TokenError)?;
        if matches!(
            (&claims.claims.azp, &claims.claims.client_id),
            (Some(azp), Some(client_id)) if azp != client_id
        ) {
            return Err(TokenError);
        }
        let client_id = claims
            .claims
            .azp
            .or(claims.claims.client_id)
            .ok_or(TokenError)?;
        if claims.claims.sub.is_empty() || !self.allowed_client_ids.contains(&client_id) {
            return Err(TokenError);
        }

        Ok(Principal {
            client_id,
            subject: claims.claims.sub,
        })
    }

    fn validation(&self, algorithm: Algorithm) -> Validation {
        let mut validation = Validation::new(algorithm);
        validation.set_issuer(&[self.issuer.as_str()]);
        validation.set_audience(&[self.audience.as_str()]);
        validation.required_spec_claims = ["aud", "exp", "iss", "sub"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        validation.validate_nbf = true;
        validation
    }

    #[cfg(test)]
    pub(crate) fn empty_test_instance() -> Self {
        Self {
            issuer: "https://identity.example.test/".to_owned(),
            audience: "rumps-api".to_owned(),
            allowed_client_ids: HashSet::from(["rumps-service".to_owned()]),
            jwks: JwkSet { keys: Vec::new() },
        }
    }
}

fn is_supported_algorithm(algorithm: Algorithm) -> bool {
    matches!(
        algorithm,
        Algorithm::RS256
            | Algorithm::RS384
            | Algorithm::RS512
            | Algorithm::ES256
            | Algorithm::ES384
            | Algorithm::EdDSA
    )
}

fn jwk_supports_algorithm(jwk: &jsonwebtoken::jwk::Jwk, algorithm: Algorithm) -> bool {
    let key_type_matches = matches!(
        (&jwk.algorithm, algorithm),
        (
            AlgorithmParameters::RSA(_),
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512
        ) | (
            AlgorithmParameters::EllipticCurve(_),
            Algorithm::ES256 | Algorithm::ES384
        ) | (AlgorithmParameters::OctetKeyPair(_), Algorithm::EdDSA)
    );
    let declared_algorithm_matches = match jwk.common.key_algorithm {
        None => true,
        Some(KeyAlgorithm::RS256) => algorithm == Algorithm::RS256,
        Some(KeyAlgorithm::RS384) => algorithm == Algorithm::RS384,
        Some(KeyAlgorithm::RS512) => algorithm == Algorithm::RS512,
        Some(KeyAlgorithm::ES256) => algorithm == Algorithm::ES256,
        Some(KeyAlgorithm::ES384) => algorithm == Algorithm::ES384,
        Some(KeyAlgorithm::EdDSA) => algorithm == Algorithm::EdDSA,
        Some(_) => false,
    };
    let intended_for_verification = jwk
        .common
        .public_key_use
        .as_ref()
        .is_none_or(|usage| usage == &PublicKeyUse::Signature)
        && jwk
            .common
            .key_operations
            .as_ref()
            .is_none_or(|operations| operations.contains(&KeyOperations::Verify));

    key_type_matches && declared_algorithm_matches && intended_for_verification
}

#[derive(Debug)]
pub struct TokenError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_symmetric_and_unsecured_algorithms() {
        assert!(!is_supported_algorithm(Algorithm::HS256));
        assert!(!is_supported_algorithm(Algorithm::HS384));
        assert!(!is_supported_algorithm(Algorithm::HS512));
    }

    #[test]
    fn accepts_asymmetric_signature_algorithms() {
        assert!(is_supported_algorithm(Algorithm::RS256));
        assert!(is_supported_algorithm(Algorithm::ES256));
        assert!(is_supported_algorithm(Algorithm::EdDSA));
    }
}
