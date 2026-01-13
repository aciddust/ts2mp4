use std::fs::File;
use std::io::{self, Read, Write};

mod mp4_writer;
mod ts_parser;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input.ts> <output.mp4>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    println!("Converting {} to {}", input_path, output_path);

    // Read TS file
    let mut input_file = File::open(input_path)?;
    let mut ts_data = Vec::new();
    input_file.read_to_end(&mut ts_data)?;

    // Convert to MP4
    let mp4_data = convert_ts_to_mp4(&ts_data)?;

    // Write MP4 file
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&mp4_data)?;

    println!("Conversion completed successfully!");

    Ok(())
}

pub fn convert_ts_to_mp4(ts_data: &[u8]) -> io::Result<Vec<u8>> {
    // Parse TS packets
    let media_data = ts_parser::parse_ts_packets(ts_data)?;

    // Create MP4 container
    let mp4_data = mp4_writer::create_mp4(media_data)?;

    Ok(mp4_data)
}
