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
    sub: Option<UserId>,
}

impl JWTAuth {
    const EXPIRE_TIME: u64 = 30 * 24 * 60 * 60;

    pub fn from_base64_secret(secret: String) -> Result<JWTAuth, jsonwebtoken::errors::Error> {
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

    pub fn validate_token(
        &self,
        token: &str,
    ) -> Result<Option<UserId>, jsonwebtoken::errors::Error> {
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

    #[test]
    async fn returns_user() {
        let secret: [u8; 32] = rand::random();
        let secret = base64::encode(secret);
        let jwt_auth = JWTAuth::from_base64_secret(secret).unwrap();

        let token = jwt_auth.create_token();
        assert_eq!(jwt_auth.validate_token(&token).unwrap(), None);

        let token = jwt_auth.create_token_for_user("alice".into());
        assert_eq!(
            jwt_auth.validate_token(&token).unwrap(),
            Some("alice".into())
        );
    }
}
