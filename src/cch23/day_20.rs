use rocket::get;

#[get("/")]
fn day_20_index() {}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![day_20_index]
}
