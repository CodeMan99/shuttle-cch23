use rocket::get;

#[get("/")]
fn day_14_index() {}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![day_14_index]
}
