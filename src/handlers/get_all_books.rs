use actix_web::{web, HttpResponse, Responder};
use mongodb::Collection;
use std::sync::{Arc, Mutex};
use futures::stream::TryStreamExt;

use crate::models::book::{Book, BookResponse};

// Define a handler for retrieving all books
pub async fn get_all_books(db_pool: web::Data<Arc<Mutex<Collection<Book>>>>) -> impl Responder {
    // Lock the collection for thread-safe access
    let collection = match db_pool.lock() {
        Ok(collection) => collection,
        Err(err) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to lock the database collection: {}", err));
        }
    };

    // Retrieve all books from the collection
    let cursor = match collection.find(None, None).await {
        Ok(cursor) => cursor,
        Err(err) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to query books: {}", err));
        }
    };

    // Collect the books from the cursor
    let books: Vec<Book> = match cursor.try_collect().await {
        Ok(books) => books,
        Err(err) => {
            eprintln!("Failed to collect books: {:?}", err);
            return HttpResponse::InternalServerError()
                .body(format!("Failed to collect books: {}", err));
        }
    };

    // Convert the books to BookResponse format and respond with JSON
    let book_responses: Vec<BookResponse> = books.into_iter().map(BookResponse::from).collect();
    HttpResponse::Ok().json(book_responses)
}
