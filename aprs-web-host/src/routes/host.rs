use std::io;
use std::path::Path;

use askama::Template;
use askama_web::WebTemplate;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::{Route, tokio};
use uuid::Uuid;

pub fn routes() -> Vec<Route> {
    routes![get, post,]
}

#[get("/host")]
pub fn get() -> HostTemplate {
    HostTemplate {}
}

#[post("/host", data = "<form>")]
pub async fn post(mut form: Form<UploadForm<'_>>) -> io::Result<String> {
    let uuid = Uuid::new_v4();
    let room_dir = format!("data/rooms/{uuid}");
    let multiworld_path = Path::new(&room_dir).join("multiworld.zip");

    tokio::fs::create_dir_all(room_dir).await?;

    form.multiworld.len();
    form.multiworld.move_copy_to(multiworld_path).await?;

    Ok("successfully uploaded".to_string())
}

#[derive(Template, WebTemplate)]
#[template(path = "host.html")]
pub struct HostTemplate {}

#[derive(FromForm)]
pub struct UploadForm<'v> {
    multiworld: TempFile<'v>,
}
