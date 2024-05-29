// Import necessary crates and modules
use actix_files::Files; // For serving static files
use actix_web::{web, App, HttpResponse, HttpServer, Responder}; // For web server functionalities
use futures::stream::TryStreamExt; // For handling asynchronous streams
use mongodb::bson::doc; // For BSON document construction
use mongodb::bson::oid::ObjectId; // For MongoDB ObjectId handling
use mongodb::{Client, Collection}; // For MongoDB client and collection handling
use serde::{Deserialize, Serialize}; // For serialization and deserialization of JSON
use serde_json::json; // For creating JSON objects
use std::sync::{Arc, Mutex}; // For thread-safe shared state
use actix_cors::Cors;
use actix_web::http::header;

// Define a struct to represent a Book, with serialization and deserialization
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Book {
    #[serde(rename = "_id")] // Rename the field to "_id" in MongoDB
    #[serde(default)] // Provide a default value if not present
    #[serde(skip_serializing_if = "Option::is_none")] // Skip if None
    id: Option<ObjectId>,
    title: String,
    author: String,
    published_year: i32,
}

// Define a struct for creating new books, without an ID
#[derive(Serialize, Deserialize)]
struct NewBook {
    title: String,
    author: String,
    published_year: i32,
}

// Define a struct for the response format of a book
#[derive(Serialize)]
struct BookResponse {
    id: String,
    title: String,
    author: String,
    published_year: i32,
}

// Implement conversion from Book to BookResponse
impl From<Book> for BookResponse {
    fn from(book: Book) -> Self {
        BookResponse {
            id: book.id.unwrap_or_else(ObjectId::new).to_string(), // Convert ObjectId to string
            title: book.title,
            author: book.author,
            published_year: book.published_year,
        }
    }
}

// Define a handler for the index route, returning a bad request response
async fn index() -> impl Responder {
    HttpResponse::BadRequest().body("please specify the operation")
}

// Define a handler for creating a new book
async fn create_book(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>, // Shared MongoDB collection
    book: web::Json<NewBook>, // JSON payload representing a new book
) -> impl Responder {
    // Validate input data
    if book.title.is_empty() || book.author.is_empty() || book.published_year <= 0 {
        return HttpResponse::BadRequest().body("Invalid input data");
    }

    // Create a new Book instance
    let new_book = Book {
        id: Some(ObjectId::new()), // Generate a new ObjectId
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

// Define a handler for deleting all books
async fn delete_all_books(db_pool: web::Data<Arc<Mutex<Collection<Book>>>>) -> impl Responder {
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

// Define a handler for retrieving all books
async fn get_all_books(db_pool: web::Data<Arc<Mutex<Collection<Book>>>>) -> impl Responder {
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

// Define a handler for retrieving a book by its ID
async fn get_book_by_id(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>, // Shared MongoDB collection
    path: web::Path<String>, // Path parameter representing the book ID
) -> impl Responder {
    // Lock the collection for thread-safe access
    let collection = db_pool.lock().unwrap();
    let id = path.into_inner(); // Extract the ID from the path

    // Parse the ID into an ObjectId
    match ObjectId::parse_str(&id) {
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

// Define a handler for updating a book
async fn update_book(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>, // Shared MongoDB collection
    path: web::Path<String>, // Path parameter representing the book ID
    updated_book: web::Json<NewBook>, // JSON payload representing the updated book data
) -> impl Responder {
    // Lock the collection for thread-safe access
    let collection = db_pool.lock().unwrap();
    let id_str = path.into_inner(); // Extract the ID from the path

    // Parse the ID into an ObjectId
    match ObjectId::parse_str(&id_str) {
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

// Define a handler for deleting a book by its ID
async fn delete_book(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>, // Shared MongoDB collection
    path: web::Path<String>, // Path parameter representing the book ID
) -> impl Responder {
    // Lock the collection for thread-safe access
    let collection = db_pool.lock().unwrap();
    let id = path.into_inner(); // Extract the ID from the path

    // Parse the ID into an ObjectId
    match ObjectId::parse_str(&id) {
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
            .route("/books", web::get().to(index)) // Route for the index
            .route("/books/create", web::post().to(create_book)) // Route for creating a book
            .route("/books/getall", web::get().to(get_all_books)) // Route for getting all books
            .route("/books/get/{id}", web::get().to(get_book_by_id)) // Route for getting a book by ID
            .route("/books/update/{id}", web::put().to(update_book)) // Route for updating a book
            .route("/books/deleteall", web::get().to(delete_all_books)) // Route for deleting all books
            .route("/books/delete/{id}", web::delete().to(delete_book)) // Route for deleting a book by ID
            .service(Files::new("/", "src/HTML").index_file("index.html")) // Serve static files from the "src/HTML" directory
    })
    .bind("127.0.0.1:8080")? // Bind to the specified address and port
    .run() // Run the server
    .await // Await the server to finish
}
