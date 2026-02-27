use rocket::Route;

mod host;
mod index;

pub fn routes() -> Vec<Route> {
    [index::routes(), host::routes()]
        .into_iter()
        .flatten()
        .collect()
}
