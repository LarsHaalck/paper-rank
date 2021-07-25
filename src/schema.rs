use diesel::{self, prelude::*};
use itertools::Itertools;
use rcir;
use rocket::serde::{Deserialize, Serialize};

use crate::DbConn;

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

#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub is_admin: bool
}

pub struct AdminUser(pub User);

#[derive(Serialize, Queryable, Debug, Clone)]
pub struct Item {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub done: bool,
}

#[derive(Queryable, Insertable, Debug, Clone)]
#[table_name = "votes"]
pub struct Vote {
    pub user_id: i32,
    pub item_id: i32,
    pub ordinal: i32,
}

use self::schema::items::dsl::{done as item_done, items as all_items};
use self::schema::users::dsl::{id as user_id, username as users_uname, users as all_users};
use self::schema::votes::dsl::{
    item_id as vote_item_id, ordinal, user_id as vote_user_id, votes as all_votes,
};

#[derive(Deserialize)]
pub struct Ballot {
    pub votes: Vec<i32>,
}

#[derive(FromForm, Insertable)]
#[table_name = "items"]
pub struct ItemData {
    pub title: String,
    pub body: String,
}

#[derive(FromForm)]
pub struct NewUser {
    pub username: String,
}

impl NewUser {
    pub async fn login(self, conn: &DbConn) -> Option<User> {
        conn.run(move |c| {
            all_users
                .filter(users_uname.eq(&self.username))
                .get_result::<User>(c)
                .ok()
        })
        .await
    }
}

impl User {
    pub async fn from_id(id: i32, conn: &DbConn) -> Option<User> {
        conn.run(move |c| {
            all_users
                .filter(user_id.eq(id))
                .get_result::<User>(c)
                .ok()
        })
        .await
    }
}

impl Item {
    pub async fn for_user(uid: i32, conn: &DbConn) -> Vec<(Item, Option<i32>)> {
        conn.run(move |c| {
            all_items
                .left_join(
                    all_votes.on(vote_user_id
                        .eq(&uid)
                        .and(vote_item_id.eq(self::schema::items::id))),
                )
                .filter(item_done.eq(false))
                .order((vote_user_id.desc(), ordinal.asc()))
                .select((self::schema::items::all_columns, ordinal.nullable()))
                .load::<(Item, Option<i32>)>(c)
                .unwrap_or(Vec::new())
        })
        .await
    }
}

impl ItemData {
    pub async fn add(self, conn: &DbConn) -> Option<()> {
        conn.run(move |c| {
            diesel::insert_into(all_items)
                .values(&self)
                .execute(c)
                .ok()?;
            Some(())
        })
        .await
    }
}

impl Vote {
    pub async fn run_election(conn: &DbConn) -> Option<Item> {
        conn.run(move |c| {
            let votes = all_votes
                .inner_join(all_items)
                .filter(item_done.eq(false))
                .order((vote_user_id.asc(), ordinal.asc()))
                .select((vote_user_id, vote_item_id, ordinal))
                .get_results::<Vote>(c)
                .ok()?;

            Vote::election_driver(&votes, &c)
        })
        .await
    }

    pub async fn run_second_election(conn: &DbConn, winner: Option<Item>) -> Option<Item> {
        conn.run(move |c| {
            let winner = winner.as_ref()?;
            let votes = all_votes
                .inner_join(all_items)
                .filter(item_done.eq(false))
                .filter(vote_item_id.ne(winner.id))
                .order((vote_user_id.asc(), ordinal.asc()))
                .select((vote_user_id, vote_item_id, ordinal))
                .get_results::<Vote>(c)
                .ok()?;

            Vote::election_driver(&votes, &c)
        })
        .await
    }

    fn election_driver(votes: &Vec<Vote>, c: &SqliteConnection) -> Option<Item> {
        // the extra collections here are sad.
        let votes: Vec<Vec<_>> = votes
            .into_iter()
            .group_by(|v| v.user_id)
            .into_iter()
            .map(|(_, ballot)| ballot.into_iter().map(|v| v.item_id).collect())
            .collect();

        match rcir::run_election(&votes, rcir::MajorityMode::RemainingMajority).ok()? {
            rcir::ElectionResult::Winner(&iid) => all_items.find(iid).get_result::<Item>(c).ok(),
            rcir::ElectionResult::Tie(iids) => {
                // TODO: maybe pick the oldest one?
                all_items.find(*iids[0]).get_result::<Item>(c).ok()
            }
        }
    }

    pub async fn save_ballot(uid: i32, ballot: Ballot, conn: &DbConn) -> Option<()> {
        conn.run(move |c| {
            diesel::delete(all_votes.filter(vote_user_id.eq(&uid)))
                .execute(c)
                .ok()?;

            for (i, iid) in ballot.votes.into_iter().enumerate() {
                diesel::insert_into(all_votes)
                    .values(Vote {
                        user_id: uid,
                        item_id: iid,
                        ordinal: i as i32,
                    })
                    .execute(c)
                    .ok()?;
            }
            Some(())
        })
        .await
    }
}
