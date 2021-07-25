use rocket::serde::Serialize;

use crate::DbConn;
use crate::schema::{Vote, Item, User};

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Context {
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

    pub async fn for_user(user: User, conn: &DbConn, flash: Option<(String, String)>) -> Context {
        let winner = Vote::run_election(conn).await;
        let second = Vote::run_second_election(conn, winner.clone()).await;
        Context {
            winner,
            second,
            items: Item::for_user(user.id, conn).await,
            flash,
        }
    }
}
