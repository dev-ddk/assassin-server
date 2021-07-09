#![allow(dead_code)]

#[macro_use]
use lazy_static;

use jsonwebtoken::{dangerous_insecure_decode, decode, Algorithm, DecodingKey, Validation};
use openssl::x509;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snailquote;
use std::collections::HashMap;

lazy_static::lazy_static! {
    pub static ref VALIDATOR: Validator = Validator::new();
}

#[derive(Debug, Serialize, Deserialize)]
struct Identity {
    email: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Firebase {
    identities: Identity,
    sign_in_provider: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserClaims {
    iss: String,
    aud: String,
    auth_time: usize,
    user_id: String,
    sub: String,
    iat: usize,
    exp: usize,
    email: String,
    email_verified: bool,
    firebase: Firebase,
}

#[derive(Debug, Deserialize)]
pub struct IdToken {
    pub id_token: String,
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

    pub fn validate_token(&self, id_token: &str) -> Option<UserClaims> {
        // Here we need to decode the token to get the key with which to re-decode, this time with verification

        let key_fingerprint = dangerous_insecure_decode::<UserClaims>(id_token)
            .unwrap()
            .header
            .kid
            .unwrap();

        let token = decode::<UserClaims>(
            id_token,
            self.validation_keys.get(&key_fingerprint).unwrap(),
            &self.validation,
        );
        match token {
            Ok(token) => Some(token.claims),
            Err(e) => None,
        }
    }
}

fn get_validator() -> Validation {
    let mut validator = Validation {
        iss: Some("https://securetoken.google.com/assassin-8c704".to_string()),
        algorithms: vec![Algorithm::RS256],
        ..Validation::default()
    };

    validator.set_audience(&["assassin-8c704"]);
    validator
}

fn get_validation_keys() -> HashMap<String, DecodingKey<'static>> {
    let body = reqwest::blocking::get(
        "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com",
    )
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
    if let Ok(cert) = x509::X509::from_pem(snailquote::unescape(&certificate).unwrap().as_bytes()) {
        let key = cert.public_key().unwrap().public_key_to_pem().unwrap();
        DecodingKey::from_rsa_pem(&key).unwrap().into_static()
    } else {
        panic!("Could not decode RSA Verification key from Google");
    }
}
