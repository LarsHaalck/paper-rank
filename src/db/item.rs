use super::*;

#[derive(Serialize, Queryable, Debug, Clone)]
pub struct Item {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub done: bool,
}

#[derive(FromForm, Insertable)]
#[table_name = "items"]
pub struct ItemData {
    pub title: String,
    pub body: String,
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
    pub async fn add(self, conn: &DbConn) -> Result<(), Error> {
        conn.run(move |c| {
            diesel::insert_into(all_items)
                .values(&self)
                .execute(c)
                .map_err(|_| Error::new(ErrorKind::Other, "Failed inserting new item into db."))?;
            Ok(())
        })
        .await
    }
}
