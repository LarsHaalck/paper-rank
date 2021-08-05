use super::*;

use rocket::outcome::{try_outcome, IntoOutcome};
use rocket::request::{FromRequest, Outcome, Request};

use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rand_core::OsRng;

#[derive(Queryable, Debug)]
struct UserDB {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub is_admin: bool,
    pub is_approved: bool,
}

#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub is_admin: bool,
    pub is_approved: bool,
}

#[derive(Debug)]
pub struct AdminUser<'a> {
    pub user: &'a User,
}

#[derive(FromForm, Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

#[derive(FromForm, Debug)]
pub struct NewPassword {
    pub old_password: String,
    pub new_password: String,
}

mod password {
    use super::*;

    pub fn verify(password: &String, hash: &String) -> Result<(), Error> {
        // Verify password against PHC string
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Failed reading password."))?;
        Pbkdf2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Wrong password."))?;
        Ok(())
    }

    pub fn generate_new_hash(password: &String) -> Result<String, Error> {
        let salt = SaltString::generate(&mut OsRng);
        // Hash password to PHC string ($pbkdf2-sha256$...)
        Ok(Pbkdf2
            .hash_password_simple(password.as_bytes(), salt.as_ref())
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "Failed hashing password."))?
            .to_string())
    }

    pub fn generate_random_password() -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect()
    }
}

impl NewUser {
    pub async fn login(self, conn: &DbConn) -> Result<User, Error> {
        conn.run(move |c| {
            let user = all_users
                .filter(user_username.eq(&self.username))
                .filter(user_approved)
                .get_result::<UserDB>(c)
                .map_err(|_| Error::new(ErrorKind::NotFound, "User not found or not approved."))?;

            password::verify(&self.password, &user.password)?;
            Ok(User::from(user))
        })
        .await
    }

    pub async fn register(self, conn: &DbConn) -> Result<(), Error> {
        conn.run(move |c| {
            let user = all_users
                .filter(user_username.eq(&self.username))
                .get_result::<UserDB>(c);

            if let Ok(_) = user {
                return Err(Error::new(ErrorKind::AlreadyExists, "User already exists."));
            }

            let password_hash = password::generate_new_hash(&self.password)?;
            let new_user = NewUser {
                username: self.username,
                password: password_hash,
            };

            diesel::insert_into(all_users)
                .values(&new_user)
                .execute(c)
                .map_err(|_| Error::new(ErrorKind::Other, "Failed to write into db."))?;
            Ok(())
        })
        .await
    }
}

impl From<UserDB> for User {
    fn from(u: UserDB) -> Self {
        Self {
            id: u.id,
            username: u.username,
            is_admin: u.is_admin,
            is_approved: u.is_approved,
        }
    }
}

impl User {
    pub async fn from_id(id: i32, conn: &DbConn) -> Option<User> {
        let user = conn
            .run(move |c| {
                all_users
                    .filter(user_id.eq(id))
                    .filter(user_approved)
                    .get_result::<UserDB>(c)
                    .ok()
            })
            .await?;
        Some(User::from(user))
    }

    pub async fn from_ids(ids: Vec<i32>, conn: &DbConn) -> Result<Vec<User>, Error> {
        conn.run(move |c| {
            let users: QueryResult<Vec<UserDB>>;
            if ids.len() > 0 {
                users = all_users
                    .filter(user_id.eq_any(ids))
                    .get_results::<UserDB>(c);
            } else {
                users = all_users.get_results::<UserDB>(c);
            }

            let users = users
                .map_err(|_| Error::new(ErrorKind::Other, "Failed to retrieve users form db."))?
                .into_iter()
                .map(|u| User::from(u))
                .collect();
            Ok(users)
        })
        .await
    }

    pub async fn change_password(
        user: &User,
        new_password: NewPassword,
        conn: &DbConn,
    ) -> Result<(), Error> {
        let id = user.id;
        conn.run(move |c| {
            let user = all_users
                .filter(user_id.eq(id))
                .get_result::<UserDB>(c)
                .map_err(|_| Error::new(ErrorKind::NotFound, "User not found in db."))?;

            password::verify(&new_password.old_password, &user.password)?;
            let hash = password::generate_new_hash(&new_password.new_password)?;
            diesel::update(all_users.filter(user_id.eq(user.id)))
                .set(user_password.eq(hash))
                .execute(c)
                .map_err(|_| Error::new(ErrorKind::Other, "Failed to write into db."))?;

            Ok(())
        })
        .await
    }

    pub async fn set_random_password(id: i32, conn: &DbConn) -> Result<String, Error> {
        conn.run(move |c| {
            let password = password::generate_random_password();
            let hash = password::generate_new_hash(&password)?;
            let rows = diesel::update(all_users.filter(user_id.eq(id)))
                .set(user_password.eq(hash))
                .execute(c)
                .map_err(|_| Error::new(ErrorKind::Other, "Failed to write into db."))?;

            if rows > 0 {
                Ok(password)
            } else {
                Err(Error::new(ErrorKind::NotFound, "User not found in db."))
            }
        })
        .await
    }

    pub async fn set_approve(ids: Vec<i32>, value: bool, conn: &DbConn) -> Result<usize, Error> {
        conn.run(move |c| {
            let rows: QueryResult<usize>;
            if ids.len() > 0 {
                rows = diesel::update(
                    all_users.filter(user_id.eq_any(ids).and(user_approved.eq(!value))),
                )
                .set(user_approved.eq(value))
                .execute(c);
            } else {
                rows = diesel::update(all_users.filter(user_approved.eq(!value)))
                    .set(user_approved.eq(value))
                    .execute(c);
            }
            let rows =
                rows.map_err(|_| Error::new(ErrorKind::Other, "Failed to approve users in db."))?;
            Ok(rows)
        })
        .await
    }

    pub async fn delete(ids: Vec<i32>, conn: &DbConn) -> Result<usize, Error> {
        conn.run(move |c| {
            let rows = diesel::delete(all_users.filter(user_id.eq_any(ids)))
                .execute(c)
                .map_err(|_| Error::new(ErrorKind::Other, "Failed to delete users from db."))?;
            Ok(rows)
        })
        .await
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r User {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, ()> {
        let user_result = request
            .local_cache_async(async {
                let conn = request.guard::<DbConn>().await.succeeded()?;
                let r = request
                    .cookies()
                    .get_private("user_id")
                    .and_then(|cookie| cookie.value().parse().ok())?;

                User::from_id(r, &conn).await
            })
            .await;

        user_result.as_ref().or_forward(())
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, ()> {
        let user = try_outcome!(request.guard::<&User>().await);
        if user.is_admin {
            Outcome::Success(AdminUser { user })
        } else {
            Outcome::Forward(())
        }
    }
}
