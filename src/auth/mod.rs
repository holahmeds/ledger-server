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

/// Validates credentials using [JWTAuth]. If valid, injects the user id into request and into the
/// [RootSpan]
pub async fn credentials_validator(
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

#[cfg(test)]
mod tests {
    use super::credentials_validator;
    use crate::auth::jwt::JWTAuth;
    use crate::user::UserId;
    use actix_web::http::StatusCode;
    use actix_web::test::TestRequest;
    use actix_web::{http, test, web, App, Responder};
    use actix_web_httpauth::middleware::HttpAuthentication;
    use rstest::fixture;
    use rstest::rstest;

    macro_rules! build_service {
        ($jwt_auth:ident) => {{
            let bearer_auth_middleware = HttpAuthentication::bearer(credentials_validator);
            let app = App::new()
                .app_data($jwt_auth)
                .route("/", web::get().to(return_user))
                .wrap(bearer_auth_middleware);
            test::init_service(app).await
        }};
    }

    #[fixture]
    fn jwt_auth() -> JWTAuth {
        let secret: [u8; 32] = rand::random();
        JWTAuth::from_secret(secret.to_vec())
    }

    #[rstest]
    #[test]
    async fn valid_user(jwt_auth: JWTAuth) {
        let user_id: UserId = "test".into();
        let token = jwt_auth.create_token(user_id.clone());

        let service = build_service!(jwt_auth);

        let request = TestRequest::get()
            .uri("/")
            .insert_header((
                http::header::AUTHORIZATION,
                (String::from("Bearer ") + &token),
            ))
            .to_request();
        let response = test::call_service(&service, request).await;
        assert!(
            response.status().is_success(),
            "Response status is {}",
            response.status()
        );

        let body = test::read_body(response).await;
        assert_eq!(user_id.as_bytes(), &body)
    }

    #[rstest]
    #[test]
    async fn invalid_user(jwt_auth: JWTAuth) {
        let user_id: UserId = "test".into();
        let token = jwt_auth.create_token("other_user".into());

        let service = build_service!(jwt_auth);

        let request = TestRequest::get()
            .uri("/")
            .insert_header((
                http::header::AUTHORIZATION,
                (String::from("Bearer ") + &token),
            ))
            .to_request();
        let response = test::call_service(&service, request).await;
        let body = test::read_body(response).await;
        assert_ne!(user_id.as_bytes(), &body)
    }

    #[rstest]
    #[test]
    async fn no_token(jwt_auth: JWTAuth) {
        let service = build_service!(jwt_auth);

        let request = TestRequest::get().uri("/").to_request();
        let response = test::call_service(&service, request).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED)
    }

    async fn return_user(user_id: web::ReqData<UserId>) -> impl Responder {
        user_id.into_inner()
    }
}
