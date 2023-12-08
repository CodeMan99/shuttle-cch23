use std::{env, fs};

use rocket::http::Status;
use rocket::{get, State};
use rustemon::client::{CACacheManager, CacheMode, RustemonClient, RustemonClientBuilder};
use rustemon::error::Error;
use rustemon::model::pokemon::Pokemon;
use rustemon::pokemon::pokemon;

fn rustemon_server_error(error: Error) -> (Status, String) {
    match error {
        Error::FollowEmptyURL => (Status::InternalServerError, "Empty URL".to_owned()),
        Error::NoTrailingSlash(message) => (Status::InternalServerError, message),
        Error::UrlParse(message) => (Status::InternalServerError, message),
        Error::Reqwest(reqwest_error) => (Status::BadGateway, reqwest_error.to_string()),
        Error::ReqwestMiddleware(reqwest_error) => (Status::BadGateway, reqwest_error.to_string()),
    }
}

#[get("/weight/<pokedex_number>")]
async fn pokemon_weight(
    pokedex_number: i64,
    rustemon_client: &State<RustemonClient>,
) -> (Status, String) {
    let pokemon_result: Result<Pokemon, Error> =
        pokemon::get_by_id(pokedex_number, rustemon_client).await;

    match pokemon_result {
        Ok(pokemon) => {
            // The weight of this Pokémon in hectograms.
            let weight = pokemon.weight;
            let kilograms = weight / 10;

            (Status::Ok, format!("{}", kilograms))
        }
        Err(error) => rustemon_server_error(error),
    }
}

#[get("/drop/<pokedex_number>")]
async fn drop_pokemon(
    pokedex_number: i64,
    rustemon_client: &State<RustemonClient>,
) -> (Status, String) {
    let pokemon_result: Result<Pokemon, Error> =
        pokemon::get_by_id(pokedex_number, rustemon_client).await;

    match pokemon_result {
        Ok(pokemon) => {
            // The weight of this Pokémon in hectograms.
            let weight = pokemon.weight as f64;
            let kilograms = weight / 10.0f64;
            // g = 9.825 m/s²
            const GRAVITY: f64 = 9.825;
            // drop from 10-meter high chimney
            const HEIGHT: f64 = 10.0;
            // v = √(2h * g)
            let velocity = f64::sqrt(2.0f64 * HEIGHT * GRAVITY);
            // momentum, measured in Newton-seconds = kg * m/s
            let momentum = kilograms * velocity;

            (Status::Ok, format!("{}", momentum))
        }
        Err(error) => rustemon_server_error(error),
    }
}

pub fn init_rustemon_client() -> RustemonClient {
    let cache = env::temp_dir().join("cch23/rustemon-cache");

    if !cache.exists() {
        fs::create_dir_all(&cache).expect("Unable to create RustemonClinet cache directory");
    }

    let client = RustemonClientBuilder::default()
        .with_mode(CacheMode::ForceCache)
        .with_manager(CACacheManager { path: cache })
        .try_build()
        .expect("Unable to build RustemonClinet");

    client
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![pokemon_weight, drop_pokemon]
}

#[cfg(test)]
mod tests_day_08 {
    use super::*;

    /// Enforce traits needed for rocket to manage state of <T>
    fn is_manage_safe<T: Send + Sync + 'static>() {}

    #[test]
    fn test_rustemon_client_is_manage_safe() {
        is_manage_safe::<RustemonClient>();
    }
}
