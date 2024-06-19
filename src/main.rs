// Import necessary crates and modules
use actix_files::Files; // For serving static files
use actix_web::{web, App, HttpServer}; // For web server functionalities
use mongodb::{Client, Collection}; // For MongoDB client and collection handling
use std::sync::{Arc, Mutex}; // For thread-safe shared state
use actix_cors::Cors;
use actix_web::http::header;

mod handlers;
mod models;

use handlers::*;
use models::book::Book;

// Main function to set up and run the Actix web server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // MongoDB connection string
    let mongo_address = "mongodb+srv://aliadel:LnLthlufzQll6DTD@bookstore.n2xxh9r.mongodb.net/?retryWrites=true&w=majority&appName=BookStore";
    
    // Create a MongoDB client
    let client = Client::with_uri_str(mongo_address).await.unwrap();
    
    // Access the database and collection
    let db = client.database("book_db");
    let collection = db.collection::<Book>("books");

    // Wrap the collection in an Arc and Mutex for thread-safe shared state
    let data = web::Data::new(Arc::new(Mutex::new(collection)));

    // Configure and start the HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://127.0.0.1:8080")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![header::CONTENT_TYPE])
                    .max_age(3600),
            )
            .app_data(data.clone()) // Share the MongoDB collection across handlers
            .route("/books", web::get().to(index::index)) // Route for the index
            .route("/books/create", web::post().to(create_book::create_book)) // Route for creating a book
            .route("/books/getall", web::get().to(get_all_books::get_all_books)) // Route for getting all books
            .route("/books/get/{id}", web::get().to(get_book_by_id::get_book_by_id)) // Route for getting a book by ID
            .route("/books/update/{id}", web::put().to(update_book::update_book)) // Route for updating a book
            .route("/books/deleteall", web::get().to(delete_all_books::delete_all_books)) // Route for deleting all books
            .route("/books/delete/{id}", web::delete().to(delete_book::delete_book)) // Route for deleting a book by ID
            .service(Files::new("/", "src/HTML").index_file("index.html")) // Serve static files from the "src/HTML" directory
    })
    .bind("127.0.0.1:8080")? // Bind to the specified address and port
    .run() // Run the server
    .await // Await the server to finish
}
