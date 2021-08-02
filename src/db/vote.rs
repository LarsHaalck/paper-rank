use super::*;

use item::Item;
use itertools::Itertools;
use rcir;

#[derive(Queryable, Insertable, Debug, Clone)]
#[table_name = "votes"]
pub struct Vote {
    pub user_id: i32,
    pub item_id: i32,
    pub ordinal: i32,
}

#[derive(Deserialize)]
pub struct Ballot {
    pub votes: Vec<i32>,
}

impl Vote {
    pub async fn run_election(conn: &DbConn) -> Option<Item> {
        conn.run(move |c| {
            let votes = all_votes
                .inner_join(all_items)
                .filter(item_discussed_on.is_null())
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
                .filter(item_discussed_on.is_null())
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

    pub async fn save_ballot(uid: i32, ballot: Ballot, conn: &DbConn) -> Result<(), Error> {
        conn.run(move |c| {
            diesel::delete(all_votes.filter(vote_user_id.eq(&uid)))
                .execute(c)
                .map_err(|_| Error::new(ErrorKind::Other, "Faile to write save ballow."))?;

            for (i, iid) in ballot.votes.into_iter().enumerate() {
                diesel::insert_into(all_votes)
                    .values(Vote {
                        user_id: uid,
                        item_id: iid,
                        ordinal: i as i32,
                    })
                    .execute(c)
                    .map_err(|_| Error::new(ErrorKind::Other, "Faile to write save ballow."))?;
            }
            Ok(())
        })
        .await
    }
}
