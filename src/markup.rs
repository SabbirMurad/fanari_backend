use tera::{Tera, Context};
use actix_web::{web, error, Error, HttpResponse};

pub async fn home(template: web::Data<Tera>) -> Result<HttpResponse, Error> {
  let res_data = template.render(
    "home.html",
    &Context::new()
  )
  .map_err(|e|error::ErrorInternalServerError(e))?;
  
  Ok(HttpResponse::Ok().content_type("text/html").body(res_data))
}

pub async fn sign_in(template: web::Data<Tera>) -> Result<HttpResponse, Error> {
  let res_data = template.render(
    "admin/auth.html",
    &Context::new()
  )
  .map_err(|e|error::ErrorInternalServerError(e))?;
  
  Ok(HttpResponse::Ok().content_type("text/html").body(res_data))
}

pub async fn admin_dashboard(template: web::Data<Tera>) -> Result<HttpResponse, Error> {
  let res_data = template.render(
    "admin/dashboard.html",
    &Context::new()
  )
  .map_err(|e|error::ErrorInternalServerError(e))?;
  
  Ok(HttpResponse::Ok().content_type("text/html").body(res_data))
}