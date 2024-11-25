mod rp2040;
mod uf2_parser;

use std::fs::File;
use std::io;
use std::io::BufReader;
use crate::rp2040::RP2040;
use crate::uf2_parser::UF2File;


fn main() -> io::Result<()> {
    let file = File::open("tests/files/dc540.uf2")?;
    let mut reader = BufReader::new(file);

    let uf2_file = UF2File::parse_file(&mut reader)?;

    println!("Parsed UF2 file with {} blocks", uf2_file.blocks.len());
    println!(
        "Total payload size: {} bytes",
        uf2_file.total_payload_size()
    );

    if uf2_file.verify() {
        println!("UF2 file is valid and blocks are contiguous");
    } else {
        println!("UF2 file has issues with block continuity");
    }

    let mut rp2040 = RP2040::new();
    rp2040.start_emulation();

    Ok(())
}