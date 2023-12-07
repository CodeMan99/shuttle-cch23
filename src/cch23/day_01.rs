use std::rc::Rc;
use std::str::FromStr;

use rocket::get;
use rocket::http::uri::{fmt, Segments};
use rocket::request::FromSegments;

#[derive(Debug)]
pub struct SegmentsRest<T>(Rc<[T]>);

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
pub fn sled_id(nums: SegmentsRest<i32>) -> String {
    let SegmentsRest(nums) = nums;
    let a = nums.iter().fold(0, |acc, &x| acc ^ x) as i64;
    let a = a.pow(3);
    a.to_string()
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
