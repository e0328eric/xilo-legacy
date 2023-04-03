use std::path::Path;

use crate::error;

pub struct SpaceShower<'p> {
    trashbin_path: &'p Path,
}

impl<'p> SpaceShower<'p> {
    pub fn new(path: &'p Path) -> Self {
        Self {
            trashbin_path: path,
        }
    }

    pub fn get_raw_space(&self) -> error::Result<u64> {
        Ok(fs_extra::dir::get_size(self.trashbin_path)?)
    }

    pub fn get_space(&self) -> error::Result<String> {
        let space = fs_extra::dir::get_size(self.trashbin_path)?;
        Ok(space_data_formatter(space))
    }
}

fn space_data_formatter(raw_size: u64) -> String {
    const BYTE: u64 = 1;
    const KILOBYTE: u64 = 10u64.pow(3);
    const MEGABYTE: u64 = 10u64.pow(6);
    const GIGABYTE: u64 = 10u64.pow(9);
    const TERABYTE: u64 = 10u64.pow(12);
    const PETABYTE: u64 = 10u64.pow(15);
    const EXABYTE: u64 = 10u64.pow(18);

    if raw_size >= EXABYTE {
        return "Are you really using a normal computer?".to_string();
    }

    let divisor = |size: u64, cur_byte: u64, bigger_byte: u64| (size % bigger_byte) / cur_byte;

    let byte = divisor(raw_size, BYTE, KILOBYTE);
    let kilo_byte = divisor(raw_size, KILOBYTE, MEGABYTE);
    let mega_byte = divisor(raw_size, MEGABYTE, GIGABYTE);
    let giga_byte = divisor(raw_size, GIGABYTE, TERABYTE);
    let tera_byte = divisor(raw_size, TERABYTE, PETABYTE);
    let peta_byte = divisor(raw_size, PETABYTE, EXABYTE);

    let mut output = String::new();

    if peta_byte > 0 {
        output.push_str(&format!("{peta_byte}P "))
    }
    if tera_byte > 0 {
        output.push_str(&format!("{tera_byte}T "))
    }
    if giga_byte > 0 {
        output.push_str(&format!("{giga_byte}G "))
    }
    if mega_byte > 0 {
        output.push_str(&format!("{mega_byte}M "))
    }
    if kilo_byte > 0 {
        output.push_str(&format!("{kilo_byte}K "))
    }
    output.push_str(&format!("{byte}B"));

    output
}
