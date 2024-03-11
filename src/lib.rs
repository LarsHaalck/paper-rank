#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_sync_db_pools;
#[macro_use]
extern crate rocket;

mod db;

pub mod mail;

pub use db::item;
pub use db::user;
pub use db::vote;
pub use db::DbConn;

use rocket::serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MailConfig {
    email_from: String,
    email_to: String,
    email_comment: Option<String>,
    email_username: String,
    email_server: String,
}
