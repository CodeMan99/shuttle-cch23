use rocket::get;

#[get("/")]
fn day_22_index() {}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![day_22_index]
}
