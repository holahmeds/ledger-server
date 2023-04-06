use crate::user::UserId;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::Deserialize;
use serde::Serialize;
use std::time::UNIX_EPOCH;

#[derive(Clone)]
pub struct JWTAuth {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    exp: usize,
    sub: UserId,
}

impl JWTAuth {
    const EXPIRE_TIME: u64 = 30 * 24 * 60 * 60;

    pub fn from_secret(secret: Vec<u8>) -> JWTAuth {
        JWTAuth {
            encoding_key: EncodingKey::from_secret(&secret),
            decoding_key: DecodingKey::from_secret(&secret),
        }
    }

    pub fn create_token(&self, user_id: UserId) -> String {
        let claims = Claims {
            exp: Self::generate_exp(),
            sub: user_id,
        };

        jsonwebtoken::encode(&Header::default(), &claims, &self.encoding_key).unwrap()
    }

    pub fn validate_token(&self, token: &str) -> Result<UserId, jsonwebtoken::errors::Error> {
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

#[cfg(test)]
mod tests {
    use crate::auth::jwt::JWTAuth;
    use base64::Engine;

    #[test]
    async fn valid_token() {
        let secret: [u8; 32] = rand::random();
        let jwt_auth = JWTAuth::from_secret(secret.to_vec());

        let token = jwt_auth.create_token("alice".into());
        assert_eq!(jwt_auth.validate_token(&token), Ok("alice".into()));
    }

    #[test]
    async fn invalid_token() {
        let secret: [u8; 32] = rand::random();
        let jwt_auth = JWTAuth::from_secret(secret.to_vec());

        let token_bytes: [u8; 32] = rand::random();
        let base64_engine = base64::engine::general_purpose::STANDARD;
        let token = base64_engine.encode(token_bytes);
        assert!(jwt_auth.validate_token(&token).is_err())
    }
}
