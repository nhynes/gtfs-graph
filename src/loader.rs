use csv;

use {Result, Error};

use rustc_serialize::Decodable;
use std::io;

pub struct DecodedFilteredRecords<'a, R: 'a, D> {
    r: csv::ByteRecords<'a, R>,
    inds: Vec<usize>,
    _phantom: ::std::marker::PhantomData<D>
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
