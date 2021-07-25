use super::*;

#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub is_admin: bool
}

#[derive(Debug)]
pub struct AdminUser {
    pub user: User
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
