use csv;

use {Result, Error};

use rustc_serialize::Decodable;
use std::io;

pub struct DecodedFilteredRecords<'a, R: 'a, D> {
    r: csv::ByteRecords<'a, R>,
    inds: Vec<usize>,
    _phantom: ::std::marker::PhantomData<D>
}

pub struct FilteredRecords<'a, R: 'a> {
    rdr: &'a mut csv::Reader<R>,
    ncols: usize,
    inds: Vec<Option<usize>>
}

impl<'a, R: io::Read, D: Decodable> Iterator for DecodedFilteredRecords<'a, R, D> {
    type Item = Result<D>;

    fn next(&mut self) -> Option<Result<D>> {
        self.r.next().map(|res| {
            let bytestr = try!(res);
            let sel_cols = self.inds.iter().map(|&ind| {
                bytestr[ind].clone()
            }).collect();
            Decodable::decode(&mut csv::Decoded::new(sel_cols)).map_err(Error::from)
        })
    }
}

impl<'a, R: io::Read> Iterator for FilteredRecords<'a, R> {
    type Item = Result<Vec<String>>;

    fn next(&mut self) -> Option<Result<Vec<String>>> {
        if self.rdr.done() {
            return None
        }
        let mut fields = vec![String::new(); self.ncols];
        // why does unsafe { fields.set_len } not work?
        let mut i = 0usize;
        loop {
            match self.rdr.next_str() {
                csv::NextField::EndOfCsv => return None,
                csv::NextField::EndOfRecord => return Some(Ok(fields)),
                csv::NextField::Error(err) => return Some(Err(Error::from(err))),
                csv::NextField::Data(d) => {
                    if let Some(ind) = self.inds[i] {
                        fields[ind] = d.into();
                    }
                    i += 1;
                }
            }
        }
    }
}

pub fn decode_csv<'a, R: io::Read, D: Decodable>(rdr: &'a mut csv::Reader<R>, cols: Vec<&str>) -> Result<DecodedFilteredRecords<'a, R, D>> {
    let headers = try!(rdr.headers());
    let mut inds:Vec<usize> = Vec::with_capacity(cols.len());

    for col in cols.iter() {
        if let Some(ind) = headers.iter().position(|h| h == col) { // n^2 < c(hashmap)
            inds.push(ind);
        } else {
            return Err(Error::Data(format!("Missing header: {}", col)));
        }
    }

    Ok(DecodedFilteredRecords {
        r: rdr.byte_records(),
        inds: inds,
        _phantom: ::std::marker::PhantomData
    })
}

pub fn read_csv<'a, R: io::Read>(rdr: &'a mut csv::Reader<R>, cols: Vec<&str>)
        -> Result<FilteredRecords<'a, R>> {
    let headers = try!(rdr.headers());
    let inds = headers.iter().map(|h| cols.iter().position(|c| h == c) ).collect();

    Ok(FilteredRecords {
        rdr: rdr,
        ncols: cols.len(),
        inds: inds
    })
}

#[cfg(test)]
mod tests {
    use loader::*;
    use csv::Reader;
    use {Stop};

    #[test]
    fn decode_csv_test() {
        let mut rdr = Reader::from_file("./tests/data/stops.txt").unwrap();
        let cols = vec!["stop_id", "stop_lat", "stop_lon"];
        let mut decoded = decode_csv::<_, Stop>(&mut rdr, cols).unwrap();
        let test = decoded.next().unwrap().unwrap();
        println!("{:?}", test.id);

        return ()
    }
}
