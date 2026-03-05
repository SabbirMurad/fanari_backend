use serde::Deserialize;
use serde_json::json;
use mongodb::bson::doc;
use futures::StreamExt;
use crate::BuiltIns::mongo::MongoDB;
use crate::BuiltIns::jwt;
use crate::utils::response::Response;
use actix_web::{web, Error, HttpResponse, HttpRequest};

use crate::model::{
    Account::{
        AccountCore,
        AccountProfile,
        Gender,
    },
    ImageStruct,
};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub exclude_self: Option<bool>,
    pub gender: Option<String>,
    pub min_age: Option<u32>,
    pub max_age: Option<u32>,
    pub verified_only: Option<bool>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
}

pub async fn task(req: HttpRequest, query: web::Query<SearchQuery>) -> Result<HttpResponse, Error> {
    // Optional authentication — extract user_id if a valid token is present
    let user_id: Option<String> = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .map(|h| h.trim_start_matches("Bearer ").to_string())
        .and_then(|token| jwt::access_token::verify(&token, jwt::Key::Local).ok())
        .map(|claims| claims.sub);

    let search_term = query.q.trim();
    if search_term.is_empty() {
        return Ok(Response::bad_request("search query cannot be empty"));
    }

    let exclude_self = query.exclude_self.unwrap_or(true);
    let limit = query.limit.unwrap_or(20).min(50) as i64;
    let page = query.page.unwrap_or(1).max(1);
    let skip = limit * ((page as i64).saturating_sub(1));

    let db = MongoDB.connect();

    // Build regex pattern for case-insensitive matching
    let regex_pattern = regex_escape(search_term);

    // Search account_core for username matches
    let core_collection = db.collection::<AccountCore>("account_core");
    let core_filter = doc! {
        "username": { "$regex": &regex_pattern, "$options": "i" },
        "email_verified": true
    };
    let core_cursor = core_collection.find(core_filter).await;
    if let Err(error) = core_cursor {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let mut username_match_ids: Vec<String> = Vec::new();
    let mut core_map: std::collections::HashMap<String, AccountCore> = std::collections::HashMap::new();
    let mut cursor = core_cursor.unwrap();
    while let Some(result) = cursor.next().await {
        if let Ok(account) = result {
            username_match_ids.push(account.uuid.clone());
            core_map.insert(account.uuid.clone(), account);
        }
    }

    println!("username_match_ids: {:#?}", username_match_ids);

    // Search account_profile for first_name or last_name matches
    let profile_collection = db.collection::<AccountProfile>("account_profile");
    let profile_filter = doc! {
        "$or": [
            { "first_name": { "$regex": &regex_pattern, "$options": "i" } },
            { "last_name": { "$regex": &regex_pattern, "$options": "i" } },
        ]
    };
    let profile_cursor = profile_collection.find(profile_filter).await;
    if let Err(error) = profile_cursor {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let mut name_match_ids: Vec<String> = Vec::new();
    let mut profile_map: std::collections::HashMap<String, AccountProfile> = std::collections::HashMap::new();
    let mut cursor = profile_cursor.unwrap();
    while let Some(result) = cursor.next().await {
        if let Ok(profile) = result {
            name_match_ids.push(profile.uuid.clone());
            profile_map.insert(profile.uuid.clone(), profile);
        }
    }

    println!("name_match_ids: {:#?}", name_match_ids);

    // Merge unique user IDs (username matches first, then name matches)
    let mut matched_ids: Vec<String> = Vec::new();
    for id in &username_match_ids {
        if !matched_ids.contains(id) {
            matched_ids.push(id.clone());
        }
    }
    for id in &name_match_ids {
        if !matched_ids.contains(id) {
            matched_ids.push(id.clone());
        }
    }

    // Exclude self if logged in and exclude_self is true
    if exclude_self {
        if let Some(ref uid) = user_id {
            matched_ids.retain(|id| id != uid);
        }
    }
    println!("matched_ids: {:#?}", matched_ids);

    // Apply filters: gender, age, verified_only
    let mut filtered_ids: Vec<String> = Vec::new();


    for id in &matched_ids {
        // Ensure we have core data and check email_verified
        let core = match core_map.get(id) {
            Some(c) => c,
            None => {
                let result = core_collection.find_one(doc!{"uuid": id}).await;
                if let Err(error) = result {
                    log::error!("{:?}", error);
                    continue;
                }
                let option = result.unwrap();
                if option.is_none() {
                    continue;
                }
                let c = option.unwrap();
                core_map.insert(id.clone(), c);
                core_map.get(id).unwrap()
            }
        };

        if !core.email_verified {
            continue;
        }

        // Ensure we have the profile data
        let profile = match profile_map.get(id) {
            Some(p) => p,
            None => {
                let result = profile_collection.find_one(doc!{"uuid": id}).await;
                if let Err(error) = result {
                    log::error!("{:?}", error);
                    continue;
                }
                let option = result.unwrap();
                if option.is_none() {
                    continue;
                }
                let p = option.unwrap();
                profile_map.insert(id.clone(), p);
                profile_map.get(id).unwrap()
            }
        };

        // Gender filter
        if let Some(ref gender_str) = query.gender {
            let target_gender = match gender_str.to_lowercase().as_str() {
                "male" => Some(Gender::Male),
                "female" => Some(Gender::Female),
                "others" => Some(Gender::Others),
                _ => None,
            };

            if let Some(target) = target_gender {
                match &profile.gender {
                    Some(g) => {
                        let gender_str_profile = format!("{:?}", g);
                        let target_str = format!("{:?}", target);
                        if gender_str_profile != target_str {
                            continue;
                        }
                    },
                    None => continue,
                }
            }
        }

        // Age filter (based on date_of_birth)
        if query.min_age.is_some() || query.max_age.is_some() {
            match profile.date_of_birth {
                Some(dob_millis) => {
                    let now_millis = chrono::Utc::now().timestamp_millis();
                    let age_millis = now_millis - dob_millis;
                    let age_years = (age_millis / (365_25 * 24 * 60 * 60 * 100)) as u32;

                    if let Some(min) = query.min_age {
                        if age_years < min {
                            continue;
                        }
                    }
                    if let Some(max) = query.max_age {
                        if age_years > max {
                            continue;
                        }
                    }
                },
                None => continue,
            }
        }

        // Verified only filter
        if query.verified_only.unwrap_or(false) && !profile.profile_verified {
            continue;
        }

        filtered_ids.push(id.clone());
    }

    println!("filtered_ids: {:#?}", filtered_ids);

    // Apply pagination
    let total = filtered_ids.len() as i64;
    let paginated_ids: Vec<String> = filtered_ids
        .into_iter()
        .skip(skip as usize)
        .take(limit as usize)
        .collect();

    // Build response
    let mut results: Vec<serde_json::Value> = Vec::new();

    for id in &paginated_ids {
        // Get core data
        let core = match core_map.get(id) {
            Some(c) => c,
            None => {
                let result = core_collection.find_one(doc!{"uuid": id}).await;
                if let Err(error) = result {
                    log::error!("{:?}", error);
                    continue;
                }
                let option = result.unwrap();
                if option.is_none() {
                    continue;
                }
                let c = option.unwrap();
                core_map.insert(id.clone(), c);
                core_map.get(id).unwrap()
            }
        };

        let profile = profile_map.get(id).unwrap();

        // Resolve profile picture
        let profile_picture: Option<ImageStruct> = match &profile.profile_picture {
            Some(image_id) => {
                let collection = db.collection::<ImageStruct>("image");
                let result = collection.find_one(doc!{"uuid": image_id}).await;
                if let Err(error) = result {
                    log::error!("{:?}", error);
                    None
                } else {
                    result.unwrap()
                }
            },
            None => None,
        };

        results.push(json!({
            "uuid": &core.uuid,
            "username": &core.username,
            "first_name": &profile.first_name,
            "last_name": &profile.last_name,
            "profile_picture": profile_picture,
        }));
    }

    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "results": results,
            "total": total,
            "page": page,
            "limit": limit,
        }))
    )
}

/// Escapes special regex characters in a string so it can be used as a literal pattern.
fn regex_escape(input: &str) -> String {
    let special_chars = ['.', '^', '$', '*', '+', '?', '(', ')', '[', ']', '{', '}', '|', '\\'];
    let mut escaped = String::with_capacity(input.len() * 2);
    for ch in input.chars() {
        if special_chars.contains(&ch) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}
