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
pub const SECRET: &str = "supersecretsecret";

#[derive(Clone)]
pub struct JWTAuth<'a> {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey<'a>
}

#[derive(Serialize, Deserialize)]
struct Claims {
    exp: usize,
}

impl JWTAuth<'_> {
    const EXPIRE_TIME: u64 = 30 * 24 * 60 * 60;

    pub fn new(secret: &[u8]) -> JWTAuth {
        JWTAuth {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret)
        }
    }

    pub fn create_token(&self) -> String {
        let claims = Claims {
            exp: (std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + Self::EXPIRE_TIME) as usize,
        };

        jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &self.encoding_key,
        )
            .unwrap()
    }

    fn validate_token(&self, token: &str) -> bool {
        jsonwebtoken::decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::default(),
        )
            .is_ok()
    }
}

pub async fn request_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    let jwt_auth = req.app_data::<JWTAuth>().unwrap();
    if jwt_auth.validate_token(credentials.token()) {
        Ok(req)
    } else {
        let challenge = Bearer::build().error(bearer::Error::InvalidToken).finish();
        Err(AuthenticationError::new(challenge).into())
    }
}
