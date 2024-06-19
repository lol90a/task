use actix_web::{web, HttpResponse, Responder};
use mongodb::Collection;
use std::sync::{Arc, Mutex};
use serde_json::json;
use mongodb::bson::doc;

use crate::models::book::{Book, NewBook};

// Define a handler for updating a book
pub async fn update_book(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>, // Shared MongoDB collection
    path: web::Path<String>, // Path parameter representing the book ID
    updated_book: web::Json<NewBook>, // JSON payload representing the updated book data
) -> impl Responder {
    // Lock the collection for thread-safe access
    let collection = db_pool.lock().unwrap();
    let id_str = path.into_inner(); // Extract the ID from the path

    // Parse the ID into an ObjectId
    match mongodb::bson::oid::ObjectId::parse_str(&id_str) {
        Ok(object_id) => {
            // Create the update document
            let update_doc = doc! {
                "$set": {
                    "title": &updated_book.title,
                    "author": &updated_book.author,
                    "published_year": &updated_book.published_year,
                }
            };

            // Update the book in the collection
            match collection
                .update_one(doc! { "_id": object_id }, update_doc, None)
                .await
            {
                Ok(result) => {
                    if result.matched_count > 0 {
                        HttpResponse::Ok().json(json!({ "message": "Book updated successfully" })) // Respond with success message
                    } else {
                        HttpResponse::NotFound().body("Book not found") // Handle book not found
                    }
                }
                Err(err) => {
                    eprintln!("Failed to update book: {:?}", err);
                    HttpResponse::InternalServerError()
                        .body(format!("Failed to update book: {}", err))
                }
            }
        }
        Err(err) => {
            eprintln!("Invalid ObjectId format: {:?}", err);
            HttpResponse::BadRequest().body("Invalid ObjectId format") // Handle invalid ObjectId format
        }
    }
}
