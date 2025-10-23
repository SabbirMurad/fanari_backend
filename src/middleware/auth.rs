use serde_json::json;
use crate::BuiltIns::jwt;
use futures::future::{ready, Ready};
use crate::Model::Account::AccountRole;
use actix_web::{web, dev::Payload, Error, FromRequest, HttpRequest};

#[derive(Debug, Clone)]
pub enum AccessRequirement {
    AnyToken,
    Role(AccountRole),
    AnyOf(Vec<AccountRole>),
}

#[derive(Debug)]
pub struct RequireAccess {
    pub user_id: String,
    pub role: AccountRole,
}

impl FromRequest for RequireAccess {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let config = req
            .app_data::<web::Data<AccessRequirement>>()
            .expect("AccessRequirement not found in app_data")
            .get_ref()
            .clone();

        let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());
    
        if auth_header.is_none() {
            return ready(Err(actix_web::error::ErrorUnauthorized(
                json!({"error": "Missing authorization header"})
            )));
        }
                
        let token = auth_header.unwrap()
        .trim_start_matches("Bearer ").to_string();

        // Validating access token
        let result = jwt::access_token::verify(
            &token,
            jwt::Key::Local
        );

        if let Err(err) = result {
            log::error!("{:?}", err);
            return ready(Err(actix_web::error::ErrorUnauthorized(
                json!({"error": "Invalid authorization token"})
            )));
        }

        let claims = result.unwrap();

        let pass = match &config {
            AccessRequirement::AnyToken => true,
            AccessRequirement::Role(r) => &claims.role == r,
            AccessRequirement::AnyOf(roles) => roles.contains(&claims.role),
        };

        if pass {
            ready(Ok(Self {
                user_id: claims.sub,
                role: claims.role,
            }))
        } else {
            ready(Err(actix_web::error::ErrorForbidden(
                json!({"error": "Not authorized to perform this action"})
            )))
        }
    }
}