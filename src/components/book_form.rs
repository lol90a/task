use yew::prelude::*;
use web_sys::HtmlInputElement;
use yewtil::NeqAssign;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Book {
    pub id: Option<u32>,
    pub title: String,
    pub author: String,
    pub published_year: i32,
}

pub struct BookForm {
    link: ComponentLink<Self>,
    title: String,
    author: String,
    published_year: String,
}

pub enum Msg {
    UpdateTitle(String),
    UpdateAuthor(String),
    UpdateYear(String),
    Submit,
}

impl Component for BookForm {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            title: String::new(),
            author: String::new(),
            published_year: String::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateTitle(title) => self.title.neq_assign(title),
            Msg::UpdateAuthor(author) => self.author.neq_assign(author),
            Msg::UpdateYear(year) => self.published_year.neq_assign(year),
            Msg::Submit => {
                let window = web_sys::window().expect("should have a window in this context");
                let storage = window.local_storage().expect("no local storage").expect("should have local storage");

                let mut books: Vec<Book> = storage
                    .get_item("books")
                    .ok()
                    .flatten()
                    .and_then(|data| serde_json::from_str(&data).ok())
                    .unwrap_or_else(Vec::new);

                books.push(Book {
                    id: None,
                    title: self.title.clone(),
                    author: self.author.clone(),
                    published_year: self.published_year.parse().unwrap(),
                });

                storage.set_item("books", &serde_json::to_string(&books).unwrap()).expect("should set book data");

                true
            }
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="container">
                <h1>{ "Add a New Book" }</h1>
                <form onsubmit=self.link.callback(|e: FocusEvent| {
                    e.prevent_default();
                    Msg::Submit
                })>
                    <label for="title">{ "Title" }</label>
                    <input
                        type="text"
                        id="title"
                        value=&self.title
                        oninput=self.link.callback(|e: InputData| Msg::UpdateTitle(e.value))
                        required=true
                    />
                    <label for="author">{ "Author" }</label>
                    <input
                        type="text"
                        id="author"
                        value=&self.author
                        oninput=self.link.callback(|e: InputData| Msg::UpdateAuthor(e.value))
                        required=true
                    />
                    <label for="published_year">{ "Published Year" }</label>
                    <input
                        type="text"
                        id="published_year"
                        value=&self.published_year
                        oninput=self.link.callback(|e: InputData| Msg::UpdateYear(e.value))
                        required=true
                    />
                    <input type="submit" value="Add Book" />
                </form>
            </div>
        }
    }
}
