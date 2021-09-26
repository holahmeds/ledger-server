use std::time::UNIX_EPOCH;

use actix_web::dev::ServiceRequest;
use actix_web::Error;
use actix_web_httpauth::extractors::{AuthenticationError, bearer};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::headers::www_authenticate::bearer::Bearer;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::Deserialize;
use serde::Serialize;

// TODO: Read from configuration file
const SECRET: &str = "supersecretsecret";

#[derive(Serialize, Deserialize)]
struct Claims {
    exp: usize,
}

pub fn create_token() -> String {
    let claims = Claims {
        exp: (std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 30 * 24 * 60 * 60) as usize,
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET.as_ref()),
    )
        .unwrap()
}

fn validate_token(token: &str) -> bool {
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET.as_ref()),
        &Validation::default(),
    )
        .is_ok()
}

pub async fn request_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    if validate_token(credentials.token()) {
        Ok(req)
    } else {
        let challenge = Bearer::build().error(bearer::Error::InvalidToken).finish();
        Err(AuthenticationError::new(challenge).into())
    }
}
