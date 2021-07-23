#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_sync_db_pools;

mod schema;

use comrak::{markdown_to_html, ComrakOptions};
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FlashMessage, FromRequest, Request};
use rocket::response::{Flash, Redirect};
use rocket::serde::{json::Json, Serialize};
use rocket_dyn_templates::Template;
use schema::{Ballot, Item, ItemData, NewUser, Vote};

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    winner: Option<Item>,
    second: Option<Item>,
    items: Vec<(Item, Option<i32>)>,
    flash: Option<(String, String)>,
}

impl Context {
    pub async fn new(conn: &DbConn, flash: Option<(String, String)>) -> Context {
        Context {
            winner: Vote::run_election(conn).await,
            second: None,
            items: Vec::new(), // not used if not logged in
            flash,
        }
    }

    pub async fn for_user(user: Auth, conn: &DbConn, flash: Option<(String, String)>) -> Context {
        let winner = Vote::run_election(conn).await;
        let second = Vote::run_second_election(conn, winner.clone()).await;
        Context {
            winner,
            second,
            items: Item::for_user(user.0, conn).await,
            flash,
        }
    }
}

#[derive(Debug)]
struct Auth(i32);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Auth, Self::Error> {
        request
            .cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(Auth)
            .or_forward(())
    }
}

#[post("/login", data = "<input>")]
async fn login(
    jar: &CookieJar<'_>,
    input: Form<NewUser>,
    conn: DbConn,
) -> Result<Redirect, Flash<Redirect>> {
    let user = input.into_inner();
    if user.username.is_empty() {
        Err(Flash::error(
            Redirect::to(uri!(index)),
            "Username must not be empty",
        ))
    } else {
        let u = user.login(&conn).await;
        match u {
            Some(x) => {
                jar.add_private(Cookie::new("user_id", x.id.to_string()));
                Ok(Redirect::to(uri!(votes)))
            }
            None => Err(Flash::error(
                Redirect::to(uri!(index)),
                "Username does not exist.",
            )),
        }
    }
}

#[post("/vote", data = "<ballot>")]
async fn vote(ballot: Json<Ballot>, user: Auth, conn: DbConn) -> Status {
    let res = Vote::save_ballot(user.0, ballot.into_inner(), &conn).await;
    match res {
        Some(_) => Status::Ok,
        None => Status::NotAcceptable,
    }
}

#[post("/preview", data = "<markdown>")]
async fn preview(markdown: &str, _user: Auth, _conn: DbConn) -> String {
    markdown_to_html(markdown, &ComrakOptions::default())
}

#[post("/new", data = "<item>")]
async fn new_item(item: Form<ItemData>, _user: Auth, conn: DbConn) -> Flash<Redirect> {
    let mut item_data = item.into_inner();
    item_data.body = markdown_to_html(&item_data.body, &ComrakOptions::default());
    let res = item_data.add(&conn).await;
    match res {
        Some(_) => Flash::success(Redirect::to(uri!(index)), "Added item to db"),
        None => Flash::error(Redirect::to(uri!(new)), "Failed to insert item into db"),
    }
}

#[get("/new")]
async fn new(flash: Option<FlashMessage<'_>>, user: Auth, conn: DbConn) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render("new", Context::for_user(user, &conn, flash).await)
}

#[get("/")]
async fn votes(flash: Option<FlashMessage<'_>>, user: Auth, conn: DbConn) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render("vote", Context::for_user(user, &conn, flash).await)
}

#[get("/", rank = 2)]
async fn index(flash: Option<FlashMessage<'_>>, conn: DbConn) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render("index", Context::new(&conn, flash).await)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .mount(
            "/",
            routes![index, login, votes, vote, new, preview, new_item],
        )
}
