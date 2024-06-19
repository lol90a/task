use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;

// Define a struct to represent a Book, with serialization and deserialization
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Book {
    #[serde(rename = "_id")] // Rename the field to "_id" in MongoDB
    #[serde(default)] // Provide a default value if not present
    #[serde(skip_serializing_if = "Option::is_none")] // Skip if None
    pub id: Option<ObjectId>,
    pub title: String,
    pub author: String,
    pub published_year: i32,
}

// Define a struct for creating new books, without an ID
#[derive(Serialize, Deserialize)]
pub struct NewBook {
    pub title: String,
    pub author: String,
    pub published_year: i32,
}

// Define a struct for the response format of a book
#[derive(Serialize)]
pub struct BookResponse {
    pub id: String,
    pub title: String,
    pub author: String,
    pub published_year: i32,
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
