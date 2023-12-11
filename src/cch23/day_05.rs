use rocket::get;

#[get("/")]
fn day_05_index() {}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![day_05_index]
}
