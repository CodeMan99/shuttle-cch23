use std::rc::Rc;
use std::str::FromStr;

use rocket::http::uri::{fmt, Segments};
use rocket::http::Status;
use rocket::request::FromSegments;
use rocket::response::status;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, post, routes};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/-1/error")]
fn error() -> status::Custom<&'static str> {
    status::Custom(Status::InternalServerError, "")
}

#[derive(Debug)]
struct SegmentsRest<T>(Rc<[T]>);

impl<'r, T: FromStr<Err = Err>, Err: std::fmt::Debug> FromSegments<'r> for SegmentsRest<T> {
    type Error = Err;

    fn from_segments(segments: Segments<'r, fmt::Path>) -> Result<Self, Self::Error> {
        let mut values: Vec<T> = Vec::new();
        for segment in segments {
            let value: T = segment.parse()?;
            values.push(value);
        }
        Ok(SegmentsRest(values.into()))
    }
}

/// Handles both Task 1-1 & 1-2.
#[get("/1/<nums..>")]
fn sled_id(nums: SegmentsRest<i32>) -> String {
    let SegmentsRest(nums) = nums;
    let a = nums.iter().fold(0, |acc, &x| acc ^ x) as i64;
    let a = a.pow(3);
    a.to_string()
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(default)]
struct Reindeer<'r> {
    name: &'r str,
    strength: i32,
    speed: f64,
    height: i32,
    antler_width: i32,
    snow_magic_power: i32,
    favorite_food: &'r str,
    #[serde(rename = "cAnD13s_3ATeN-yesT3rdAy")]
    candies_eaten_yesterday: i32,
}

// TODO not sure if the return type is correct. Might need to be Json<String>.
#[post("/4/strength", data = "<team>")]
fn reindeer_team_strength(team: Json<Vec<Reindeer<'_>>>) -> String {
    team.iter()
        .map(|reindeer| reindeer.strength)
        .sum::<i32>()
        .to_string()
}

trait ContestAward {
    fn award_text(&self) -> String;
}

struct FastestReindeer<'r>(&'r Reindeer<'r>);

impl<'r> ContestAward for FastestReindeer<'r> {
    fn award_text(&self) -> String {
        let strength = self.0.strength;
        let name = self.0.name;
        format!("Speeding past the finish line with a strength of {strength} is {name}")
    }
}

struct TallestReindeer<'r>(&'r Reindeer<'r>);

impl<'r> ContestAward for TallestReindeer<'r> {
    fn award_text(&self) -> String {
        let name = self.0.name;
        let antler_width = self.0.antler_width;
        format!("{name} is standing tall with his {antler_width} cm wide antlers")
    }
}

struct MagicianReindeer<'r>(&'r Reindeer<'r>);

impl<'r> ContestAward for MagicianReindeer<'r> {
    fn award_text(&self) -> String {
        let name = self.0.name;
        let snow_magic_power = self.0.snow_magic_power;
        format!("{name} could blast you away with a snow magic power of {snow_magic_power}")
    }
}

struct ConsumerReindeer<'r>(&'r Reindeer<'r>);

impl<'r> ContestAward for ConsumerReindeer<'r> {
    fn award_text(&self) -> String {
        let name = self.0.name;
        let favorite_food = self.0.favorite_food;
        format!("{name} ate lots of candies, but also some {favorite_food}")
    }
}

#[derive(Debug, Default, Serialize, PartialEq)]
struct ReindeerContest {
    fastest: String,
    tallest: String,
    magician: String,
    consumer: String,
}

#[post("/4/contest", data = "<team>")]
fn reindeer_contest(team: Json<Vec<Reindeer<'_>>>) -> Json<ReindeerContest> {
    let fastest = team
        .iter()
        .max_by(|&r1, &r2| r1.speed.total_cmp(&r2.speed))
        .map(|r| FastestReindeer(r).award_text())
        .unwrap_or_default();
    let tallest = team
        .iter()
        .max_by_key(|&r| r.height)
        .map(|r| TallestReindeer(r).award_text())
        .unwrap_or_default();
    let magician = team
        .iter()
        .max_by_key(|&r| r.snow_magic_power)
        .map(|r| MagicianReindeer(r).award_text())
        .unwrap_or_default();
    let consumer = team
        .iter()
        .max_by_key(|&r| r.candies_eaten_yesterday)
        .map(|r| ConsumerReindeer(r).award_text())
        .unwrap_or_default();
    Json(ReindeerContest {
        fastest,
        tallest,
        magician,
        consumer,
    })
}

#[shuttle_runtime::main]
async fn main() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build().mount(
        "/",
        routes![
            index,
            error,
            sled_id,
            reindeer_team_strength,
            reindeer_contest,
        ],
    );

    Ok(rocket.into())
}

#[cfg(test)]
mod tests_day_01 {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(vec![4, 8], "1728")]
    #[case(vec![10], "1000")]
    #[case(vec![4, 5, 8, 10], "27")]
    #[case(vec![1, 2, 4, 8], "3375")]
    fn test_sled_id(#[case] values: Vec<i32>, #[case] expected: &str) {
        let segments = SegmentsRest(values.into());
        let result = sled_id(segments);

        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod tests_day_04 {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(
        r#"[
            {"name": "Dasher", "strength": 5},
            {"name": "Dancer", "strength": 6},
            {"name": "Prancer", "strength": 4},
            {"name": "Vixen", "strength": 7}
        ]"#,
        "22"
    )]
    #[case("[]", "0")]
    #[case(r#"[{"name": "Rudolph", "strength": 2}]"#, "2")]
    fn test_reindeer_team_strength(#[case] body: &str, #[case] expected: &str) {
        let team: Vec<Reindeer<'_>> = serde_json::from_str(body).unwrap();
        let result = reindeer_team_strength(Json(team));

        assert_eq!(result, expected);
    }

    #[test]
    fn test_reindeer_contest() {
        let team: Vec<Reindeer<'_>> = serde_json::from_str(
            r#"[
                {
                    "name": "Dasher",
                    "strength": 5,
                    "speed": 50.4,
                    "height": 80,
                    "antler_width": 36,
                    "snow_magic_power": 9001,
                    "favorite_food": "hay",
                    "cAnD13s_3ATeN-yesT3rdAy": 2
                },
                {
                    "name": "Dancer",
                    "strength": 6,
                    "speed": 48.2,
                    "height": 65,
                    "antler_width": 37,
                    "snow_magic_power": 4004,
                    "favorite_food": "grass",
                    "cAnD13s_3ATeN-yesT3rdAy": 5
                }
            ]"#,
        )
        .unwrap();
        let result = reindeer_contest(Json(team));
        let expected = Json(ReindeerContest {
            fastest: "Speeding past the finish line with a strength of 5 is Dasher".to_owned(),
            tallest: "Dasher is standing tall with his 36 cm wide antlers".to_owned(),
            magician: "Dasher could blast you away with a snow magic power of 9001".to_owned(),
            consumer: "Dancer ate lots of candies, but also some grass".to_owned(),
        });

        assert_eq!(result, expected);
    }
}
