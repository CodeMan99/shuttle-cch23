use rocket::http::uri::{fmt, Segments};
use rocket::http::Status;
use rocket::request::FromSegments;
use rocket::response::status;
use rocket::{get, routes};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/-1/error")]
fn error() -> status::Custom<&'static str> {
    status::Custom(Status::InternalServerError, "")
}

#[derive(Debug)]
struct SledIdInput(Vec<i32>);

impl<'r> FromSegments<'r> for SledIdInput {
    type Error = std::num::ParseIntError;

    fn from_segments(segments: Segments<'r, fmt::Path>) -> Result<Self, Self::Error> {
        let mut nums = Vec::new();
        for segment in segments {
            let num: i32 = segment.parse()?;
            nums.push(num);
        }
        Ok(SledIdInput(nums))
    }
}

/// Handles both Task 1-1 & 1-2.
#[get("/1/<nums..>")]
fn sled_id(nums: SledIdInput) -> String {
    let SledIdInput(nums) = nums;
    let a = nums.iter().fold(0, |acc, &x| acc ^ x) as i64;
    let a = a.pow(3);
    a.to_string()
}

#[shuttle_runtime::main]
async fn main() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build().mount("/", routes![index, error, sled_id]);

    Ok(rocket.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sled_id_1() {
        let values = vec![4, 8];
        let segments = SledIdInput(values.into());
        let result = sled_id(segments);

        assert_eq!(result, "1728");
    }

    #[test]
    fn test_sled_id_2() {
        let values = vec![10];
        let segments = SledIdInput(values.into());
        let result = sled_id(segments);

        assert_eq!(result, "1000");
    }

    #[test]
    fn test_sled_id_3() {
        let values = vec![4, 5, 8, 10];
        let segments = SledIdInput(values.into());
        let result = sled_id(segments);

        assert_eq!(result, "27");
    }
}
