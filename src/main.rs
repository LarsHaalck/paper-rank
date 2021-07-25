#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_sync_db_pools;

mod schema;
mod markdown;
mod context;
mod auth;

use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::{json::Json};
use rocket_dyn_templates::Template;
use rocket::fs::{FileServer, relative};

use schema::{Ballot, ItemData, NewUser, Vote, User, AdminUser};
use markdown::markdown_to_html;
use context::Context;

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);

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
async fn vote(ballot: Json<Ballot>, user: User, conn: DbConn) -> Status {
    let res = Vote::save_ballot(user.id, ballot.into_inner(), &conn).await;
    match res {
        Some(_) => Status::Ok,
        None => Status::NotAcceptable,
    }
}

#[post("/preview", data = "<markdown>")]
async fn preview(markdown: &str, _user: User, _conn: DbConn) -> String {
    markdown_to_html(markdown)
}

#[post("/new", data = "<item>")]
async fn new_item(item: Form<ItemData>, _user: User, conn: DbConn) -> Flash<Redirect> {
    let mut item_data = item.into_inner();
    item_data.body = markdown_to_html(&item_data.body);
    let res = item_data.add(&conn).await;
    match res {
        Some(_) => Flash::success(Redirect::to(uri!(index)), "Added item to db"),
        None => Flash::error(Redirect::to(uri!(new)), "Failed to insert item into db"),
    }
}

#[get("/new")]
async fn new(flash: Option<FlashMessage<'_>>, user: User, conn: DbConn) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render("new", Context::for_user(user, &conn, flash).await)
}

#[get("/")]
async fn votes(flash: Option<FlashMessage<'_>>, user: User, conn: DbConn) -> Template {
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
        .mount("/", FileServer::from(relative!("static")))
}
