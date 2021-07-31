use diesel::{self, prelude::*};
use rocket::serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};

pub mod item;
pub mod user;
pub mod vote;

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);

// use crate::rocket_sync_db_pools::Poolable;
// use rocket::{Rocket, Build};
// impl DbConn {
//     pub fn establish(rocket: &Rocket<Build>) -> DbConn {
//         let manager = diesel::SqliteConnection::pool("sqlite_database", rocket).unwrap();
//         let conn = manager.get().unwrap();
//         // let conn = conn.new();
//         DbConn(conn)

//     }
// }


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
            body -> Text,
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

use self::schema::items;
use self::schema::users;
use self::schema::votes;

use self::schema::items::dsl::{discussed_on, items as all_items};
use self::schema::users::dsl::{
    id as user_id, is_approved as user_approved, password as user_password,
    username as user_username, users as all_users,
};
use self::schema::votes::dsl::{
    item_id as vote_item_id, ordinal, user_id as vote_user_id, votes as all_votes,
};
