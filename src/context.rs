use rocket::serde::Serialize;

use crate::db::{Item, User, Vote};
use crate::DbConn;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    username: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct VoteContext {
    winner: Option<Item>,
    second: Option<Item>,
    items: Vec<(Item, Option<i32>)>,
    context: Context,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct HistoryContext {
    items: Vec<Item>,
    context: Context,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct UserContext {
    context: Context,
}

impl VoteContext {
    pub async fn new(conn: &DbConn, flash: Option<(String, String)>) -> VoteContext {
        VoteContext {
            winner: Vote::run_election(conn).await,
            second: None,
            items: Vec::new(),
            context: Context {
                flash,
                username: None,
            },
        }
    }

    pub async fn for_user(
        user: User,
        conn: &DbConn,
        flash: Option<(String, String)>,
    ) -> VoteContext {
        let winner = Vote::run_election(conn).await;
        let second = Vote::run_second_election(conn, winner.clone()).await;
        VoteContext {
            winner,
            second,
            items: Item::for_user(user.id, conn).await,
            context: Context {
                flash,
                username: Some(user.username),
            },
        }
    }
}

impl UserContext {
    pub async fn new(_conn: &DbConn, flash: Option<(String, String)>) -> UserContext {
        UserContext {
            context: Context {
                flash,
                username: None,
            },
        }
    }

    pub async fn for_user(user: User, flash: Option<(String, String)>) -> UserContext {
        UserContext {
            context: Context {
                flash,
                username: Some(user.username),
            },
        }
    }
}

impl HistoryContext {
    pub async fn for_user(
        user: User,
        conn: &DbConn,
        flash: Option<(String, String)>,
    ) -> HistoryContext {
        HistoryContext {
            items: Item::get_history(conn).await,
            context: Context {
                flash,
                username: Some(user.username),
            },
        }
    }
}
