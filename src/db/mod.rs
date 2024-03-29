use anyhow::{Context, Result};
use diesel::{self, prelude::*};
use rocket::serde::{Deserialize, Serialize};

pub mod item;
pub mod user;
pub mod vote;

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);

//////////////////////////////
// internal db stuff
//////////////////////////////
mod schema {
    table! {
        users {
            id -> Integer,
            username -> Text,
            password -> Text,
            is_admin -> Bool,
            is_approved -> Bool,
        }
    }

    table! {
        items {
            id -> Integer,
            title -> Text,
            html -> Text,
            markdown -> Text,
            discussed_on -> Nullable<Date>,
        }
    }

    table! {
        votes (user_id, item_id) {
            user_id -> Integer,
            item_id -> Integer,
            ordinal -> Integer,
        }
    }

    joinable!(votes -> items (item_id));
    joinable!(votes -> users (user_id));
    allow_tables_to_appear_in_same_query!(users, items, votes);
}

use self::schema::items::dsl::{
    discussed_on as item_discussed_on, id as item_id, items as all_items,
};
use self::schema::users::dsl::{
    id as user_id, is_admin as user_admin, is_approved as user_approved, password as user_password,
    username as user_username, users as all_users,
};
use self::schema::votes::dsl::{
    item_id as vote_item_id, ordinal, user_id as vote_user_id, votes as all_votes,
};
