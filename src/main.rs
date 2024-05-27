use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use std::sync::{Arc, Mutex};
use mongodb::{Client, Collection};
use futures::stream::TryStreamExt;
use serde_json::json;
use mongodb::{bson::doc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
#[derive(Debug)]

struct Book {
    #[serde(rename= "_id")]
    #[serde(default)]
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

    let new_book_id = Uuid::new_v4().to_string(); // Generate UUID for the id field

    let new_book = Book {
        id: new_book_id,
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



async fn get_all_books(db_pool: web::Data<Arc<Mutex<Collection<Book>>>>) -> impl Responder {
    let collection = match db_pool.lock() {
      Ok(collection) => collection,
      Err(err) => {
        eprintln!("Failed to lock the database collection: {:?}", err);
        return HttpResponse::InternalServerError().body(format!("Failed to lock the database collection: {}", err));
      }
    };
  
    let cursor = match collection.find(None, None).await {
      Ok(cursor) => cursor,
      Err(err) => {
        eprintln!("Failed to query books: {:?}", err);
        return HttpResponse::InternalServerError().body(format!("Failed to query books: {}", err));
      }
    };
  
    let books: Vec<Book> = match cursor.try_collect().await {
      Ok(books) => books,
      Err(err) => {
        eprintln!("Failed to collect books: {:?}", err);
        return HttpResponse::InternalServerError().body(format!("Failed to collect books: {}", err));
      }
    };
  
     //Print the first book for inspection (optional)
     println!("{:?}", books[0]);
  
    HttpResponse::Ok().json(books)
  }
  

async fn get_book_by_id(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let collection = db_pool.lock().unwrap();
    let id = path.into_inner();

    match collection.find_one(doc! { "id": id.to_string() }, None).await {
        Ok(Some(book)) => HttpResponse::Ok().json(BookResponse::from(book)),
        Ok(None) => HttpResponse::NotFound().body("Book not found"),
        Err(err) => {
            eprintln!("Failed to get book by ID: {:?}", err);
            HttpResponse::InternalServerError().body(format!("Failed to get book by ID: {}", err))
        }
    }
}

async fn update_book(
    db_pool: web::Data<Arc<Mutex<Collection<Book>>>>,
    path: web::Path<Uuid>,
    updated_book: web::Json<NewBook>,
) -> impl Responder {
    let collection = db_pool.lock().unwrap();
    let id = path.into_inner();

    let update_doc = doc! {
        "$set": {
            "title": &updated_book.title,
            "author": &updated_book.author,
            "published_year": &updated_book.published_year,
        }
    };

    match collection.update_one(doc! { "id": id.to_string() }, update_doc, None).await {
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

    match collection.delete_one(doc! { "id": id.to_string() }, None).await {
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
    let client = Client::with_uri_str(mongo_address).await.unwrap();
    let db = client.database("book_db");
    let collection = db.collection::<Book>("books");

    let data = web::Data::new(Arc::new(Mutex::new(collection)));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/books", web::post().to(create_book))
            .route("/books", web::get().to(get_all_books))
            .route("/books/{id}", web::get().to(get_book_by_id))
            .route("/books/{id}", web::put().to(update_book))
            .route("/books/{id}", web::delete().to(delete_book))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
