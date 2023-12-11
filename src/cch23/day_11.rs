use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use rocket::form::{Form, FromForm};
use rocket::fs::{relative, NamedFile, TempFile};
use rocket::http::Status;
use rocket::{get, post};
use tokio::io::AsyncReadExt;

macro_rules! http_not_implemented {
    ($message:expr) => {
        (
            Status::NotImplemented,
            format!("Not Implemented: {}", $message),
        )
    };
}

#[get("/assets/<filename..>")]
async fn load_assets(filename: PathBuf) -> Result<NamedFile, (Status, String)> {
    let path = Path::new(relative!("assets")).join(&filename);
    let file = NamedFile::open(path).await;

    file.map_err(|err| match err.kind() {
        ErrorKind::NotFound => (
            Status::NotFound,
            format!("assets/{} was not found", filename.display()),
        ),
        ErrorKind::PermissionDenied => (Status::Forbidden, "Not Allowed".to_owned()),
        _ => (Status::InternalServerError, "Unknown IO Error".to_owned()),
    })
}

#[derive(Debug, Default)]
struct Pixel {
    red: u8,
    green: u8,
    blue: u8,
}

impl From<&[u8]> for Pixel {
    fn from(value: &[u8]) -> Self {
        let red = value[0];
        let green = value[1];
        let blue = value[2];

        Pixel { red, green, blue }
    }
}

impl Pixel {
    fn is_magic_red(&self) -> bool {
        let (sum, overflow) = u8::overflowing_add(self.green, self.blue);
        self.red > sum && !overflow
    }
}

#[derive(FromForm)]
struct DetectMagic<'r> {
    image: TempFile<'r>,
}

#[post("/red_pixels", data = "<detect_magic>")]
async fn count_red_pixels(detect_magic: Form<DetectMagic<'_>>) -> Result<String, (Status, String)> {
    let mut buf = Vec::with_capacity(detect_magic.image.len() as usize);
    let mut file = detect_magic.image.open().await.map_err(|err| {
        dbg!(&err);
        (Status::UnprocessableEntity, String::new())
    })?;
    let size = file.read_buf(&mut buf).await.map_err(|err| {
        dbg!(&err);
        (Status::UnprocessableEntity, String::new())
    })?;
    let bytes = &buf[..size];
    let decoder = png::Decoder::new(bytes);
    let mut reader = decoder.read_info().map_err(|err| {
        dbg!(&err);
        (Status::UnprocessableEntity, String::new())
    })?;
    let mut buffer = vec![0; reader.output_buffer_size()];
    let mut count_magic_red_pixels = 0;

    while let Ok(info) = reader.next_frame(&mut buffer) {
        let bytes = &buffer[..info.buffer_size()];
        let color_chunk_size: usize = match info.color_type {
            png::ColorType::Rgb => Ok(3),
            png::ColorType::Rgba => Ok(4),
            png::ColorType::Grayscale => Err(http_not_implemented!("Grayscale Color")),
            png::ColorType::GrayscaleAlpha => Err(http_not_implemented!("Grayscale Alpha Color")),
            png::ColorType::Indexed => Err(http_not_implemented!("Indexed Color")),
        }?;

        for color in bytes.chunks_exact(color_chunk_size) {
            let pixel: Pixel = color.into();

            if pixel.is_magic_red() {
                count_magic_red_pixels += 1;
            }
        }
    }

    Ok(count_magic_red_pixels.to_string())
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![load_assets, count_red_pixels]
}
