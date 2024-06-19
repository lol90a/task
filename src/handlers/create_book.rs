use actix_web::{web, HttpResponse, Responder};
use mongodb::Collection;
use std::sync::{Arc, Mutex};

use crate::models::book::{Book, NewBook};

// Define a handler for creating a new book
pub async fn create_book(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>, // Shared MongoDB collection
    book: web::Json<NewBook>, // JSON payload representing a new book
) -> impl Responder {
    // Validate input data
    if book.title.is_empty() || book.author.is_empty() || book.published_year <= 0 {
        return HttpResponse::BadRequest().body("Invalid input data");
    }

    // Create a new Book instance
    let new_book = Book {
        id: Some(mongodb::bson::oid::ObjectId::new()), // Generate a new ObjectId
        title: book.title.clone(),
        author: book.author.clone(),
        published_year: book.published_year,
    };

    // Insert the new book into the MongoDB collection
    let collection = db_pool.lock().unwrap();
    match collection.insert_one(new_book.clone(), None).await {
        Ok(_) => HttpResponse::Created().json(new_book), // Respond with the created book
        Err(err) => HttpResponse::InternalServerError().body(format!("Failed to insert book: {}", err)), // Handle errors
    }
}
