use rocket::{get, routes};
use rocket::http::Status;
use rocket::response::status;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/-1/error")]
fn error() -> status::Custom<&'static str> {
    status::Custom(Status::InternalServerError, "")
}

#[shuttle_runtime::main]
async fn main() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build().mount("/", routes![index, error]);

    Ok(rocket.into())
}
