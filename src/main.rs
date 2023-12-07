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
    let rocket = rocket::build().mount(
        "/",
        routes![
            index,
            error,
            cch23::day_01::sled_id,
            cch23::day_04::reindeer_team_strength,
            cch23::day_04::reindeer_contest,
            cch23::day_06::elf_on_a_shelf,
            cch23::day_07::decode,
            cch23::day_07::bake,
        ],
    );

    Ok(rocket.into())
}
