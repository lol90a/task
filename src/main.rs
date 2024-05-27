use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use mongodb::{Client, Collection, bson::{doc, Document, from_document}};
use futures::StreamExt;
use uuid::Uuid;
use actix_files as fs;



#[derive(Serialize, Deserialize, Clone)]
struct Book {
    #[serde(rename = "_id")]
    id: String,
    title: String,
    author: String,
    published_year: i32,
}

#[derive(Serialize, Deserialize)]
struct NewBook {
    title: String,
    author: String,
    published_year: i32,
}

#[derive(Serialize)]
struct BookResponse {
    id: String,
    title: String,
    author: String,
    published_year: i32,
}

impl From<Book> for BookResponse {
    fn from(book: Book) -> Self {
        BookResponse {
            id: book.id,
            title: book.title,
            author: book.author,
            published_year: book.published_year,
        }
    }
}

async fn create_book(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>,
    book: web::Json<NewBook>,
) -> impl Responder {
    if book.title.is_empty() || book.author.is_empty() || book.published_year <= 0 {
        return HttpResponse::BadRequest().body("Invalid input data");
    }

    let new_book = Book {
        id: Uuid::new_v4().to_string(),
        title: book.title.clone(),
        author: book.author.clone(),
        published_year: book.published_year,
    };

    let collection = db_pool.lock().unwrap();
    match collection.insert_one(new_book.clone(), None).await {
        Ok(_) => {
            HttpResponse::Created().json(new_book)
        },
        Err(err) => {
            eprintln!("Failed to insert book: {:?}", err);
            HttpResponse::InternalServerError().body(format!("Failed to insert book: {}", err))
        }
    }
}

use serde_json::json;

#[derive(Serialize)]
struct BooksResponse {
    books: Vec<BookResponse>,
}

use log::error;


// Function to get all books from the database
async fn get_all_books(db_pool: web::Data<Collection<Book>>) -> impl Responder {
    let cursor = match db_pool.find(None, None).await {
        Ok(cursor) => cursor,
        Err(err) => {
            error!("Failed to query books: {:?}", err);
            return HttpResponse::InternalServerError().json("Failed to query books");
        }
    };

    let mut books = Vec::new();
    let mut stream = cursor;

    while let Some(result) = stream.next().await {
        match result {
            Ok(doc) => {
                match mongodb::bson::from_document(doc) {
                    Ok(book) => book.PUSH(BookResponse::from(book)),
                    Err(err) => {
                        error!("Failed to deserialize book: {:?}", err);
                        return HttpResponse::InternalServerError().json("Failed to deserialize book");
                    }
                };
            }
            Err(err) => {
                error!("Failed to retrieve book document: {:?}", err);
                return HttpResponse::InternalServerError().json("Failed to retrieve book document");
            }
        }
    }

    let response_json = json!({ "books": books });

    HttpResponse::Ok().json(response_json)
}



async fn update_book(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>,
    path: web::Path<Uuid>,
    updated_book: web::Json<NewBook>,
) -> impl Responder {
    if updated_book.title.is_empty() || updated_book.author.is_empty() || updated_book.published_year <= 0 {
        return HttpResponse::BadRequest().body("Invalid input data");
    }

    let collection = db_pool.lock().unwrap();
    let id = path.into_inner();

    let update_doc = doc! {
        "$set": {
            "title": &updated_book.title,
            "author": &updated_book.author,
            "published_year": &updated_book.published_year,
        }
    };

    match collection.update_one(doc! { "_id": id.to_string() }, update_doc, None).await {
        Ok(_) => HttpResponse::Ok().json(json!({ "message": "Book updated successfully" })),
        Err(err) => {
            eprintln!("Failed to update book: {:?}", err);
            HttpResponse::InternalServerError().body(format!("Failed to update book: {}", err))
        }
    }
}

async fn delete_book(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let collection = db_pool.lock().unwrap();
    let id = path.into_inner();

    match collection.delete_one(doc! { "_id": id.to_string() }, None).await {
        Ok(_) => HttpResponse::Ok().json(json!({ "message": "Book deleted successfully" })),
        Err(err) => {
            eprintln!("Failed to delete book: {:?}", err);
            HttpResponse::InternalServerError().body(format!("Failed to delete book: {}", err))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_address = "mongodb+srv://aliadel:LnLthlufzQll6DTD@bookstore.n2xxh9r.mongodb.net/?retryWrites=true&w=majority&appName=BookStore";
    let client = Client::with_uri_str(mongo_address).await.expect("Failed to connect to MongoDB");
    let db = client.database("book_db");
    let collection = db.collection::<Book>("books");

    let collection_names = db.list_collection_names(None).await.expect("Failed to list collection names");
    if !collection_names.contains(&"books".to_string()) {
        db.create_collection("books", None).await.expect("Failed to create collection");
    }

    let data = web::Data::new(Arc::new(Mutex::new(collection)));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(data.clone())
            .service(fs::Files::new("/", "static").index_file("index.html")) // Serve static files
            .route("/books", web::post().to(create_book))
            .route("/books", web::get().to(get_all_books))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
