use actix_web::{web, HttpResponse, Responder};
use mongodb::Collection;
use std::sync::{Arc, Mutex};
use serde_json::json;
use mongodb::bson::doc;

use crate::models::book::Book;

// Define a handler for deleting a book by its ID
pub async fn delete_book(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>, // Shared MongoDB collection
    path: web::Path<String>, // Path parameter representing the book ID
) -> impl Responder {
    // Lock the collection for thread-safe access
    let collection = db_pool.lock().unwrap();
    let id = path.into_inner(); // Extract the ID from the path

    // Parse the ID into an ObjectId
    match mongodb::bson::oid::ObjectId::parse_str(&id) {
        Ok(object_id) => match collection.delete_one(doc! { "_id": object_id }, None).await {
            Ok(result) => {
                if result.deleted_count > 0 {
                    HttpResponse::Ok().json(json!({ "message": "Book deleted successfully" })) // Respond with success message
                } else {
                    HttpResponse::NotFound().body("Book not found") // Handle book not found
                }
            }
            Err(err) => {
                eprintln!("Failed to delete book: {:?}", err);
                HttpResponse::InternalServerError().body(format!("Failed to delete book: {}", err))
            }
        },
        Err(err) => {
            eprintln!("Invalid ObjectId format: {:?}", err);
            HttpResponse::BadRequest().body("Invalid ObjectId format") // Handle invalid ObjectId format
        }
    }
}
