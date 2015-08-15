extern crate csv;
extern crate rustc_serialize;
extern crate chrono;

mod loader;
mod stops;
mod connections;

use std::{error, fmt, num, result};
pub use stops::{Stop, get_stops};
pub use connections::{Connection, Trips, Services, get_connections};

// convenience type for Errors returned by this lib
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    LoadCSV(csv::Error),
    Parse(num::ParseFloatError),
    Data(String)
}

// Error wrapping
impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::LoadCSV(..) => "CSV load error",
            Error::Parse(..) => "GTFS feed parse error",
            Error::Data(..) => "Invalid input"
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::LoadCSV(ref err) => Some(err),
            Error::Parse(ref err) => Some(err),
            _ => None
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::LoadCSV(ref err) => write!(f, "{}", err),
            Error::Parse(ref err) => write!(f, "{}", err),
            Error::Data(ref msg) => write!(f, "{}", msg)
        }
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Error { Error::LoadCSV(err) }
}

impl From<num::ParseFloatError> for Error {
    fn from(err: num::ParseFloatError) -> Error { Error::Parse(err) }
}
