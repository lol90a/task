use yew::prelude::*;
use super::book_form::Book;

pub struct BookList {
    books: Vec<Book>,
}

impl Component for BookList {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        let window = web_sys::window().expect("should have a window in this context");
        let storage = window.local_storage().expect("no local storage").expect("should have local storage");

        let books: Vec<Book> = storage
            .get_item("books")
            .ok()
            .flatten()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_else(Vec::new);

        Self { books }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="container">
                <h1>{ "Book List" }</h1>
                <div class="books">
                    { for self.books.iter().map(|book| self.view_book(book)) }
                </div>
            </div>
        }
    }
}

impl BookList {
    fn view_book(&self, book: &Book) -> Html {
        html! {
            <div class="book">
                <h2>{ &book.title }</h2>
                <p>{ format!("Author: {}", &book.author) }</p>
                <p>{ format!("Published Year: {}", &book.published_year) }</p>
            </div>
        }
    }
}
