use base64::engine::{general_purpose::URL_SAFE, Engine};
use indexmap::map::IndexMap;
use rocket::get;
use rocket::http::CookieJar;
use rocket::serde::{json::Json, Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct RecipeError {
    error: &'static str,
}

impl RecipeError {
    fn new(error: &'static str) -> RecipeError {
        RecipeError { error }
    }
}

const MAX_COOKIE_SIZE: usize = 4096;

fn decode_cookie(cookie: &str) -> Result<serde_json::Value, RecipeError> {
    let mut bytes: [u8; MAX_COOKIE_SIZE] = [0; MAX_COOKIE_SIZE];
    let size = URL_SAFE
        .decode_slice(cookie.as_bytes(), &mut bytes)
        .map_err(|_| RecipeError::new("Recipe decoding failed, no cookies for Santa!"))?;
    let value = serde_json::from_slice(&bytes[..size])
        .map_err(|_| RecipeError::new("Recipe deserialization failed, no cookies for Santa!"))?;

    Ok(value)
}

#[get("/7/decode")]
pub fn decode(cookies: &CookieJar<'_>) -> Result<Json<serde_json::Value>, Json<RecipeError>> {
    if let Some(recipe) = cookies.get("recipe").map(|cookie| cookie.value()) {
        let recipe = decode_cookie(recipe)?;

        Ok(Json(recipe))
    } else {
        Err(Json(RecipeError::new("No recipe cookie found")))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Kitchen {
    recipe: IndexMap<String, u32>,
    pantry: IndexMap<String, u32>,
}

#[derive(Debug, Serialize)]
pub struct BakeCookies {
    cookies: u32,
    pantry: IndexMap<String, u32>,
}

#[get("/7/bake")]
pub fn bake(cookies: &CookieJar<'_>) -> Result<Json<BakeCookies>, Json<RecipeError>> {
    if let Some(recipe_b64) = cookies.get("recipe").map(|cookie| cookie.value()) {
        let value = decode_cookie(recipe_b64)?;
        let Kitchen { recipe, mut pantry } = serde_json::from_value(value)
            .map_err(|_| RecipeError::new("Could not understand recipe, refusing to make cookies for Santa!"))?;
        let mut cookies: u32 = 0;
        let mut pantry_update = pantry.clone();

        'baking_counter: loop {
            for (ingredient, &quantity) in &recipe {
                let pantry_has = pantry.get(ingredient).cloned().unwrap_or_default();
                
                if quantity <= pantry_has {
                    pantry_update.insert(ingredient.to_owned(), pantry_has - quantity);
                } else {
                    break 'baking_counter;
                }
            }

            cookies += 1;
            pantry.clone_from(&pantry_update);
        }

        Ok(Json(BakeCookies {
            cookies,
            pantry,
        }))
    } else {
        Err(Json(RecipeError::new("No recipe cookie found")))
    }
}
