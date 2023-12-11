use rocket::post;
use rocket::serde::{json::Json, Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct ElfCounter {
    elf: usize,
    #[serde(rename = "elf on a shelf")]
    shelf_with_elf: usize,
    #[serde(rename = "shelf with no elf on it")]
    shelf_without_elf: usize,
}

#[post("/", data = "<text>")]
fn elf_on_a_shelf(text: &str) -> Json<ElfCounter> {
    let elf_count = text.matches("elf").count();
    let shelf_count = text.matches("shelf").count();
    let elf_on_a_shelf = b"elf on a shelf";
    let shelf_with_elf = text
        .as_bytes()
        .windows(elf_on_a_shelf.len())
        .filter(|window| window == elf_on_a_shelf)
        .count();
    let shelf_without_elf = shelf_count - shelf_with_elf;

    Json(ElfCounter {
        elf: elf_count,
        shelf_with_elf,
        shelf_without_elf,
    })
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![elf_on_a_shelf]
}

#[cfg(test)]
mod tests_day_06 {
    use super::*;
    use rstest::*;

    #[test]
    fn test_elf_on_a_shelf() {
        let input = r#"The mischievous elf peeked out from behind the toy workshop,
                             and another elf joined in the festive dance.
                             Look, there is also an elf on that shelf!"#;
        let expected_elf_count = 4;
        let Json(result) = elf_on_a_shelf(input);

        assert_eq!(result.elf, expected_elf_count);
    }

    #[rstest]
    #[case(
        r#"there is an elf on a shelf on an elf.
           there is also another shelf in Belfast."#,
        r#"{
            "elf": 5,
            "elf on a shelf": 1,
            "shelf with no elf on it": 1
        }"#
    )]
    #[case(
        "One elf and another elf on a shelf on a shelf then another elf on a shelf",
        r#"{
            "elf": 6,
            "elf on a shelf": 3,
            "shelf with no elf on it": 0
        }"#
    )]
    #[case(
        "there is an elf on a shelf on a shelf. there is also another shelf in Belfast.",
        r#"{
            "elf": 5,
            "elf on a shelf": 2,
            "shelf with no elf on it": 1
        }"#
    )]
    fn test_elf_on_a_shelf_bonus(#[case] input: &str, #[case] response_body: &str) {
        let expected: ElfCounter = serde_json::from_str(response_body).unwrap();
        let Json(result) = elf_on_a_shelf(input);

        assert_eq!(result, expected);
    }
}
