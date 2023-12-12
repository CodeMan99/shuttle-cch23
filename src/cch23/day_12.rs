use std::collections::HashMap;
use std::ops::Deref;

use rocket::http::Status;
use rocket::request::FromParam;
use rocket::serde::uuid::Uuid;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::time::error::InvalidVariant;
use rocket::time::{Duration, Instant, Month, OffsetDateTime as DateTime, Weekday};
use rocket::tokio::sync::RwLock;
use rocket::{get, post, State};
use ulid::Ulid;

#[derive(Debug, Default)]
pub struct Storage(RwLock<HashMap<String, Instant>>);

#[get("/load/<value>")]
async fn load<'r>(value: &'r str, storage: &State<Storage>) -> (Status, String) {
    let instant = Instant::now();
    let read_lock = storage.0.read().await;
    match read_lock.get(value) {
        Some(&start) => {
            let duration: Duration = instant - start;
            let seconds = duration.as_seconds_f32();
            (Status::Ok, format!("{seconds:.0}"))
        }
        None => (
            Status::NotFound,
            format!("No value found at /load/{}", value),
        ),
    }
}

#[post("/save/<value>")]
async fn save<'r>(value: &'r str, storage: &State<Storage>) {
    let instant = Instant::now();
    let mut write_lock = storage.0.write().await;
    write_lock.insert(value.to_string(), instant);
}

#[post("/ulids", data = "<ulids>")]
fn ulid_to_uuid(ulids: Json<Vec<Ulid>>) -> Json<Vec<Uuid>> {
    let uuids = ulids.iter().map(|&ulid| ulid.into()).rev().collect();
    Json(uuids)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct LsbReport {
    #[serde(rename(serialize = "christmas eve"))]
    christmas_eve: u32,
    weekday: u32,
    #[serde(rename(serialize = "in the future"))]
    future: u32,
    #[serde(rename(serialize = "LSB is 1"))]
    lsb_one: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
struct RWeekday(Weekday);

impl Deref for RWeekday {
    type Target = Weekday;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'r> FromParam<'r> for RWeekday {
    type Error = InvalidVariant;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        match param.parse() {
            Ok(0) => Ok(RWeekday(Weekday::Monday)),
            Ok(1) => Ok(RWeekday(Weekday::Tuesday)),
            Ok(2) => Ok(RWeekday(Weekday::Wednesday)),
            Ok(3) => Ok(RWeekday(Weekday::Thursday)),
            Ok(4) => Ok(RWeekday(Weekday::Friday)),
            Ok(5) => Ok(RWeekday(Weekday::Saturday)),
            Ok(6) => Ok(RWeekday(Weekday::Sunday)),
            _ => Err(InvalidVariant),
        }
    }
}

#[post("/ulids/<weekday>", data = "<ulids>")]
fn lsb(weekday: RWeekday, ulids: Json<Vec<Ulid>>) -> Json<LsbReport> {
    let mut report = LsbReport::default();
    let today = DateTime::now_utc();

    for ulid in ulids.iter() {
        let dt: DateTime = ulid.datetime().into();

        if dt.month() == Month::December && dt.day() == 24 {
            report.christmas_eve += 1;
        }

        if dt.weekday() == *weekday {
            report.weekday += 1;
        }

        if dt > today {
            report.future += 1;
        }

        let bytes = ulid.to_bytes();

        if bytes[15] & 1 == 1 {
            report.lsb_one += 1;
        }
    }

    Json(report)
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![load, save, ulid_to_uuid, lsb]
}

pub fn create_storage() -> Storage {
    Storage::default()
}

#[cfg(test)]
mod tests_day_12 {
    use super::*;

    /// Enforce traits needed for rocket to manage state of <T>
    fn is_manage_safe<T: Send + Sync + 'static>() {}

    #[test]
    fn test_rustemon_client_is_manage_safe() {
        is_manage_safe::<Storage>();
    }
}
