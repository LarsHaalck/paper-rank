use super::*;

use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use rand_core::OsRng;
use std::io::{Error, ErrorKind};
// let password = b"hunter42";
// let salt = SaltString::generate(&mut OsRng);

// // Hash password to PHC string ($pbkdf2-sha256$...)
// let password_hash = Pbkdf2.hash_password_simple(password, salt.as_ref()).unwrap().to_string();

// // Verify password against PHC string
// let parsed_hash = PasswordHash::new(&password_hash).unwrap();
// assert!(Pbkdf2.verify_password(password, &parsed_hash).is_ok());

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
}

#[derive(Debug)]
pub struct AdminUser {
    pub user: User,
}

#[derive(FromForm, Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

impl NewUser {
    pub async fn login(self, conn: &DbConn) -> Result<User, Error> {
        conn.run(move |c| {
            let user = all_users
                .filter(user_username.eq(&self.username))
                .filter(user_approved)
                .get_result::<UserDB>(c)
                .map_err(|_| Error::new(ErrorKind::NotFound, "User not found or not approved"))?;

            // Verify password against PHC string
            let parsed_hash = PasswordHash::new(&user.password)
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Failed reading password"))?;
            Pbkdf2
                .verify_password(&self.password.as_bytes(), &parsed_hash)
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Passwords do not match"))?;
            Ok(User {
                id: user.id,
                username: user.username,
            })
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

            let salt = SaltString::generate(&mut OsRng);
            // Hash password to PHC string ($pbkdf2-sha256$...)
            let password_hash = Pbkdf2
                .hash_password_simple(&self.password.as_bytes(), salt.as_ref())
                .map_err(|_| Error::new(ErrorKind::InvalidInput, "Failed hashing password."))?
                .to_string();

            let new_user = NewUser {
                username: self.username,
                password: password_hash,
            };

            println!("{:?}", new_user);

            diesel::insert_into(all_users)
                .values(&new_user)
                .execute(c)
                .map_err(|_| Error::new(ErrorKind::Other, "Faile to write into db."))?;
            Ok(())
        })
        .await
    }
}

impl User {
    pub async fn from_id(id: i32, conn: &DbConn) -> Option<User> {
        let user = conn
            .run(move |c| {
                all_users
                    .filter(user_id.eq(id))
                    .get_result::<UserDB>(c)
                    .ok()
            })
            .await?;
        Some(User {
            id: user.id,
            username: user.username,
        })
    }
}
