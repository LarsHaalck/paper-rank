use diesel::{self, prelude::*};
use rocket::serde::{Deserialize, Serialize};

use crate::DbConn;

mod user;
mod item;
mod vote;

pub use user::{User, AdminUser, NewUser};
pub use vote::{Vote, Ballot};
pub use item::{Item, ItemData};

mod schema {
    table! {
        users {
            id -> Integer,
            username -> Text,
            is_admin -> Bool,
        }
    }

    table! {
        items {
            id -> Integer,
            title -> Text,
            body -> Text,
            done -> Bool,
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
use self::schema::votes;

use self::schema::items::dsl::{done as item_done, items as all_items};
use self::schema::users::dsl::{id as user_id, username as users_uname, users as all_users};
use self::schema::votes::dsl::{
    item_id as vote_item_id, ordinal, user_id as vote_user_id, votes as all_votes,
};

