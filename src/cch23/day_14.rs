use rocket::response::content::RawHtml;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::post;
use rocket_dyn_templates::{context, Template};

#[derive(Debug, Deserialize, Serialize)]
struct SantaInput {
    content: String,
}

#[post("/unsafe", data = "<santa_input>")]
fn unsafe_render(santa_input: Json<SantaInput>) -> RawHtml<String> {
    RawHtml(format!(r#"<html>
  <head>
    <title>CCH23 Day 14</title>
  </head>
  <body>
    {}
  </body>
</html>"#, santa_input.content))
}

#[post("/safe", data = "<santa_input>")]
fn safe_render(santa_input: Json<SantaInput>) -> Template {
    Template::render("day_14", context! {
        content: &santa_input.content,
    })
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![unsafe_render, safe_render]
}
