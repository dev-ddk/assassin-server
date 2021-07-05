use serde::{Serialize, Deserialize};
use jsonwebtoken::{decode, Algorithm, Validation, DecodingKey};
use openssl::x509;

#[derive(Debug, Serialize, Deserialize)]
struct Identity {
    email: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
struct Firebase {
    identities: Identity,
    sign_in_provider: String
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
    firebase: Firebase
}

pub struct Validator {
    validation: Validation,
    validation_key: DecodingKey<'static>
}

static TEST_KEY: &[u8] = "-----BEGIN CERTIFICATE-----\nMIIDHDCCAgSgAwIBAgIICRh9AZ3kIy0wDQYJKoZIhvcNAQEFBQAwMTEvMC0GA1UE\nAxMmc2VjdXJldG9rZW4uc3lzdGVtLmdzZXJ2aWNlYWNjb3VudC5jb20wHhcNMjEw\nNzAzMDkyMDIzWhcNMjEwNzE5MjEzNTIzWjAxMS8wLQYDVQQDEyZzZWN1cmV0b2tl\nbi5zeXN0ZW0uZ3NlcnZpY2VhY2NvdW50LmNvbTCCASIwDQYJKoZIhvcNAQEBBQAD\nggEPADCCAQoCggEBAKw7TI2FsKTuCo829eqKsOb9mskZ6giHMcSsjrT+yRRT3rA3\nk/QcZ/L148EC3tMRtnnSMgDbBBPXol3zRooDMNpphjQY88BEK77LyMXD3ZDSi0Dl\ngF4OJJ1YMLtuxg8jUlyYooXQ+hH/XOSjjFAkBTpiC3svTqfn/57Iu8Z61egcnyyC\n3Wm+rt4rXPsmni/97sEx/HRWwJ/5RTA0tDNnfyFploMBUInN36TUcj7g7vmY3xsc\ne/zfJeKLReEnhWv7mTDV/L5LJOYH2ghQpGZes9YCR9VnxwBn5qLE5wJCSVtH9y3u\n5kok1uv3Q24gcailGi6N7ujy0zUiSCscwZPoOh0CAwEAAaM4MDYwDAYDVR0TAQH/\nBAIwADAOBgNVHQ8BAf8EBAMCB4AwFgYDVR0lAQH/BAwwCgYIKwYBBQUHAwIwDQYJ\nKoZIhvcNAQEFBQADggEBAA+OHI08nqK4aBKWI8rg0PtDJb0skcrq29/BtQK8YvFm\nEE/zzwSZRy7z1zWW6SOPprepDwVCalrSGuSt31+pGZquijNXdzoSpKr210QBW/Y3\nxD4dR25+IYHiR8ivU/M2yBKsd2GtnrK1Mgq/2XRXUjLj2ELP/yL+ir6tkOyBqSRX\nxSmww0QdSlJDC0VauU5bxxGLWmXfWXcRR8j56l8UkRfKp71MPJF9F1GEf7loTBe2\n8+EeU1CLUrCYCRrquBHHZMgz2ITVrH4fFftYGyDkmIy01SnmAjdq45i8JS3RLTkm\ntHglMPn7Qoc8N7od8JXi+6y4nXM6wGoVnAOYTuQx8Wg=\n-----END CERTIFICATE-----\n".as_bytes();

impl Validator {
    pub fn new() -> Validator {
        let validation  = get_validator();
        let validation_key = get_validation_key();
        Validator { validation, validation_key }
    }

    pub fn validate_token(&self, id_token: &str) -> Option<UserClaims> {
        let token = decode::<UserClaims>(id_token, &self.validation_key, &Validation::new(Algorithm::RS256));
        match token {
            Ok(token) => Some(token.claims),
            Err(e) => {
                panic!("Received invalid token. Error: {:?}", e);
            }
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

fn get_validation_key() -> DecodingKey<'static> {
    if let Ok(cert) = x509::X509::from_pem(TEST_KEY) {
        let key = cert.public_key().unwrap().public_key_to_der().unwrap();
        DecodingKey::from_rsa_der(&key).into_static()
    } else {
        panic!("Could not decode RSA Verification key from Google");
    }
}
