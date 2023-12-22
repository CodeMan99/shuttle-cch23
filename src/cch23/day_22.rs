use std::fmt::{self, Display};
use std::io::Cursor;
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;

use petgraph::prelude::*;
use petgraph::Graph;
use rocket::http::{ContentType, Status};
use rocket::post;
use rocket::response::{Responder, Response};

#[post("/integers", data = "<nums>")]
fn locate_gift_id(nums: String) -> String {
    let mut lines: Vec<_> = nums.lines().collect();

    lines.sort();

    let unique = lines
        .chunks(2)
        .find_map(|chunk| {
            if chunk.len() == 1 || chunk[0] != chunk[1] {
                Some(chunk[0])
            } else {
                None
            }
        })
        .unwrap();

    let count: usize = unique.parse().unwrap();

    String::from("🎁").repeat(count)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StarPosition {
    x: f32,
    y: f32,
    z: f32,
}

impl Display for StarPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{}, {}, {}}}", self.x, self.y, self.z)
    }
}

impl StarPosition {
    fn distance_to(&self, target: &StarPosition) -> f32 {
        let x = target.x - self.x;
        let y = target.y - self.y;
        let z = target.z - self.z;

        f32::sqrt(x * x + y * y + z * z)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StarPositionError {
    IncorrectDimensionCountError,
    ParseError(ParseFloatError),
}

impl Display for StarPositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncorrectDimensionCountError => write!(f, "Expected the text `x y z` exactly"),
            Self::ParseError(err) => write!(f, "{err}"),
        }
    }
}

impl From<ParseFloatError> for StarPositionError {
    fn from(value: ParseFloatError) -> Self {
        Self::ParseError(value)
    }
}

impl FromStr for StarPosition {
    type Err = StarPositionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let nums: Vec<f32> = s
            .split_whitespace()
            .map(|item| item.parse())
            .collect::<Result<_, _>>()?;

        if let &[x, y, z] = nums.as_slice() {
            Ok(StarPosition { x, y, z })
        } else {
            Err(StarPositionError::IncorrectDimensionCountError)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PortalLine {
    a: usize,
    b: usize,
}

impl Display for PortalLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.a, self.b)
    }
}

impl PortalLine {
    fn as_edge(&self) -> (u32, u32) {
        (self.a as u32, self.b as u32)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PortalLineError {
    NotLine,
    ParseError(ParseIntError),
}

impl Display for PortalLineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotLine => write!(f, "Expect the text `n n` exactly"),
            Self::ParseError(err) => write!(f, "{err}"),
        }
    }
}

impl From<ParseIntError> for PortalLineError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseError(value)
    }
}

impl FromStr for PortalLine {
    type Err = PortalLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let nums: Vec<usize> = s
            .split_whitespace()
            .map(|item| item.parse())
            .collect::<Result<_, _>>()?;

        if let &[a, b] = nums.as_slice() {
            Ok(PortalLine { a, b })
        } else {
            Err(PortalLineError::NotLine)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SpaceError {
    StarPositionError(StarPositionError),
    PortalLineError(PortalLineError),
    ParseError(ParseIntError),
    UnexpectedEndOfInput,
    NoPathFoundError,
}

impl Display for SpaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StarPositionError(err) => write!(f, "{err}"),
            Self::PortalLineError(err) => write!(f, "{err}"),
            Self::ParseError(err) => write!(f, "{err}"),
            Self::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            Self::NoPathFoundError => write!(f, "No path found"),
        }
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for SpaceError {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let status = if self == Self::NoPathFoundError {
            Status::NotFound
        } else {
            Status::BadRequest
        };
        let body = self.to_string();

        Response::build()
            .header(ContentType::Plain)
            .sized_body(body.len(), Cursor::new(body))
            .status(status)
            .ok()
    }
}

impl From<StarPositionError> for SpaceError {
    fn from(value: StarPositionError) -> Self {
        Self::StarPositionError(value)
    }
}

impl From<PortalLineError> for SpaceError {
    fn from(value: PortalLineError) -> Self {
        Self::PortalLineError(value)
    }
}

impl From<ParseIntError> for SpaceError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseError(value)
    }
}

#[post("/rocket", data = "<space>")]
fn shuttle_rocket(space: String) -> Result<String, SpaceError> {
    let mut lines = space.lines();
    let star_count: usize = match lines.next() {
        Some(item) => item.parse().map_err(Into::into),
        None => Err(SpaceError::UnexpectedEndOfInput),
    }?;
    let stars: Vec<StarPosition> = (0..star_count)
        .map(|_| match lines.next() {
            Some(item) => item.parse::<StarPosition>().map_err(Into::into),
            None => Err(SpaceError::UnexpectedEndOfInput),
        })
        .collect::<Result<_, _>>()?;
    let line_count: usize = match lines.next() {
        Some(item) => item.parse().map_err(Into::into),
        None => Err(SpaceError::UnexpectedEndOfInput),
    }?;
    let portal_lines: Vec<PortalLine> = (0..line_count)
        .map(|_| match lines.next() {
            Some(item) => item.parse::<PortalLine>().map_err(Into::into),
            None => Err(SpaceError::UnexpectedEndOfInput),
        })
        .collect::<Result<_, _>>()?;

    if let Some(line) = lines.next() {
        eprintln!("WARNING: unexpected line: {}", line);
    }

    let mut space: Graph<(), (), Directed> = Graph::with_capacity(star_count, line_count);

    space.extend_with_edges(portal_lines.iter().map(PortalLine::as_edge));

    let edge_cost: usize = 1;
    let finish = NodeIndex::from(star_count as u32 - 1);

    if let Some((jump_count, pathing)) =
        petgraph::algo::astar(&space, 0.into(), |f| f == finish, |_| edge_cost, |_| 0)
    {
        let mut pathing_iter = pathing.iter();
        let mut origin_star = match pathing_iter.next() {
            Some(node_index) => Ok(&stars[node_index.index()]),
            None => Err(SpaceError::NoPathFoundError),
        }?;
        let mut distance = 0.0;

        for node_index in pathing_iter {
            let dest_star = &stars[node_index.index()];
            distance += origin_star.distance_to(dest_star);
            origin_star = dest_star;
        }

        Ok(format!("{} {:.3}", jump_count, distance))
    } else {
        Err(SpaceError::NoPathFoundError)
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![locate_gift_id, shuttle_rocket]
}
