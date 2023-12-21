use rocket::post;
use rocket::serde::json::Json;

#[post("/?<offset>&<limit>&<split>", data = "<names>")]
fn name_pager(
    offset: Option<usize>,
    limit: Option<usize>,
    split: Option<usize>,
    names: Json<Vec<String>>,
) -> Json<serde_json::Value> {
    let offset = offset.unwrap_or_default();
    let limit = limit.unwrap_or(usize::MAX);

    if let Some(split) = split {
        Json(names[offset..].chunks(split).take(limit).collect())
    } else {
        Json(names.iter().skip(offset).take(limit).cloned().collect())
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![name_pager]
}
