use rocket::http::Status;
use rocket::response::status;
use rocket::{get, routes};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/-1/error")]
fn error() -> status::Custom<&'static str> {
    status::Custom(Status::InternalServerError, "")
}

#[get("/1/<num1>/<num2>")]
fn cube_the_bits(num1: i32, num2: i32) -> String {
    let a = num1 ^ num2;
    let a = a.pow(3);
    a.to_string()
}

#[shuttle_runtime::main]
async fn main() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build().mount("/", routes![index, error, cube_the_bits]);

    Ok(rocket.into())
}
