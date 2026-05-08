use actix_web::{
    body::BoxBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use crypto::jwt::verify_token;
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

pub struct Authenticated;

impl<S> Transform<S, ServiceRequest> for Authenticated
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { service }))
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));

        match auth_header {
            Some(token) => match verify_token(token) {
                Ok(claims) => {
                    req.extensions_mut().insert(claims);
                    let fut = self.service.call(req);
                    Box::pin(async move { fut.await })
                }
                Err(_) => {
                    let (req, _) = req.into_parts();
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({ "error": "Invalid or expired token" }))
                        .map_into_boxed_body();
                    Box::pin(async { Ok(ServiceResponse::new(req, response)) })
                }
            },
            None => {
                let (req, _) = req.into_parts();
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({ "error": "Missing authorization token" }))
                    .map_into_boxed_body();
                Box::pin(async { Ok(ServiceResponse::new(req, response)) })
            }
        }
    }
}