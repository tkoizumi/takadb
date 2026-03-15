mod nyc_311_temp_model;
use csv::{Error, Reader, ReaderBuilder};
use nyc_311_temp_model::RawRecord;
use std::fs::File;

use takadb::util;

fn main() -> Result<(), Error> {
    let file_name = "datasets/NYCopendata.csv_20260314.csv";
    let mut reader = get_reader(file_name)?;
    let headers = reader.headers()?;
    println!("{:#?}", headers);

    util::pack_string();

    for record in reader.deserialize().take(1) {
        let res: RawRecord = record?;
        println!("{:#?}", res);
    }

    Ok(())
}

fn get_reader(file_path: &str) -> Result<Reader<File>, Error> {
    let reader_builder = ReaderBuilder::new();
    reader_builder.from_path(file_path)
}
