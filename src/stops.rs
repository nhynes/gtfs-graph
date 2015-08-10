use std::path::Path;
use csv;
use Result;
use loader::decode_csv;

#[derive(RustcDecodable)]
pub struct Stop {
    pub id: String,
    pub lat: f64,
    pub lon: f64
}

pub fn get_stops<P: AsRef<Path>>(stopsfile: P) -> Result<Vec<Stop>> {
    let mut rdr = try!(csv::Reader::from_file(stopsfile));
    let cols = vec!["stop_id", "stop_lat", "stop_lon"];
    let stops = try!(decode_csv::<_, Stop>(&mut rdr, cols));
    stops.collect()
}
