use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::{Client, Collection};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};
// use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Book {
    #[serde(rename = "_id")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
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
            id: book.id.unwrap_or_else(ObjectId::new).to_string(),
            title: book.title,
            author: book.author,
            published_year: book.published_year,
        }
    }
}

async fn index() -> impl Responder {
    HttpResponse::BadRequest().body("please specify the operation")
}

async fn create_book(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>,
    book: web::Json<NewBook>,
) -> impl Responder {
    if book.title.is_empty() || book.author.is_empty() || book.published_year <= 0 {
        return HttpResponse::BadRequest().body("Invalid input data");
    }

    let new_book = Book {
        id: Some(ObjectId::new()),
        title: book.title.clone(),
        author: book.author.clone(),
        published_year: book.published_year,
    };

    let collection = db_pool.lock().unwrap();
    match collection.insert_one(new_book.clone(), None).await {
        Ok(_) => HttpResponse::Created().json(new_book),
        Err(err) => {
            HttpResponse::InternalServerError().body(format!("Failed to insert book: {}", err))
        }
    }
}

async fn delete_all_books(db_pool: web::Data<Arc<Mutex<Collection<Book>>>>) -> impl Responder {
    let collection = match db_pool.lock() {
        Ok(collection) => collection,
        Err(err) => {
            eprintln!("Failed to lock the database collection: {:?}", err);
            return HttpResponse::InternalServerError()
                .body(format!("Failed to lock the database collection: {}", err));
        }
    };

    match collection.delete_many(doc! {}, None).await {
        Ok(result) => HttpResponse::Ok().body(format!("Deleted {} books", result.deleted_count)),
        Err(err) => {
            eprintln!("Failed to delete books: {:?}", err);
            HttpResponse::InternalServerError().body(format!("Failed to delete books: {}", err))
        }
    }
}

async fn get_all_books(db_pool: web::Data<Arc<Mutex<Collection<Book>>>>) -> impl Responder {
    let collection = match db_pool.lock() {
        Ok(collection) => collection,
        Err(err) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to lock the database collection: {}", err));
        }
    };

    let cursor = match collection.find(None, None).await {
        Ok(cursor) => cursor,
        Err(err) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to query books: {}", err));
        }
    };

    let books: Vec<Book> = match cursor.try_collect().await {
        Ok(books) => books,
        Err(err) => {
            eprintln!("Failed to collect books: {:?}", err);
            return HttpResponse::InternalServerError()
                .body(format!("Failed to collect books: {}", err));
        }
    };

    let book_responses: Vec<BookResponse> = books.into_iter().map(BookResponse::from).collect();
    HttpResponse::Ok().json(book_responses)
}

async fn get_book_by_id(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>,
    path: web::Path<String>,
) -> impl Responder {
    let collection = db_pool.lock().unwrap();
    let id = path.into_inner();

    match ObjectId::parse_str(&id) {
        Ok(object_id) => match collection.find_one(doc! { "_id": object_id }, None).await {
            Ok(Some(book)) => HttpResponse::Ok().json(BookResponse::from(book)),
            Ok(None) => HttpResponse::NotFound().body("Book not found"),
            Err(err) => {
                eprintln!("Failed to get book by ID: {:?}", err);
                HttpResponse::InternalServerError()
                    .body(format!("Failed to get book by ID: {}", err))
            }
        },
        Err(err) => {
            eprintln!("Invalid ObjectId format: {:?}", err);
            HttpResponse::BadRequest().body("Invalid ObjectId format")
        }
    }
}

async fn update_book(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>,
    path: web::Path<String>,
    updated_book: web::Json<NewBook>,
) -> impl Responder {
    let collection = db_pool.lock().unwrap();
    let id_str = path.into_inner();

    match ObjectId::parse_str(&id_str) {
        Ok(object_id) => {
            let update_doc = doc! {
                "$set": {
                    "title": &updated_book.title,
                    "author": &updated_book.author,
                    "published_year": &updated_book.published_year,
                }
            };

            match collection
                .update_one(doc! { "_id": object_id }, update_doc, None)
                .await
            {
                Ok(result) => {
                    if result.matched_count > 0 {
                        HttpResponse::Ok().json(json!({ "message": "Book updated successfully" }))
                    } else {
                        HttpResponse::NotFound().body("Book not found")
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
            HttpResponse::BadRequest().body("Invalid ObjectId format")
        }
    }
}

async fn delete_book(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>,
    path: web::Path<String>,
) -> impl Responder {
    let collection = db_pool.lock().unwrap();
    let id = path.into_inner();

    match ObjectId::parse_str(&id) {
        Ok(object_id) => match collection.delete_one(doc! { "_id": object_id }, None).await {
            Ok(result) => {
                if result.deleted_count > 0 {
                    HttpResponse::Ok().json(json!({ "message": "Book deleted successfully" }))
                } else {
                    HttpResponse::NotFound().body("Book not found")
                }
            }
            Err(err) => {
                eprintln!("Failed to delete book: {:?}", err);
                HttpResponse::InternalServerError().body(format!("Failed to delete book: {}", err))
            }
        },
        Err(err) => {
            eprintln!("Invalid ObjectId format: {:?}", err);
            HttpResponse::BadRequest().body("Invalid ObjectId format")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_address = "mongodb+srv://aliadel:LnLthlufzQll6DTD@bookstore.n2xxh9r.mongodb.net/?retryWrites=true&w=majority&appName=BookStore";
    let client = Client::with_uri_str(mongo_address).await.unwrap();
    let db = client.database("book_db");
    let collection = db.collection::<Book>("books");

    let data = web::Data::new(Arc::new(Mutex::new(collection)));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/books", web::get().to(index))
            .route("/books/create", web::post().to(create_book))
            .route("/books/getall", web::get().to(get_all_books))
            .route("/books/get/{id}", web::get().to(get_book_by_id))
            .route("/books/update/{id}", web::put().to(update_book))
            .route("/books/deleteall", web::get().to(delete_all_books))
            .route("/books/delete/{id}", web::delete().to(delete_book))
            .service(Files::new("/", "src/HTML").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

