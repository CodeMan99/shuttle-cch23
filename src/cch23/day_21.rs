use dms_coordinates::DMS3d;
use google_maps::geocoding::GeocodingReverseRequest;
use google_maps::{GoogleMapsClient, Language, LatLng as GLatLng, PlaceType};
use rocket::http::Status;
use rocket::request::FromParam;
use rocket::{get, State};
use s2::cellid::CellID;
use s2::latlng::LatLng;

#[derive(Debug)]
struct S2CellIdParam(u64);

#[derive(Debug)]
enum S2CellIdParamError {
    NonBinaryDigit,
    IncorrectLength,
}

impl<'r> FromParam<'r> for S2CellIdParam {
    type Error = S2CellIdParamError;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        if param.len() == 64 {
            param
                .chars()
                .rev()
                .enumerate()
                .try_fold(0u64, |acc, (i, c)| match c {
                    '1' => Ok(acc | (1u64 << i)),
                    '0' => Ok(acc),
                    _ => Err(S2CellIdParamError::NonBinaryDigit),
                })
                .map(S2CellIdParam)
        } else {
            Err(S2CellIdParamError::IncorrectLength)
        }
    }
}

impl Into<LatLng> for S2CellIdParam {
    fn into(self) -> LatLng {
        CellID(self.0).into()
    }
}

impl Into<DMS3d> for S2CellIdParam {
    fn into(self) -> DMS3d {
        let pos: LatLng = CellID(self.0).into();
        DMS3d::from_decimal_degrees(pos.lat.deg(), pos.lng.deg(), None)
    }
}

#[get("/coords/<s2cell_id>")]
fn s2cell_to_dms(s2cell_id: S2CellIdParam) -> String {
    let DMS3d {
        latitude,
        longitude,
        ..
    } = s2cell_id.into();

    format!(
        "{}°{}'{:.3}''{} {}°{}'{:.3}''{}",
        latitude.degrees,
        latitude.minutes,
        latitude.seconds,
        latitude.bearing,
        longitude.degrees,
        longitude.minutes,
        longitude.seconds,
        longitude.bearing,
    )
}

#[get("/country/<s2cell_id>")]
async fn country_lookup(
    s2cell_id: S2CellIdParam,
    client: &State<GoogleMapsClient>,
) -> Result<String, (Status, String)> {
    let pos: LatLng = s2cell_id.into();
    let latlng = GLatLng::try_from_f64(pos.lat.deg(), pos.lng.deg())
        .map_err(|err| (Status::BadRequest, err.to_string()))?;
    let place_type = PlaceType::Country;
    let geocoding = GeocodingReverseRequest::new(client, latlng)
        .with_language(Language::EnglishUs)
        .with_result_type(place_type)
        .build()
        .execute()
        .await
        .map_err(|err| (Status::BadGateway, err.to_string()))?;
    let long_name = geocoding
        .results
        .iter()
        .flat_map(|geocode| geocode.address_components.iter())
        .find_map(|address_component| {
            if address_component.types.contains(&place_type) {
                Some(address_component.long_name.clone())
            } else {
                None
            }
        });

    if let Some(name) = long_name {
        Ok(name)
    } else {
        Err((Status::NotFound, "Reverse lookup failed".into()))
    }
}

pub fn create_google_maps_client(google_api_key: &str) -> GoogleMapsClient {
    GoogleMapsClient::new(google_api_key)
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![s2cell_to_dms, country_lookup]
}

#[cfg(test)]
mod tests_day_21 {
    use super::*;

    /// Enforce traits needed for rocket to manage state of <T>
    fn is_manage_safe<T: Send + Sync + 'static>() {}

    #[test]
    fn test_google_maps_client_is_manage_safe() {
        is_manage_safe::<GoogleMapsClient>()
    }
}
