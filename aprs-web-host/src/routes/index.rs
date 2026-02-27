#![expect(clippy::toplevel_ref_arg)]

use askama::Template;
use askama_web::WebTemplate;
use rocket::Route;

use crate::db::Db;
use crate::models::{User, UserId};

pub fn routes() -> Vec<Route> {
    routes![get]
}

#[get("/")]
pub async fn get(ref mut db: Db<'_>, user: User) -> IndexTemplate {
    IndexTemplate { user }
}

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    user: User,
}
