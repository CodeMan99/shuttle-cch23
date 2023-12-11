use base64::engine::{general_purpose::URL_SAFE, Engine};
use indexmap::map::IndexMap;
use rocket::get;
use rocket::http::{CookieJar, Status};
use rocket::serde::{json::Json, Deserialize, Serialize};
use serde::de::DeserializeOwned;

#[derive(Debug, Serialize)]
struct RecipeError {
    error: &'static str,
}

impl RecipeError {
    fn new(error: &'static str) -> RecipeError {
        RecipeError { error }
    }

    fn as_response(self, status: Status) -> (Status, Json<RecipeError>) {
        (status, Json(self))
    }
}

const MAX_COOKIE_SIZE: usize = 4096;

fn decode_cookie_recipe<T: DeserializeOwned>(cookie_b64: &str) -> Result<T, RecipeError> {
    let mut bytes: [u8; MAX_COOKIE_SIZE] = [0; MAX_COOKIE_SIZE];
    let size = URL_SAFE
        .decode_slice(cookie_b64.as_bytes(), &mut bytes)
        .map_err(|_| RecipeError::new("Recipe decoding failed, no cookies for Santa!"))?;
    let value = serde_json::from_slice(&bytes[..size])
        .map_err(|_| RecipeError::new("Recipe deserialization failed, no cookies for Santa!"))?;

    Ok(value)
}

type JsonResult<T, E> = Result<Json<T>, (Status, Json<E>)>;

#[get("/decode")]
fn decode(cookies: &CookieJar<'_>) -> JsonResult<serde_json::Value, RecipeError> {
    if let Some(recipe) = cookies.get("recipe").map(|cookie| cookie.value()) {
        let recipe = decode_cookie_recipe(recipe)
            .map_err(|err| err.as_response(Status::UnprocessableEntity))?;

        Ok(Json(recipe))
    } else {
        Err(RecipeError::new("No recipe cookie found").as_response(Status::BadRequest))
    }
}

type IngredientUnit = u64;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Kitchen {
    recipe: IndexMap<String, IngredientUnit>,
    pantry: IndexMap<String, IngredientUnit>,
}

#[derive(Debug, Serialize)]
struct BakeCookies {
    cookies: IngredientUnit,
    pantry: IndexMap<String, IngredientUnit>,
}

#[get("/bake")]
fn bake(cookies: &CookieJar<'_>) -> JsonResult<BakeCookies, RecipeError> {
    if let Some(recipe_b64) = cookies.get("recipe").map(|cookie| cookie.value()) {
        let Kitchen { recipe, mut pantry } = decode_cookie_recipe(recipe_b64)
            .map_err(|err| err.as_response(Status::UnprocessableEntity))?;
        let mut cookies: IngredientUnit = 0;
        let mut pantry_update = pantry.clone();

        'baking_counter: loop {
            'cookie: for (ingredient, &quantity) in &recipe {
                if quantity == 0 {
                    continue 'cookie;
                }

                match pantry.get(ingredient) {
                    Some(&pantry_has) if quantity <= pantry_has => {
                        pantry_update.insert(ingredient.to_owned(), pantry_has - quantity);
                    }
                    // Pantry does not have enough
                    Some(_) | None => break 'baking_counter,
                }
            }

            cookies += 1;
            pantry.clone_from(&pantry_update);
        }

        Ok(Json(BakeCookies { cookies, pantry }))
    } else {
        Err(RecipeError::new("No recipe cookie found").as_response(Status::BadRequest))
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![decode, bake]
}

#[cfg(test)]
mod tests_day_07 {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("eyJmbG91ciI6MTAwLCJjaG9jb2xhdGUgY2hpcHMiOjIwfQ==", serde_json::json!({"flour": 100, "chocolate chips": 20}))]
    #[case("eyJwZWFudXQgYnV0dGVyIjo0MH0=", serde_json::json!({"peanut butter": 40}))]
    fn test_decode_cookie_value(#[case] cookie: &str, #[case] expected: serde_json::Value) {
        match decode_cookie_recipe::<serde_json::Value>(cookie) {
            Ok(value) => assert_eq!(value, expected),
            Err(err) => assert!(false, "{}", err.error),
        }
    }

    #[rstest]
    #[case(
        "eyJyZWNpcGUiOnsiZmxvdXIiOjk1LCJzdWdhciI6NTAsImJ1dHRlciI6MzAsImJha2luZyBwb3dkZXIiOjEwLCJjaG9jb2xhdGUgY2hpcHMiOjUwfSwicGFudHJ5Ijp7ImZsb3VyIjozODUsInN1Z2FyIjo1MDcsImJ1dHRlciI6MjEyMiwiYmFraW5nIHBvd2RlciI6ODY1LCJjaG9jb2xhdGUgY2hpcHMiOjQ1N319",
        Kitchen {
            recipe: IndexMap::from([
                ("flour".to_owned(), 95), 
                ("sugar".to_owned(), 50), 
                ("butter".to_owned(), 30), 
                ("baking powder".to_owned(), 10), 
                ("chocolate chips".to_owned(), 50),
            ]),
            pantry: IndexMap::from([
                ("flour".to_owned(), 385),
                ("sugar".to_owned(), 507),
                ("butter".to_owned(), 2122),
                ("baking powder".to_owned(), 865),
                ("chocolate chips".to_owned(), 457),
            ]),
        },
    )]
    #[case(
        "eyJyZWNpcGUiOnsic2xpbWUiOjl9LCJwYW50cnkiOnsiY29iYmxlc3RvbmUiOjY0LCJzdGljayI6IDR9fQ==",
        Kitchen {
            recipe: IndexMap::from([
                ("slime".to_owned(), 9),
            ]),
            pantry: IndexMap::from([
                ("cobblestone".to_owned(), 64),
                ("stick".to_owned(), 4),
            ]),
        },
    )]
    fn test_decode_cookie_kitchen(#[case] cookie: &str, #[case] expected: Kitchen) {
        match decode_cookie_recipe::<Kitchen>(cookie) {
            Ok(kitchen) => assert_eq!(kitchen, expected),
            Err(err) => assert!(false, "{}", err.error),
        }
    }
}
