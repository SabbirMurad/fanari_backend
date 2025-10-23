use crate::utils::response::Response;
use serde::{ Serialize, Deserialize };
use actix_web::{web, Error, HttpResponse };
use serde_json::json;
use crate::{Model::Account::AccountRole, BuiltIns::jwt};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReqBody {
    refresh_token: String,
    user_id: String,
    role: AccountRole
}

pub async fn task(form_data: web::Json<ReqBody>) -> Result<HttpResponse, Error> {  
    let refresh_token = form_data.refresh_token.to_string();
    
    let result = jwt::refresh_token::status(&refresh_token);

    if let Err(error) = result {
        log::error!("{:?}",error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let option = result.unwrap();
    if let None = option {
        return Ok(Response::forbidden("status not found on this token"));
    }

    let status = option.unwrap();

    match status {
        jwt::Status::Active => {
            let access_token = jwt::access_token::generate_default(
                &form_data.user_id,
                form_data.role.clone(),    
            );

            return Ok(
                HttpResponse::Ok()
                .content_type("application/json")
                .json(json!({"access_token": access_token}))
            );
        },
        jwt::Status::Blocked => {
            return Ok(Response::forbidden("this token is blocked"));
        }
    }
}