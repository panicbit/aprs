use std::ops::Deref;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use color_eyre::eyre::{self, Result};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_derive_newtype::DieselNewType;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::try_outcome;
use rocket::request::{self, FromRequest, Outcome, Request};
use uuid::Uuid;

use crate::db::Db;
use crate::models::UUID;
use crate::schema;

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: UserId,
    pub first_visit: DateTime<Utc>,
    pub last_visit: DateTime<Utc>,
}

impl User {
    pub async fn get(db: &mut Db<'_>, user_id: UserId) -> Result<Option<User>> {
        use schema::users::dsl::*;

        let user = users
            .select(User::as_select())
            .filter(id.eq(user_id))
            .first(db)
            .await
            .optional()?;

        Ok(user)
    }

    pub async fn get_or_create(db: &mut Db<'_>, user_id: UserId) -> Result<User> {
        use schema::users::dsl::*;

        let now = Utc::now();
        let user = User {
            id: user_id,
            first_visit: now,
            last_visit: now,
        };

        let user = diesel::insert_into(users)
            .values(user)
            .on_conflict(id)
            .do_update()
            .set(last_visit.eq(now))
            .get_result::<User>(db)
            .await?;

        Ok(user)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = color_eyre::Report;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cookies = request.cookies();
        let user_id = match cookies.get_pending("user_id") {
            Some(user_id) => match user_id.value().parse::<UserId>() {
                Ok(user_id) => user_id,
                Err(err) => {
                    eprintln!("Failed to parse user id ({err}); generating new one");

                    let user_id = UserId::new_random();
                    cookies.add_private(("user_id", user_id.to_string()));

                    user_id
                }
            },
            None => {
                let user_id = UserId::new_random();
                cookies.add_private(("user_id", user_id.to_string()));

                user_id
            }
        };

        let mut db = try_outcome!(request.guard::<Db>().await);

        let user = match User::get_or_create(&mut db, user_id).await {
            Ok(user) => user,
            Err(err) => return Outcome::Error((Status::InternalServerError, err)),
        };

        Outcome::Success(user)
    }
}

#[derive(Copy, Clone, Debug, DieselNewType)]
pub struct UserId(UUID);

impl UserId {
    pub fn new_random() -> Self {
        Self(UUID::new_random())
    }
}

impl FromStr for UserId {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = <UUID as FromStr>::from_str(s)?;

        Ok(Self(uuid))
    }
}

impl Deref for UserId {
    type Target = Uuid;

    fn deref(&self) -> &Uuid {
        &self.0
    }
}
