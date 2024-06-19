use actix_web::{web, HttpResponse, Responder};
use mongodb::Collection;
use std::sync::{Arc, Mutex};
use mongodb::bson::doc;

use crate::models::book::Book;

// Define a handler for deleting all books
pub async fn delete_all_books(db_pool: web::Data<Arc<Mutex<Collection<Book>>>>) -> impl Responder {
    // Lock the collection for thread-safe access
    let collection = match db_pool.lock() {
        Ok(collection) => collection,
        Err(err) => {
            eprintln!("Failed to lock the database collection: {:?}", err);
            return HttpResponse::InternalServerError()
                .body(format!("Failed to lock the database collection: {}", err));
        }
    };

    // Delete all documents in the collection
    match collection.delete_many(doc! {}, None).await {
        Ok(result) => HttpResponse::Ok().body(format!("Deleted {} books", result.deleted_count)), // Respond with the number of deleted books
        Err(err) => {
            eprintln!("Failed to delete books: {:?}", err);
            HttpResponse::InternalServerError().body(format!("Failed to delete books: {}", err))
        }
    }
}
