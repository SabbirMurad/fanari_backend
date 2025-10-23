use actix_web::web;
use crate::Handler;

pub fn router(cfg: &mut web::ServiceConfig) {
    cfg.service(
    web::scope("/api/auth")
        .route(
            "/refresh",
            web::post().to(Handler::Auth::Refresh::task)
        )
        .route(
            "/sign-up",
            web::post().to(Handler::Auth::SignUp::task)
        )
        .route(
            "/resend-verification-code",
            web::post().to(Handler::Auth::ResendVerificationCode::task)
        )
        .route(
            "/validate-email",
            web::post().to(Handler::Auth::VerifyEmail::task)
        )
        .route(
            "/sign-in",
            web::post().to(Handler::Auth::SignIn::task)
        )
        .route(
            "/sign-out",
            web::post().to(Handler::Auth::SignOut::task)
        )
        .route(
            "/forgot-password",
            web::post().to(Handler::Auth::ForgotPassword::task)
        )
        .route(
            "/verify-reset-code",
            web::post().to(Handler::Auth::VerifyResetCode::task)
        )
        .route(
            "/reset-password",
            web::post().to(Handler::Auth::ResetPassword::task)
        )
        .route(
            "/user/{email_or_username}",
            web::get().to(Handler::Auth::Get::task)
        )
    );
}