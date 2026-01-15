use std::fs::File;
use std::io::{self, Read, Write};

mod mp4_writer;
mod thumbnail;
mod ts_parser;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "convert" => {
            if args.len() != 4 {
                print_usage(&args[0]);
                std::process::exit(1);
            }
            convert_command(&args[2], &args[3])
        }
        "thumbnail-ts" => {
            if args.len() != 4 {
                print_usage(&args[0]);
                std::process::exit(1);
            }
            extract_thumbnail_ts(&args[2], &args[3])
        }
        "thumbnail-mp4" => {
            if args.len() != 4 {
                print_usage(&args[0]);
                std::process::exit(1);
            }
            extract_thumbnail_mp4(&args[2], &args[3])
        }
        _ => {
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }
}

fn print_usage(program: &str) {
    eprintln!("Usage:");
    eprintln!("  {} convert <input.ts> <output.mp4>", program);
    eprintln!("  {} thumbnail-ts <input.ts> <output.h264>", program);
    eprintln!("  {} thumbnail-mp4 <input.mp4> <output.h264>", program);
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  convert        Convert TS file to MP4");
    eprintln!("  thumbnail-ts   Extract thumbnail from TS file as H.264 frame");
    eprintln!("  thumbnail-mp4  Extract thumbnail from MP4 file as H.264 frame");
}

fn convert_command(input_path: &str, output_path: &str) -> io::Result<()> {
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

fn extract_thumbnail_ts(input_path: &str, output_path: &str) -> io::Result<()> {
    println!("Extracting thumbnail from TS file: {}", input_path);

    // Read TS file
    let mut input_file = File::open(input_path)?;
    let mut ts_data = Vec::new();
    input_file.read_to_end(&mut ts_data)?;

    // Extract thumbnail
    let thumbnail_data = thumbnail::extract_thumbnail_from_ts(&ts_data)?;

    // Write thumbnail file
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&thumbnail_data)?;

    println!(
        "Thumbnail extracted successfully! ({} bytes)",
        thumbnail_data.len()
    );
    println!("Output saved to: {}", output_path);

    Ok(())
}

fn extract_thumbnail_mp4(input_path: &str, output_path: &str) -> io::Result<()> {
    println!("Extracting thumbnail from MP4 file: {}", input_path);

    // Read MP4 file
    let mut input_file = File::open(input_path)?;
    let mut mp4_data = Vec::new();
    input_file.read_to_end(&mut mp4_data)?;

    // Extract thumbnail
    let thumbnail_data = thumbnail::extract_thumbnail_from_mp4(&mp4_data)?;

    // Write thumbnail file
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&thumbnail_data)?;

    println!(
        "Thumbnail extracted successfully! ({} bytes)",
        thumbnail_data.len()
    );
    println!("Output saved to: {}", output_path);

    Ok(())
}

pub fn convert_ts_to_mp4(ts_data: &[u8]) -> io::Result<Vec<u8>> {
    // Parse TS packets
    let media_data = ts_parser::parse_ts_packets(ts_data)?;

    // Create MP4 container
    let mp4_data = mp4_writer::create_mp4(media_data)?;

    Ok(mp4_data)
}
