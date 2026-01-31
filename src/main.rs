use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Combine {
        #[arg(short, long, required = true, num_args = 1..)]
        inputs: Vec<PathBuf>,

        #[arg(short, long)]
        output: PathBuf,
    },
    Compress {
        #[arg(short, long)]
        input: PathBuf,

        #[arg(short, long)]
        output: PathBuf,

        /// Constant Rate Factor (0-51, lower is better quality). Default is 23.
        #[arg(long, default_value_t = 23)]
        crf: u8,
    },
    AddMusic {
        #[arg(short, long)]
        video: PathBuf,

        #[arg(short, long)]
        audio: PathBuf,

        #[arg(short, long)]
        output: PathBuf,

        /// Volume of original video audio (0.0 to 1.0, etc).
        /// If specified, it will be mixed with the new audio.
        #[arg(long, default_value = "1.0")]
        reduce_original: String,
    },
    Timelapse {
        #[arg(short, long)]
        input: PathBuf,

        #[arg(short, long)]
        output: PathBuf,

        /// Speed factor (e.g. 10.0 for 10x speed)
        #[arg(short, long)]
        speed: f64,
    },
    Info {
        #[arg(short, long)]
        input: PathBuf,
    },
}

mod commands;
mod tui;

fn main() -> Result<()> {
    // Check if arguments were provided (other than the binary name)
    use commands::ProgressInfo;

    // ...

    if std::env::args().len() > 1 {
        let cli = Cli::parse();

        let print_progress = |info: ProgressInfo| {
            if let ProgressInfo::Log(log) = info {
                println!("{}", log);
            }
        };

        match &cli.command {
            Commands::Combine { inputs, output } => {
                commands::combine_videos(inputs, output, print_progress)?;
            }
            Commands::Compress { input, output, crf } => {
                commands::compress_video(input, output, *crf, print_progress)?;
            }
            Commands::AddMusic {
                video,
                audio,
                output,
                reduce_original,
            } => {
                commands::add_music(video, audio, output, reduce_original, print_progress)?;
            }
            Commands::Timelapse {
                input,
                output,
                speed,
            } => {
                commands::timelapse(input, output, *speed, print_progress)?;
            }
            Commands::Info { input } => {
                commands::get_info(input, print_progress)?;
            }
        }
    } else {
        // No args? Launch TUI
        tui::run()?;
    }

    Ok(())
}
