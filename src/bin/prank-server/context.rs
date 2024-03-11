use prank::MailConfig;
use rocket::serde::Serialize;

use crate::{DbConn, Item, User, Vote};

use std::collections::HashMap;
pub type Empty = HashMap<i32, i32>;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    username: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct VoteContext {
    next: Option<Item>,
    winner: Option<Item>,
    second: Option<Item>,
    items: Vec<(Item, Option<i32>)>,
    context: Context,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ItemContext {
    items: Vec<Item>,
    context: Context,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct UserContext {
    context: Context,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct EditContext {
    item: Option<Item>,
    context: Context,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct MailContext {
    item: Option<Item>,
    config: MailConfig,
    context: Context,
}

impl Context {
    fn new(flash: Option<(String, String)>) -> Context {
        Context {
            flash,
            username: None,
        }
    }

    fn for_user(user: &User, flash: Option<(String, String)>) -> Context {
        Context {
            flash,
            username: Some(user.username.clone()),
        }
    }
}

impl VoteContext {
    pub async fn new(conn: &DbConn, flash: Option<(String, String)>) -> VoteContext {
        VoteContext {
            next: Item::get_decided(conn).await,
            winner: Vote::run_election(conn).await,
            second: None,
            items: Vec::new(),
            context: Context::new(flash),
        }
    }

    pub async fn for_user(
        user: &User,
        conn: &DbConn,
        flash: Option<(String, String)>,
    ) -> VoteContext {
        let winner = Vote::run_election(conn).await;
        let second = Vote::run_second_election(conn, winner.clone()).await;
        VoteContext {
            next: Item::get_decided(conn).await,
            winner,
            second,
            items: Item::get_user_and_votes(user.id, conn).await,
            context: Context::for_user(user, flash),
        }
    }
}

impl UserContext {
    pub async fn new(_conn: &DbConn, flash: Option<(String, String)>) -> UserContext {
        UserContext {
            context: Context::new(flash),
        }
    }

    pub async fn for_user(user: &User, flash: Option<(String, String)>) -> UserContext {
        UserContext {
            context: Context::for_user(user, flash),
        }
    }
}

impl ItemContext {
    pub async fn for_user_history(
        user: &User,
        conn: &DbConn,
        flash: Option<(String, String)>,
    ) -> ItemContext {
        ItemContext {
            items: Item::get_history(conn).await,
            context: Context::for_user(user, flash),
        }
    }
    pub async fn for_user_full(
        user: &User,
        conn: &DbConn,
        flash: Option<(String, String)>,
    ) -> ItemContext {
        ItemContext {
            items: Item::get_all(conn).await,
            context: Context::for_user(user, flash),
        }
    }
}

impl EditContext {
    pub async fn for_user(
        id: i32,
        user: &User,
        conn: &DbConn,
        flash: Option<(String, String)>,
    ) -> EditContext {
        EditContext {
            item: Item::from_id(id, conn).await,
            context: Context::for_user(user, flash),
        }
    }
}

impl MailContext {
    pub async fn new(
        id: i32,
        user: &User,
        config: &MailConfig,
        conn: &DbConn,
        flash: Option<(String, String)>,
    ) -> MailContext {
        MailContext {
            item: Item::from_id(id, conn).await,
            config: config.clone(),
            context: Context::for_user(user, flash),
        }
    }
}
