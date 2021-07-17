use lazy_static;

use actix_web::{dev::ServiceRequest, Error};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use eyre::{eyre, Result};
use jsonwebtoken::{dangerous_insecure_decode, decode, Algorithm, DecodingKey, Validation};
use openssl::x509::X509;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snailquote;
use std::collections::HashMap;

lazy_static::lazy_static! {
    pub static ref VALIDATOR: Validator = Validator::new();
}

const ISSUER: &str = "https://securetoken.google.com/assassin-8c704";
const GOOGLE_PK_URL: &str =
    "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct Identity {
    pub email: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Firebase {
    pub identities: Identity,
    pub sign_in_provider: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserClaims {
    pub iss: String,
    pub aud: String,
    pub auth_time: usize,
    pub user_id: String,
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
    pub email: String,
    pub email_verified: bool,
    pub firebase: Firebase,
}

#[derive(Debug, Clone)]
pub struct Validator {
    validation: Validation,
    validation_keys: HashMap<String, DecodingKey<'static>>,
}

impl Validator {
    pub fn new() -> Validator {
        let validation = get_validator();
        let validation_keys = get_validation_keys();
        Validator {
            validation,
            validation_keys,
        }
    }

    pub fn validate_token(&self, id_token: &str) -> Result<UserClaims> {
        // Here we need to decode the token to get the key with which to re-decode, this time with verification

        let kid = dangerous_insecure_decode::<UserClaims>(id_token)?
            .header
            .kid
            .ok_or(eyre!("Could not extract KID from given JWT"))?;

        let validation_key = self.validation_keys.get(&kid).ok_or(eyre!(
            "Key used for signature is not present in Google's Public Key Repository"
        ))?;

        decode::<UserClaims>(id_token, validation_key, &self.validation)
            .map(|tok| tok.claims)
            .map_err(|e| eyre::Report::new(e).wrap_err("Token validation failed"))
    }
}

pub async fn bearer_auth_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    let config = req
        .app_data::<Config>()
        .map(|data| data.clone())
        .unwrap_or_else(Default::default);

    let claims = VALIDATOR.validate_token(credentials.token());

    match claims {
        Ok(claims) => {
            let (http_request, payload) = req.into_parts();
            http_request.extensions_mut().insert(claims);
            let req_result = ServiceRequest::from_parts(http_request, payload);
            match req_result {
                Ok(req) => Ok(req),
                Err(_) => panic!("Could not reconstruct from parts"),
            }
        }
        Err(_) => Err(AuthenticationError::from(config).into()),
    }
}

fn get_validator() -> Validation {
    let mut validator = Validation {
        iss: Some(ISSUER.to_string()),
        algorithms: vec![Algorithm::RS256],
        ..Validation::default()
    };

    validator.set_audience(&["assassin-8c704"]);
    validator
}

fn get_validation_keys() -> HashMap<String, DecodingKey<'static>> {
    let body = reqwest::blocking::get(GOOGLE_PK_URL)
        .unwrap()
        .text()
        .unwrap();

    let keys: Value = serde_json::from_str(&body).unwrap();

    let decoding_keys: HashMap<String, DecodingKey<'static>> = keys
        .as_object()
        .unwrap()
        .iter()
        .map(|(fingerprint, cert)| {
            (
                fingerprint.clone(),
                get_key_from_certificate(cert.to_string()),
            )
        })
        .collect();

    decoding_keys
}

fn get_key_from_certificate(certificate: String) -> DecodingKey<'static> {
    if let Ok(cert) = X509::from_pem(snailquote::unescape(&certificate).unwrap().as_bytes()) {
        let key = cert.public_key().unwrap().public_key_to_pem().unwrap();
        DecodingKey::from_rsa_pem(&key).unwrap().into_static()
    } else {
        panic!("Could not decode RSA Verification key from Google");
    }
}
