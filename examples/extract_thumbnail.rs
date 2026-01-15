/// Example: Extract thumbnail from TS and MP4 files
///
/// This example demonstrates how to extract thumbnails from video files.
/// The extracted thumbnails are saved as raw H.264 NAL units which can be
/// converted to images using ffmpeg.
///
/// Usage:
///   cargo run --example extract_thumbnail -- input.ts thumbnail.h264
///
/// Then convert to image:
///   ffmpeg -i thumbnail.h264 -frames:v 1 thumbnail.jpg
use std::env;
use std::fs;
use ts2mp4::{extract_thumbnail_from_mp4, extract_thumbnail_from_ts};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input.ts|input.mp4> <output.h264>", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} input.ts thumbnail.h264", args[0]);
        eprintln!("  {} input.mp4 thumbnail.h264", args[0]);
        eprintln!();
        eprintln!("Convert to image:");
        eprintln!("  ffmpeg -i thumbnail.h264 -frames:v 1 thumbnail.jpg");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    println!("Reading input file: {}", input_path);
    let input_data = fs::read(input_path)?;

    let thumbnail_data = if input_path.ends_with(".ts") {
        println!("Extracting thumbnail from TS file...");
        extract_thumbnail_from_ts(&input_data)?
    } else if input_path.ends_with(".mp4") {
        println!("Extracting thumbnail from MP4 file...");
        extract_thumbnail_from_mp4(&input_data)?
    } else {
        eprintln!("Error: Input file must be .ts or .mp4");
        std::process::exit(1);
    };

    println!("Writing thumbnail to: {}", output_path);
    fs::write(output_path, &thumbnail_data)?;

    println!("✓ Thumbnail extracted successfully!");
    println!("  Size: {} bytes", thumbnail_data.len());
    println!();
    println!("To convert to image, run:");
    println!("  ffmpeg -i {} -frames:v 1 thumbnail.jpg", output_path);

    Ok(())
}
