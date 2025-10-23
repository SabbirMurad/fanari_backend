use serde::{Deserialize, Serialize};
use super::ImageStruct;

//role for account
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountRole { Administrator, User}
impl std::fmt::Display for AccountRole {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt,"{:?}", self)
    }
}

//gender for account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Gender { Male, Female, Others }
impl std::fmt::Display for Gender {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt,"{:?}", self)
    }
}

//account_core
#[derive(Debug, Deserialize, Serialize)]
pub struct AccountCore {
    pub uuid: String,
    pub email_address: String,
    pub username: String,
    pub password: String,
    pub email_verified: bool,
    pub role: AccountRole,
    pub two_a_factor_auth_enabled: bool,
    pub two_a_factor_auth_updated: Option<i64>,
    
    pub created_at: i64,
    pub suspended_at: Option<i64>,
    pub suspended_by: Option<String>,
}

//account_profile
#[derive(Debug, Deserialize, Serialize)]
pub struct AccountProfile {
    pub uuid: String,
    
    pub full_name: String,
    pub phone_number: Option<String>,
    pub date_of_birth: Option<i64>,
    pub gender: Option<Gender>,
    pub profile_picture: Option<ImageStruct>,
    pub biography: Option<String>,
    pub profile_verified: bool,

    pub modified_at: i64,
}

//account_verification_request
#[derive(Debug, Deserialize, Serialize)]
pub struct AccountVerificationRequest {
    pub uuid: String,
    pub user_id: String,
    pub validation_code: String,
    pub expires_at: i64
}

//password_reset_request
#[derive(Debug, Deserialize, Serialize)]
pub struct PasswordResetRequest {
    pub uuid: String,
    pub user_id: String,
    pub secret_key: String,
    pub validation_code: String,
    pub code_validated: bool,
    pub expires_at: i64,
}

//sign_in_verification_request
#[derive(Debug, Deserialize, Serialize)]
pub struct SignInVerificationRequest {
    pub uuid: String,
    pub user_id: String,
    pub validation_code: String,
    pub expires_at: i64,
}