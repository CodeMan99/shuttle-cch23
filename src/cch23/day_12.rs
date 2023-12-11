use rocket::get;

#[get("/")]
fn day_12_index() {}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![day_12_index]
}
