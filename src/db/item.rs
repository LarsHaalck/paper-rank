use super::*;

use chrono::{NaiveDate, Utc};

#[derive(Serialize, Queryable, Debug, Clone)]
pub struct Item {
    pub id: i32,
    pub title: String,
    pub html: String,
    pub markdown: String,
    pub discussed_on: Option<NaiveDate>,
}

#[derive(FromForm, Insertable)]
#[diesel(table_name = self::schema::items)]
pub struct NewItemData {
    pub title: String,
    pub html: String,
    pub markdown: String,
}

#[derive(FromForm)]
pub struct ChangeItemData {
    pub id: i32,
    pub title: String,
    pub html: String,
    pub markdown: String,
    pub discussed_on: String,
}

impl Item {
    pub async fn get_user_and_votes(uid: i32, conn: &DbConn) -> Vec<(Item, Option<i32>)> {
        conn.run(move |c| {
            all_items
                .left_join(
                    all_votes.on(vote_user_id
                        .eq(&uid)
                        .and(vote_item_id.eq(self::schema::items::id))),
                )
                .filter(item_discussed_on.is_null())
                .order((vote_user_id.desc(), ordinal.asc()))
                .select((self::schema::items::all_columns, ordinal.nullable()))
                .load::<(Item, Option<i32>)>(c)
                .unwrap_or(Vec::new())
        })
        .await
    }

    pub async fn get_decided(conn: &DbConn) -> Option<Item> {
        conn.run(move |c| {
            let item = all_items
                .filter(item_discussed_on.is_not_null())
                .order(item_discussed_on.desc())
                .limit(1)
                .get_result::<Item>(c)
                .ok()?;

            let item_date = item.discussed_on?;
            let today: NaiveDate = Utc::today().naive_utc();
            if today <= item_date {
                return Some(item);
            }
            return None;
        })
        .await
    }

    pub async fn get_history(conn: &DbConn) -> Vec<Item> {
        conn.run(move |c| {
            all_items
                .filter(item_discussed_on.is_not_null())
                .order(item_discussed_on.desc())
                .load::<Item>(c)
                .unwrap_or(Vec::new())
        })
        .await
    }

    pub async fn get_all(conn: &DbConn) -> Vec<Item> {
        conn.run(move |c| all_items.load::<Item>(c).unwrap_or(Vec::new()))
            .await
    }

    pub async fn from_id(id: i32, conn: &DbConn) -> Option<Item> {
        conn.run(move |c| {
            let item = all_items
                .filter(item_id.eq(id))
                .get_result::<Item>(c)
                .ok()?;
            Some(item)
        })
        .await
    }

    pub async fn from_ids(
        ids: Vec<i32>,
        discussed_only: bool,
        undiscussed_only: bool,
        conn: &DbConn,
    ) -> Result<Vec<Item>> {
        conn.run(move |c| {
            let mut query = all_items.into_boxed();
            if ids.len() > 0 {
                query = query.filter(item_id.eq_any(ids));
            }
            if discussed_only {
                query = query
                    .filter(item_discussed_on.is_not_null())
            } else if undiscussed_only {
                query = query
                    .filter(item_discussed_on.is_null())
            }

            let items = query.get_results::<Item>(c);

            Ok(items.context("Failed to retrieve items form db.")?)
        })
        .await
    }

    pub async fn add(item_data: NewItemData, conn: &DbConn) -> Result<()> {
        conn.run(move |c| {
            diesel::insert_into(all_items)
                .values(&item_data)
                .execute(c)
                .context("Failed inserting new item into db.")?;
            Ok(())
        })
        .await
    }

    pub async fn update(item_data: ChangeItemData, conn: &DbConn) -> Result<()> {
        use self::schema::items::dsl::{html, markdown, title};
        use std::ops::Not;

        conn.run(move |c| {
            let discussed_on = &item_data.discussed_on;
            let discussed = discussed_on.is_empty().not().then(|| discussed_on);
            diesel::update(all_items.filter(item_id.eq(item_data.id)))
                .set((
                    title.eq(item_data.title),
                    html.eq(item_data.html),
                    markdown.eq(item_data.markdown),
                    item_discussed_on.eq(discussed),
                ))
                .execute(c)
                .context("Failed inserting new item into db.")?;
            Ok(())
        })
        .await
    }

    pub async fn delete(ids: Vec<i32>, conn: &DbConn) -> Result<usize> {
        conn.run(move |c| {
            let rows = diesel::delete(all_items.filter(item_id.eq_any(ids)))
                .execute(c)
                .context("Failed to delete items from db.")?;
            Ok(rows)
        })
        .await
    }

    pub async fn set_discussed(id: i32, date: Option<NaiveDate>, conn: &DbConn) -> Result<()> {
        conn.run(move |c| {
            diesel::update(all_items.filter(item_id.eq(id)))
                .set(item_discussed_on.eq(date))
                .execute(c)
                .context("Failed inserting new item into db.")?;
            Ok(())
        })
        .await
    }
}
