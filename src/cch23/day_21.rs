use rocket::get;

#[get("/")]
fn day_21_index() {}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![day_21_index]
}
