mod nyc_311_temp_model;
use csv::{Error, Reader, ReaderBuilder};
use nyc_311_temp_model::RawRecord;
use std::fs::File;

use takadb::constants::SLOT_SIZE;
use takadb::util;

fn main() -> Result<(), Error> {
    let file_name = "datasets/NYCopendata.csv_20260314.csv";
    let mut reader = get_reader(file_name)?;

    for record in reader.deserialize().take(1) {
        let mut slot: [u8; SLOT_SIZE] = [0u8; SLOT_SIZE];
        let res: RawRecord = record?;

        slot[0..16].copy_from_slice(&util::pack_string::<16>(&res.unique_key));
        slot[16..32].copy_from_slice(&util::pack_string::<16>(&res.agency));
        slot[32..96].copy_from_slice(&util::pack_string::<64>(&res.complaint_type));
        slot[96..160].copy_from_slice(&util::pack_string::<64>(&res.descriptor));
        slot[160..224].copy_from_slice(&util::pack_string::<64>(&res.street_name));
        slot[224..256].copy_from_slice(&util::pack_string::<32>(&res.city));

        println!("{:#?}", res);
        println!("{:#?}", slot);
    }

    Ok(())
}

fn get_reader(file_path: &str) -> Result<Reader<File>, Error> {
    let reader_builder = ReaderBuilder::new();
    reader_builder.from_path(file_path)
}
