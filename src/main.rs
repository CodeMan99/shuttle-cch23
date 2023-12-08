use rocket::http::Status;
use rocket::response::status;
use rocket::{get, routes};

mod cch23;

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
    let rocket = rocket::build()
        .mount("/", routes![index, error])
        .mount("/1", cch23::day_01::routes())
        .mount("/4", cch23::day_04::routes())
        .mount("/6", cch23::day_06::routes())
        .mount("/7", cch23::day_07::routes())
        .mount("/8", cch23::day_08::routes())
        .manage(cch23::day_08::init_rustemon_client());

    Ok(rocket.into())
}
