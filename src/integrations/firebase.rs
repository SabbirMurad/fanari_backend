use std::env;

use actix_web::HttpResponse;
use google_api_auth::AuthenticationHandler;
use reqwest::StatusCode;
use serde_json::{json, Value};
use reqwest::{self, header::HOST};
use reqwest::header::{HeaderMap, CONTENT_TYPE};
use serde::{ Serialize, Deserialize };
use crate::utils::response::Response;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestBody { message: Message }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    token: String,
    notification: Option<Notification>,
    data: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Notification {
    title: Option<String>,
    body: Option<String>,
    image: Option<String>,
}

pub fn firebase_access_token() -> String {
    let json_string = json!({
        //TODO:
        // Your Firebase account authentication json data goes here
    }).to_string();

    //Handler using json `String`
    let handler = AuthenticationHandler::new(json_string.into());

    //Get a token with scoped read / write access to GCP DNS API.
    let token = handler.get_access_token_model(
    vec!["https://www.googleapis.com/auth/firebase.messaging".into()]);

    token.access_token
}

pub async fn send_notification(
    fcm_token: &str, 
    title: Option<String>,
    body: Option<String>,
    image: Option<String>,
    data: Option<serde_json::Value>
) -> Result<(), HttpResponse>{
    let firebase_project_id = env::var("FIREBASE_PROJECT_ID")
    .expect("FIREBASE_PROJECT_ID must be set on .env file");

    let access_token = match tokio::task::spawn_blocking(firebase_access_token).await {
        Ok(token) => token,
        Err(error) => {
            log::error!("{:?}", error);
            return Err(Response::internal_server_error(&error.to_string()));
        }
    };

    let client = reqwest::Client::new();
    let url = format!(
        "https://fcm.googleapis.com/v1/projects/{}/messages:send", firebase_project_id
    );
    let token = format!("Bearer {}", access_token);
    let mut header = HeaderMap::new();
    header.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    header.insert(HOST, "fcm.googleapis.com".parse().unwrap());
    header.insert("Authorization", token.parse().unwrap());

    let notification;
    if title.is_none() && body.is_none() && image.is_none() {
        notification = None;
    }
    else {
        notification = Some(Notification {
            title,
            body,
            image,
        });
    }

    let message = Message {
        token: fcm_token.to_string(),
        notification,
        data,
    };

    let body = RequestBody { message };

    let req_body = serde_json::to_string(&body).unwrap();

    let result = client
        .post(url)
        .headers(header)
        .body(req_body)
        .send().await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Err(Response::internal_server_error(&error.to_string()));
    }

    let res = result.unwrap();

    if res.status() == StatusCode::OK {
        Ok(())
    }
    else {
        let response = res.text().await;
        
        if let Err(error) = response {
            log::error!("{:?}", error);
            return Err(Response::internal_server_error(&error.to_string()));
        }
      
        let response_body = response.unwrap();
        Err(Response::internal_server_error(&response_body))
    }
}