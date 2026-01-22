use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ts2mp4")]
#[command(about = "Convert TS/MP4 files and extract thumbnails", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert TS or MP4 file (with optional timestamp reset)
    Convert {
        /// Input file path (TS or MP4)
        #[arg(short, long)]
        input: PathBuf,

        /// Output MP4 file path
        #[arg(short, long)]
        output: PathBuf,

        /// Reset timestamps to start from 0 (like ffmpeg -avoid_negative_ts make_zero)
        #[arg(short, long, default_value_t = false)]
        reset_timestamps: bool,
    },
    /// Extract thumbnail from TS file
    ThumbnailTs {
        /// Input TS file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output JPEG file path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Extract thumbnail from MP4 file
    ThumbnailMp4 {
        /// Input MP4 file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output JPEG file path
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert {
            input,
            output,
            reset_timestamps,
        } => convert_command(&input, &output, reset_timestamps),
        Commands::ThumbnailTs { input, output } => extract_thumbnail_ts(&input, &output),
        Commands::ThumbnailMp4 { input, output } => extract_thumbnail_mp4(&input, &output),
    }
}

fn detect_file_type(data: &[u8]) -> FileType {
    // Check for MP4 signature (ftyp box)
    if data.len() >= 8 && &data[4..8] == b"ftyp" {
        return FileType::Mp4;
    }

    // Check for TS sync byte (0x47)
    if data.len() >= 188 && data[0] == 0x47 {
        return FileType::Ts;
    }

    FileType::Unknown
}

enum FileType {
    Ts,
    Mp4,
    Unknown,
}

fn convert_command(input: &PathBuf, output: &PathBuf, reset_timestamps: bool) -> io::Result<()> {
    eprintln!("Input: {}", input.display());
    eprintln!("Output: {}", output.display());
    if reset_timestamps {
        eprintln!("Timestamp reset: enabled");
    }

    let input_data = fs::read(input)?;
    let file_type = detect_file_type(&input_data);

    let mp4_data = match file_type {
        FileType::Ts => {
            eprintln!("Detected: MPEG-TS format");
            ts2mp4::convert_ts_to_mp4_with_options(&input_data, reset_timestamps)?
        }
        FileType::Mp4 => {
            eprintln!("Detected: MP4 format");
            if reset_timestamps {
                eprintln!("Converting Fragmented MP4 to regular MP4...");
                // Fragmented MP4를 일반 MP4로 변환 시도
                match ts2mp4::defragment_mp4(&input_data) {
                    Ok(data) => {
                        eprintln!("Defragmentation successful");
                        data
                    }
                    Err(_) => {
                        // Fragmented MP4가 아니면 timestamp reset만
                        eprintln!("Not fragmented, resetting timestamps...");
                        ts2mp4::reset_mp4_timestamps(&input_data)?
                    }
                }
            } else {
                eprintln!("No conversion needed, copying MP4 file...");
                input_data.clone()
            }
        }
        FileType::Unknown => {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Unknown file format. Expected TS or MP4 file.",
            ));
        }
    };

    fs::write(output, mp4_data)?;
    eprintln!("Conversion complete!");
    Ok(())
}

fn extract_thumbnail_ts(input: &PathBuf, output: &PathBuf) -> io::Result<()> {
    eprintln!("Extracting thumbnail from TS: {}", input.display());

    let ts_data = fs::read(input)?;
    let jpeg_data = ts2mp4::extract_thumbnail_from_ts(&ts_data)?;
    fs::write(output, jpeg_data)?;

    eprintln!("Thumbnail saved to {}", output.display());
    Ok(())
}

fn extract_thumbnail_mp4(input: &PathBuf, output: &PathBuf) -> io::Result<()> {
    eprintln!("Extracting thumbnail from MP4: {}", input.display());

    let mp4_data = fs::read(input)?;
    let jpeg_data = ts2mp4::extract_thumbnail_from_mp4(&mp4_data)?;
    fs::write(output, jpeg_data)?;

    eprintln!("Thumbnail saved to {}", output.display());
    Ok(())
}
