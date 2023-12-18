use rocket::http::Status;
use rocket::response::status;
use rocket::{get, routes};
use rocket_dyn_templates::Template;
use sqlx::PgPool;

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
async fn main(#[shuttle_shared_db::Postgres()] pool: PgPool) -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![index, error])
        .mount("/1", cch23::day_01::routes())
        .mount("/4", cch23::day_04::routes())
        .mount("/5", cch23::day_05::routes())
        .mount("/6", cch23::day_06::routes())
        .mount("/7", cch23::day_07::routes())
        .mount("/8", cch23::day_08::routes())
        .mount("/11", cch23::day_11::routes())
        .mount("/12", cch23::day_12::routes())
        .mount("/13", cch23::day_13::routes())
        .mount("/14", cch23::day_14::routes())
        .mount("/15", cch23::day_15::routes())
        .mount("/18", cch23::day_18::routes())
        .mount("/19", cch23::day_19::routes())
        .mount("/20", cch23::day_20::routes())
        .mount("/21", cch23::day_21::routes())
        .mount("/22", cch23::day_22::routes())
        .manage(cch23::day_08::init_rustemon_client())
        .manage(cch23::day_12::create_storage())
        .manage(cch23::create_gift_db(pool));

    Ok(rocket.into())
}
