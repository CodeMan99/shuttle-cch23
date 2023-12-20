use regex::Regex;
use rocket::http::Status;
use rocket::post;
use rocket::serde::{json::Json, Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
struct Password {
    input: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(tag = "result", rename_all = "lowercase")]
enum Determination {
    Nice,
    Naughty,
}

#[post("/nice", data = "<pw>")]
fn nice_password(pw: Json<Password>) -> (Status, Json<Determination>) {
    const VOWELS: &[u8] = b"aeiouy";
    const NAUGHTY: [&[u8]; 4] = [b"ab", b"cd", b"pq", b"xy"];

    let vowel_count = |input: &str| input.bytes().filter(|b| VOWELS.contains(b)).count();
    let has_repeat_letter = |input: &str| {
        input
            .as_bytes()
            .windows(2)
            .any(|window| window[0] == window[1] && window[0].is_ascii_alphabetic())
    };
    let is_naughty = |input: &str| {
        input
            .as_bytes()
            .windows(2)
            .any(|window| NAUGHTY.contains(&window))
    };

    if vowel_count(&pw.input) >= 3 && has_repeat_letter(&pw.input) && !is_naughty(&pw.input) {
        (Status::Ok, Json(Determination::Nice))
    } else {
        (Status::BadRequest, Json(Determination::Naughty))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct GameOutcome {
    #[serde(flatten)]
    result: Determination,
    reason: String,
}

impl GameOutcome {
    fn nice(reason: &str) -> Self {
        GameOutcome {
            result: Determination::Nice,
            reason: String::from(reason),
        }
    }

    fn naughty(reason: &str) -> Self {
        GameOutcome {
            result: Determination::Naughty,
            reason: String::from(reason),
        }
    }
}

type GameErrorResponse = (Status, Json<GameOutcome>);

macro_rules! game_err {
    ($status:literal, $msg:literal) => {
        (Status::new($status), Json(GameOutcome::naughty($msg)))
    };
}

impl Password {
    fn rule1(&self) -> Result<(), GameErrorResponse> {
        if self.input.len() >= 8 {
            Ok(())
        } else {
            Err(game_err!(400, "8 chars"))
        }
    }

    fn rule2(&self) -> Result<(), GameErrorResponse> {
        let has_lower = self
            .input
            .chars()
            .any(|char| char.is_lowercase() && char.is_alphabetic());
        let has_upper = self
            .input
            .chars()
            .any(|char| char.is_uppercase() && char.is_alphabetic());
        let has_digit = self.input.chars().any(|char| char.is_ascii_digit());

        if has_lower && has_upper && has_digit {
            Ok(())
        } else {
            Err(game_err!(400, "more types of chars"))
        }
    }

    fn rule3(&self) -> Result<(), GameErrorResponse> {
        let digit_count = self
            .input
            .chars()
            .filter(|char| char.is_ascii_digit())
            .count();

        if digit_count >= 5 {
            Ok(())
        } else {
            Err(game_err!(400, "55555"))
        }
    }

    fn rule4(&self) -> Result<(), GameErrorResponse> {
        let numbers_re = Regex::new(r"\d+").unwrap();
        let total: i32 = numbers_re
            .find_iter(&self.input)
            .flat_map(|m| m.as_str().parse::<i32>())
            .sum();

        if total == 2023 {
            Ok(())
        } else {
            Err(game_err!(400, "math is hard"))
        }
    }

    fn rule5(&self) -> Result<(), GameErrorResponse> {
        let joy_re = Regex::new(r"j.+o.+y").unwrap();

        if joy_re.is_match(&self.input) {
            Ok(())
        } else {
            Err(game_err!(406, "not joyful enough"))
        }
    }

    fn rule6(&self) -> Result<(), GameErrorResponse> {
        let repeat_sandwich = self.input.as_bytes().windows(3).any(|window| {
            window[0] == window[2]
                && window[0].is_ascii_alphabetic()
                && window[1].is_ascii_alphabetic()
        });

        if repeat_sandwich {
            Ok(())
        } else {
            Err(game_err!(451, "illegal: no sandwich"))
        }
    }

    fn rule7(&self) -> Result<(), GameErrorResponse> {
        let range = '\u{2980}'..='\u{2BFF}';

        if self.input.chars().any(|c| range.contains(&c)) {
            Ok(())
        } else {
            Err(game_err!(416, "outranged"))
        }
    }

    fn rule8(&self) -> Result<(), GameErrorResponse> {
        let emoticons_block = '\u{1F600}'..='\u{1F64F}';
        let transport_block = '\u{1F680}'..='\u{1F6FF}';
        let supplemental_block = '\u{1F900}'..='\u{1F9FF}';

        if self.input.chars().any(|c| {
            emoticons_block.contains(&c)
                || transport_block.contains(&c)
                || supplemental_block.contains(&c)
        }) {
            Ok(())
        } else {
            Err(game_err!(426, "😳"))
        }
    }

    fn rule9(&self) -> Result<(), GameErrorResponse> {
        let s = sha256::digest(&self.input);

        if s.ends_with('a') {
            Ok(())
        } else {
            Err(game_err!(418, "not a coffee brewer"))
        }
    }
}

#[post("/game", data = "<pw>")]
fn game(pw: Json<Password>) -> Result<Json<GameOutcome>, GameErrorResponse> {
    pw.rule1()?;
    pw.rule2()?;
    pw.rule3()?;
    pw.rule4()?;
    pw.rule5()?;
    pw.rule6()?;
    pw.rule7()?;
    pw.rule8()?;
    pw.rule9()?;

    Ok(Json(GameOutcome::nice("that's a nice password")))
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![nice_password, game]
}
