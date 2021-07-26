use rocket::outcome::{try_outcome, IntoOutcome};
use rocket::request::{self, FromRequest, Outcome, Request};

use crate::db::{AdminUser, User};
use crate::DbConn;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<User, ()> {
        let conn = try_outcome!(request.guard::<DbConn>().await);
        let r = request
            .cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .or_forward(());

        let r = try_outcome!(r);
        let user = User::from_id(r, &conn).await;
        user.or_forward(())
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<AdminUser, ()> {
        let user = try_outcome!(request.guard::<User>().await);
        // if user.is_admin {
        //     Outcome::Success(AdminUser { user })
        // } else {
        //     Outcome::Forward(())
        // }
        Outcome::Forward(())
    }
}
