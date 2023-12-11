use rocket::get;

#[get("/")]
fn day_11_index() {}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![day_11_index]
}
