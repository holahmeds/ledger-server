use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::HttpMessage;
use futures_util::future::{ready, LocalBoxFuture, Ready};
use ledger::user::UserId;

pub struct MockAuthentication {
    pub user_id: UserId,
}

impl<S, B> Transform<S, ServiceRequest> for MockAuthentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = MockAuthenticationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MockAuthenticationMiddleware {
            service,
            user_id: self.user_id.clone(),
        }))
    }
}

pub struct MockAuthenticationMiddleware<S> {
    service: S,
    user_id: UserId,
}

impl<S, B> Service<ServiceRequest> for MockAuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        req.extensions_mut().insert::<UserId>(self.user_id.clone());
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
