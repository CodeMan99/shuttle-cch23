use rocket::get;

#[get("/")]
fn day_15_index() {}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![day_15_index]
}
