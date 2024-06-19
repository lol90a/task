use actix_web::{HttpResponse, Responder};

// Define a handler for the index route, returning a bad request response
pub async fn index() -> impl Responder {
    HttpResponse::BadRequest().body("please specify the operation")
}
