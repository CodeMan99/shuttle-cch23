use rocket::get;

#[get("/")]
fn day_13_index() {}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![day_13_index]
}
