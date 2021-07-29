use super::*;

use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
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
}

// #[derive(Debug)]
// pub struct AdminUser {
//     pub user: User,
// }

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

            let password_hash = password::generate_new_hash(&self.password)?;
            let new_user = NewUser {
                username: self.username,
                password: password_hash,
            };

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
                    .filter(user_approved)
                    .get_result::<UserDB>(c)
                    .ok()
            })
            .await?;
        Some(User {
            id: user.id,
            username: user.username,
        })
    }

    pub async fn change_password(
        self,
        new_password: NewPassword,
        conn: &DbConn,
    ) -> Result<(), Error> {
        conn.run(move |c| {
            let user = all_users
                .filter(user_id.eq(self.id))
                .get_result::<UserDB>(c)
                .map_err(|_| Error::new(ErrorKind::NotFound, "User not found in db."))?;

            password::verify(&new_password.old_password, &user.password)?;
            let hash = password::generate_new_hash(&new_password.new_password)?;
            diesel::update(all_users.filter(user_id.eq(self.id)))
                .set(user_password.eq(hash))
                .execute(c)
                .map_err(|_| Error::new(ErrorKind::Other, "Faile to write into db."))?;

            Ok(())
        })
        .await
    }
}
