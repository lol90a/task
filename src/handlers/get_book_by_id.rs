use actix_web::{web, HttpResponse, Responder};
use mongodb::Collection;
use std::sync::{Arc, Mutex};
use mongodb::bson::doc;

use crate::models::book::{Book, BookResponse};

// Define a handler for retrieving a book by its ID
pub async fn get_book_by_id(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>, // Shared MongoDB collection
    path: web::Path<String>, // Path parameter representing the book ID
) -> impl Responder {
    // Lock the collection for thread-safe access
    let collection = db_pool.lock().unwrap();
    let id = path.into_inner(); // Extract the ID from the path

    // Parse the ID into an ObjectId
    match mongodb::bson::oid::ObjectId::parse_str(&id) {
        Ok(object_id) => match collection.find_one(doc! { "_id": object_id }, None).await {
            Ok(Some(book)) => HttpResponse::Ok().json(BookResponse::from(book)), // Respond with the found book
            Ok(None) => HttpResponse::NotFound().body("Book not found"), // Handle book not found
            Err(err) => {
                eprintln!("Failed to get book by ID: {:?}", err);
                HttpResponse::InternalServerError()
                    .body(format!("Failed to get book by ID: {}", err))
            }
        },
        Err(err) => {
            eprintln!("Invalid ObjectId format: {:?}", err);
            HttpResponse::BadRequest().body("Invalid ObjectId format") // Handle invalid ObjectId format
        }
    }
}
