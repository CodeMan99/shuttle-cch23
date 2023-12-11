use rocket::get;

#[get("/")]
fn day_18_index() {}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![day_18_index]
}
