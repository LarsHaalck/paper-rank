#[macro_use]
extern crate rocket;

mod context;
mod markdown;

use rocket::figment::value::magic::RelativePathBuf;
use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::json::Json;
use rocket::Request;
use rocket_dyn_templates::Template;

use context::{EditContext, Empty, ItemContext, UserContext, VoteContext};
use markdown::markdown_to_html;
use prank::item::{ChangeItemData, Item, NewItemData};
use prank::user::{AdminUser, NewPassword, NewUser, User};
use prank::vote::{Ballot, Vote};
use prank::DbConn;

///////////////////////////////////////////////////////////////////////////////
// Post Routes
///////////////////////////////////////////////////////////////////////////////
#[post("/login", data = "<input>")]
async fn login(
    jar: &CookieJar<'_>,
    input: Form<NewUser>,
    conn: DbConn,
) -> Result<Redirect, Flash<Redirect>> {
    let user = input.into_inner();
    if user.username.is_empty() {
        Err(Flash::error(
            Redirect::to(uri!(user)),
            "Username must not be empty",
        ))
    } else {
        let u = user.login(&conn).await;
        match u {
            Ok(x) => {
                jar.add_private(Cookie::new("user_id", x.id.to_string()));
                Ok(Redirect::to(uri!(index_user)))
            }
            Err(e) => Err(Flash::error(Redirect::to(uri!(user)), e.to_string())),
        }
    }
}

#[post("/logout")]
fn logout(jar: &CookieJar<'_>) -> Flash<Redirect> {
    jar.remove_private(Cookie::named("user_id"));
    Flash::success(Redirect::to(uri!(user)), "Successfully logged out.")
}

#[post("/register", data = "<input>")]
async fn register(input: Form<NewUser>, conn: DbConn) -> Flash<Redirect> {
    let user = input.into_inner();
    if user.username.is_empty() {
        Flash::error(Redirect::to(uri!(user)), "Username must not be empty")
    } else {
        let res = user.register(&conn).await;
        match res {
            Ok(_) => Flash::success(
                Redirect::to(uri!(user)),
                "Registered new user which must be approved by the admin.",
            ),
            Err(e) => Flash::error(Redirect::to(uri!(user)), e.to_string()),
        }
    }
}

#[post("/change_password", data = "<input>")]
async fn change_password(input: Form<NewPassword>, user: &User, conn: DbConn) -> Flash<Redirect> {
    let new_password = input.into_inner();
    let change = User::change_password(&user, new_password, &conn).await;
    match change {
        Ok(_) => Flash::success(Redirect::to(uri!(user)), "Sucessfully changed password"),
        Err(e) => Flash::error(Redirect::to(uri!(user)), e.to_string()),
    }
}

#[post("/vote", data = "<ballot>")]
async fn vote(ballot: Json<Ballot>, user: &User, conn: DbConn) -> Status {
    let res = Vote::save_ballot(user.id, ballot.into_inner(), &conn).await;
    match res {
        Ok(_) => Status::Ok,
        Err(_) => Status::NotAcceptable,
    }
}

#[post("/preview", data = "<markdown>")]
async fn preview(markdown: &str, _user: &User, _conn: DbConn) -> Result<String, std::io::Error> {
    markdown_to_html(markdown)
}

#[post("/new_item", data = "<item>")]
async fn add_new_item(item: Form<NewItemData>, _user: &User, conn: DbConn) -> Flash<Redirect> {
    let mut item_data = item.into_inner();
    item_data.html = match markdown_to_html(&item_data.markdown) {
        Ok(html) => html,
        Err(e) => return Flash::error(Redirect::to(uri!(add_new_item)), e.to_string()),
    };

    let res = Item::add(item_data, &conn).await;
    match res {
        Ok(_) => Flash::success(Redirect::to(uri!(index)), "Added item to db"),
        Err(e) => Flash::error(Redirect::to(uri!(add_new_item)), e.to_string()),
    }
}

#[post("/update_item", data = "<item>")]
async fn update_item(item: Form<ChangeItemData>, _user: &User, conn: DbConn) -> Flash<Redirect> {
    let mut item_data = item.into_inner();
    item_data.html = match markdown_to_html(&item_data.markdown) {
        Ok(html) => html,
        Err(e) => return Flash::error(Redirect::to(uri!(add_new_item)), e.to_string()),
    };

    let res = Item::update(item_data, &conn).await;
    match res {
        Ok(_) => Flash::success(Redirect::to(uri!(index)), "Changed item in db"),
        Err(e) => Flash::error(Redirect::to(uri!(add_new_item)), e.to_string()),
    }
}

///////////////////////////////////////////////////////////////////////////////
// GET Routes
///////////////////////////////////////////////////////////////////////////////
#[get("/history")]
async fn history(flash: Option<FlashMessage<'_>>, user: &User, conn: DbConn) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "history",
        ItemContext::for_user_history(user, &conn, flash).await,
    )
}

#[get("/edit?<id>")]
async fn edit_id(
    id: i32,
    flash: Option<FlashMessage<'_>>,
    admin: AdminUser<'_>,
    conn: DbConn,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "item",
        EditContext::for_user(id, admin.user, &conn, flash).await,
    )
}

#[get("/show")]
async fn edit(flash: Option<FlashMessage<'_>>, admin: AdminUser<'_>, conn: DbConn) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "show",
        ItemContext::for_user_full(admin.user, &conn, flash).await,
    )
}

#[get("/user")]
async fn user_user(flash: Option<FlashMessage<'_>>, user: &User, _conn: DbConn) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render("user", UserContext::for_user(user, flash).await)
}

#[get("/user", rank = 2)]
async fn user(flash: Option<FlashMessage<'_>>, conn: DbConn) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render("login", UserContext::new(&conn, flash).await)
}

#[get("/new_item")]
async fn new_item(flash: Option<FlashMessage<'_>>, user: &User, _conn: DbConn) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render("item", UserContext::for_user(user, flash).await)
}

#[get("/")]
async fn index_user(flash: Option<FlashMessage<'_>>, user: &User, conn: DbConn) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render("vote", VoteContext::for_user(user, &conn, flash).await)
}

#[get("/", rank = 2)]
async fn index(flash: Option<FlashMessage<'_>>, conn: DbConn) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render("index", VoteContext::new(&conn, flash).await)
}

#[catch(404)]
fn not_found(_req: &Request) -> Template {
    Template::render("404", Empty::new())
}

#[launch]
fn rocket() -> _ {
    let rocket = rocket::build()
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .register("/", catchers![not_found])
        .mount(
            "/",
            routes![index, index_user, new_item, user, user_user, history, edit, edit_id],
        )
        .mount(
            "/",
            routes![
                login,
                logout,
                register,
                change_password,
                vote,
                preview,
                add_new_item,
                update_item
            ],
        );

    let static_dir = rocket
        .figment()
        .extract_inner::<RelativePathBuf>("static_dir")
        .map(|path| path.relative())
        .unwrap();
    rocket.mount("/", FileServer::from(static_dir))
}
