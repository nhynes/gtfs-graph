use std::hash::Hash;
use std::borrow::Borrow;
use std::result::Result as stdResult;
use std::path::Path;
use std::collections::HashSet;
use rustc_serialize::{Decoder, Decodable};
use csv;
use chrono::{Datelike, Duration, NaiveDate, NaiveTime, NaiveDateTime, Weekday};
use Result;
use loader::{read_csv, decode_csv};

fn dowstr(date: NaiveDate) -> &'static str {
    match date.weekday() {
        Weekday::Mon => "monday",
        Weekday::Tue => "tuesday",
        Weekday::Wed => "wednesday",
        Weekday::Thu => "thursday",
        Weekday::Fri => "friday",
        Weekday::Sat => "saturday",
        Weekday::Sun => "sunday"
    }
}

pub trait Running {
    fn is_running<Q: ?Sized>(&self, id: &Q) -> bool
        where String: Borrow<Q>, Q: Hash + Eq;
}

pub struct Trips {
    date: NaiveDate,
    ids: HashSet<String>
}

pub struct Services {
    date: NaiveDate,
    ids: HashSet<String>
}

impl Services {
    pub fn get_running<P: AsRef<Path>>(cal: P, caldates: P, date: NaiveDate) ->
            Result<Services> {
        let mut running_svcs = HashSet::new();

        let mut cal_rdr = try!(csv::Reader::from_file(cal));
        let mut cal_cols = vec!["service_id", "start_date", "end_date"];
        cal_cols.push(dowstr(date));

        let cal_entries = try!(decode_csv(&mut cal_rdr, cal_cols));

        for record in cal_entries {
            let (svc_id, start, end, running):
                (String, DecodedDate, DecodedDate, u8) = try!(record);
            if start.date < date && end.date > date && running == 1 {
                running_svcs.insert( svc_id );
            }
        }

        let mut caldates_rdr = try!(csv::Reader::from_file(caldates));
        let caldate_cols = vec!["service_id", "date", "exception_type"];
        let caldate_entries = try!(decode_csv(&mut caldates_rdr, caldate_cols));

        for record in caldate_entries {
            let (svc_id, caldate, exception): (String, DecodedDate, u8) = try!(record);
            if caldate.date != date { continue }
            match exception {
                1 => running_svcs.insert(svc_id),
                2 => running_svcs.remove(&svc_id),
                _ => unreachable!()
            };
        }

        Ok(Services { date: date, ids: running_svcs })
    }
}

impl Running for Services {
    fn is_running<Q: ?Sized>(&self, id: &Q) -> bool
        where String: Borrow<Q>, Q: Hash + Eq {
        self.ids.contains(id)
    }
}

impl Trips {
    pub fn get_running<P: AsRef<Path>>(trips: P, svcs: Services) -> Result<Trips> {
        let mut trip_ids = HashSet::new();

        let mut rdr = try!(csv::Reader::from_file(trips));
        let cols = vec!["service_id", "trip_id"];
        let trip_entries = try!(read_csv(&mut rdr, cols));

        for record in trip_entries {
            let ids = try!(record);
            if svcs.is_running(&ids[0]) {
                trip_ids.insert(ids[1].clone());
            }
        }

        Ok(Trips {ids: trip_ids, date: svcs.date })
    }
}

impl Running for Trips {
    fn is_running<Q: ?Sized>(&self, id: &Q) -> bool
        where String: Borrow<Q>, Q: Hash + Eq {
        self.ids.contains(id)
    }
}

struct DecodedDate {
    date: NaiveDate
}

impl Decodable for DecodedDate {
    fn decode<D: Decoder>(d: &mut D) -> stdResult<DecodedDate, D::Error> {
        let field = try!(d.read_str());
        NaiveDate::parse_from_str(&field, "%Y%m%d")
            .map(|d| { DecodedDate { date: d } })
            .or(Err(d.error("Invalid date found in feed file!")))
    }
}

struct StopTime {
    trip_id: String,
    stop_id: String,
    arrives: i64, // seconds since midnight on the date the trip began
    departs: i64
}

pub struct Connection {
    pub from: String,
    pub to: String,
    pub departs: NaiveDateTime,
    pub duration: Duration
}

// TODO: is it safe to trust the feed spec? probably not...
fn parse_timestamp(s: &String) -> i64 {
    let hh: i64 = s[0..2].parse().unwrap();
    let mm: i64 = s[3..5].parse().unwrap();
    let ss: i64 = s[6..8].parse().unwrap();
    ss + 60 * (mm + 60 * hh)
}

pub fn get_connections<P: AsRef<Path>>(times: P, trips: Trips)
        -> Result<Vec<Connection>> {
    let mut rdr = try!(csv::Reader::from_file(times));
    let cols = vec!["arrival_time", "departure_time", "stop_id", "trip_id"];
    let strecs = try!(read_csv::<_>(&mut rdr, cols));

    let mut connections = Vec::new();

    let base_dt = NaiveDateTime::new(trips.date, NaiveTime::from_hms(0, 0, 0));

    let mut prev_st = StopTime{trip_id: String::new(), stop_id: String::new(),
                               arrives: 0, departs: 0};

    for record in strecs {
        let mut stfields = try!(record);

        let trip_id = stfields.pop().unwrap();

        if !trips.is_running(&trip_id) { continue; }

        let st = StopTime {
            trip_id: trip_id,
            stop_id: stfields.pop().unwrap(),
            arrives: parse_timestamp(&stfields[0]),
            departs: parse_timestamp(&stfields[1])
        };

        // assumes that consecutive stops are grouped and ordered in file
        // so far, this seems to be reasonable
        if st.trip_id != prev_st.trip_id {
            prev_st = st;
            continue
        }

        if st.arrives <  prev_st.departs {
            panic!("Looks like we need to parse seqnums!");
        }

        let departs = base_dt + Duration::seconds(prev_st.departs);
        let duration = Duration::seconds(st.arrives - prev_st.departs);

        connections.push(Connection {
            from: prev_st.stop_id.clone(),
            to: st.stop_id.clone(),
            departs: departs,
            duration: duration
        });

        prev_st = st;
    }

    Ok(connections)
}
