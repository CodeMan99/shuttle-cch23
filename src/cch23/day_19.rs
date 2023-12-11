use rocket::get;

#[get("/")]
fn day_19_index() {}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![day_19_index]
}
