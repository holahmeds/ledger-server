use crate::user::UserId;
use actix_web::dev::ServiceRequest;
use actix_web::{Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::extractors::{bearer, AuthenticationError};
use actix_web_httpauth::headers::www_authenticate::bearer::Bearer;
use jwt::JWTAuth;
use tracing_actix_web::RootSpan;

pub mod handlers;
pub mod jwt;
pub mod password;

pub async fn request_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_auth = req.app_data::<JWTAuth>().unwrap();
    if let Ok(user) = jwt_auth.validate_token(credentials.token()) {
        if let Some(root_span) = req.extensions().get::<RootSpan>() {
            root_span.record("user_id", &user.as_str());
        }
        req.extensions_mut().insert::<UserId>(user);
        Ok(req)
    } else {
        let challenge = Bearer::build().error(bearer::Error::InvalidToken).finish();
        Err((AuthenticationError::new(challenge).into(), req))
    }
}
