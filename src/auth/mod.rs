use std::time::UNIX_EPOCH;

use crate::user::UserId;
use actix_web::dev::ServiceRequest;
use actix_web::{Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::extractors::{bearer, AuthenticationError};
use actix_web_httpauth::headers::www_authenticate::bearer::Bearer;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::Deserialize;
use serde::Serialize;
use tracing_actix_web::RootSpan;

pub mod handlers;
pub mod password;

#[derive(Clone)]
pub struct JWTAuth<'a> {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey<'a>,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    exp: usize,
    sub: Option<UserId>,
}

impl JWTAuth<'_> {
    const EXPIRE_TIME: u64 = 30 * 24 * 60 * 60;

    pub fn from_base64_secret(
        secret: String,
    ) -> Result<JWTAuth<'static>, jsonwebtoken::errors::Error> {
        Ok(JWTAuth {
            encoding_key: EncodingKey::from_base64_secret(&secret)?,
            decoding_key: DecodingKey::from_base64_secret(&secret)?,
        })
    }

    pub fn create_token(&self) -> String {
        let claims = Claims {
            exp: Self::generate_exp(),
            sub: None,
        };

        jsonwebtoken::encode(&Header::default(), &claims, &self.encoding_key).unwrap()
    }

    pub fn create_token_for_user(&self, user_id: UserId) -> String {
        let claims = Claims {
            exp: Self::generate_exp(),
            sub: Some(user_id),
        };

        jsonwebtoken::encode(&Header::default(), &claims, &self.encoding_key).unwrap()
    }

    fn validate_token(&self, token: &str) -> Result<Option<UserId>, jsonwebtoken::errors::Error> {
        let claim =
            jsonwebtoken::decode::<Claims>(token, &self.decoding_key, &Validation::default())?;
        Ok(claim.claims.sub)
    }

    fn generate_exp() -> usize {
        (std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + Self::EXPIRE_TIME) as usize
    }
}

pub async fn request_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    let jwt_auth = req.app_data::<JWTAuth>().unwrap();
    if let Ok(user) = jwt_auth.validate_token(credentials.token()) {
        if let Some(user) = user {
            if let Some(root_span) = req.extensions().get::<RootSpan>() {
                root_span.record("user_id", &user.as_str());
            }
            req.extensions_mut().insert::<UserId>(user);
        }
        Ok(req)
    } else {
        let challenge = Bearer::build().error(bearer::Error::InvalidToken).finish();
        Err(AuthenticationError::new(challenge).into())
    }
}
